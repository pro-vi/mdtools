//! Opt-in CLI stream measurements. No payload or path is persisted.

use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use serde::Serialize;

use crate::errors::DiagnosticCode;

// Bound transient payload retention for measurement, independently of output size.
const MAX_CAPTURE_BYTES: usize = 16 * 1024 * 1024;
static TOKENIZER: OnceLock<Result<tiktoken_rs::CoreBPE, ()>> = OnceLock::new();

#[derive(Clone, Copy, PartialEq, Eq)]
struct FileIdentity {
    device: u64,
    inode: u64,
}

fn identity(metadata: &fs::Metadata) -> Option<FileIdentity> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        Some(FileIdentity {
            device: metadata.dev(),
            inode: metadata.ino(),
        })
    }
    #[cfg(not(unix))]
    {
        let _ = metadata;
        None
    }
}

fn stdio_identities() -> Option<Vec<FileIdentity>> {
    #[cfg(unix)]
    {
        use std::os::fd::AsFd;
        let descriptors = [
            io::stdin().as_fd().try_clone_to_owned().ok()?,
            io::stdout().as_fd().try_clone_to_owned().ok()?,
            io::stderr().as_fd().try_clone_to_owned().ok()?,
        ];
        let mut identities = Vec::new();
        for descriptor in descriptors {
            let metadata = fs::File::from(descriptor).metadata().ok()?;
            if metadata.is_file() {
                identities.push(identity(&metadata)?);
            }
        }
        Some(identities)
    }
    // No measurement without the file-identity checks needed for noninterference.
    #[cfg(not(unix))]
    {
        None
    }
}

fn resolved_path(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| {
        path.parent()
            .and_then(|parent| fs::canonicalize(parent).ok())
            .and_then(|parent| path.file_name().map(|name| parent.join(name)))
            .unwrap_or_else(|| path.to_path_buf())
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Unavailable {
    CaptureLimit,
    InvalidUtf8,
    TokenizerInit,
    TokenizerCount,
    WriteFailed,
}

#[derive(Debug)]
pub struct Capture {
    bytes: u64,
    payload: Option<Vec<u8>>,
    limit: usize,
    pub write_failed: bool,
}

impl Capture {
    fn new(limit: usize) -> Self {
        Self {
            bytes: 0,
            payload: Some(Vec::new()),
            limit,
            write_failed: false,
        }
    }

    fn accept(&mut self, bytes: &[u8]) {
        self.bytes += bytes.len() as u64;
        if let Some(payload) = &mut self.payload {
            if bytes.len() <= self.limit.saturating_sub(payload.len()) {
                payload.extend_from_slice(bytes);
            } else {
                self.payload = None;
            }
        }
    }

    fn measure(&self) -> StreamMeasurement {
        if self.write_failed {
            return StreamMeasurement {
                bytes: Some(self.bytes),
                tokens: None,
                unavailable: Some(Unavailable::WriteFailed),
                write_failed: Some(true),
            };
        }
        let count = self
            .payload
            .as_deref()
            .ok_or(Unavailable::CaptureLimit)
            .and_then(|bytes| {
                let text = std::str::from_utf8(bytes).map_err(|_| Unavailable::InvalidUtf8)?;
                if text.is_empty() {
                    return Ok(0);
                }
                TOKENIZER
                    .get_or_init(|| tiktoken_rs::o200k_base().map_err(|_| ()))
                    .as_ref()
                    .map_err(|_| Unavailable::TokenizerInit)?
                    .count(text, &HashSet::new())
                    .map_err(|_| Unavailable::TokenizerCount)
            });
        StreamMeasurement {
            bytes: Some(self.bytes),
            tokens: count.as_ref().ok().copied(),
            unavailable: count.err(),
            write_failed: Some(self.write_failed),
        }
    }
}

pub struct MeasuredWriter<'a, W> {
    pub inner: W,
    pub capture: Option<&'a mut Capture>,
}

impl<W: Write> Write for MeasuredWriter<'_, W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let result = self.inner.write(bytes);
        if let Some(capture) = &mut self.capture {
            match &result {
                Ok(written) => {
                    capture.accept(&bytes[..*written]);
                    if *written == 0 && !bytes.is_empty() {
                        capture.write_failed = true;
                    }
                }
                Err(_) => capture.write_failed = true,
            }
        }
        result
    }

    fn flush(&mut self) -> io::Result<()> {
        let result = self.inner.flush();
        if result.is_err() {
            if let Some(capture) = &mut self.capture {
                capture.write_failed = true;
            }
        }
        result
    }
}

#[derive(Debug, Serialize)]
struct StreamMeasurement {
    bytes: Option<u64>,
    tokens: Option<usize>,
    unavailable: Option<Unavailable>,
    write_failed: Option<bool>,
}

#[derive(Serialize)]
struct Record<'a> {
    schema: &'static str,
    surface: &'static str,
    md_version: &'static str,
    process_id: u32,
    started_unix_ms: Option<u128>,
    command: &'a str,
    query_kind: Option<&'static str>,
    format: &'a str,
    exit: u8,
    diagnostic: Option<DiagnosticCode>,
    caller: &'static str,
    session: Option<String>,
    run: Option<String>,
    provenance: &'static str,
    command_ms: f64,
    counting_ms: f64,
    tokenizer: &'static str,
    tokenizer_implementation: &'static str,
    special_tokens: &'static str,
    stdout: StreamMeasurement,
    stderr: StreamMeasurement,
}

pub struct Usage {
    sink: PathBuf,
    cwd: PathBuf,
    protected_paths: Vec<PathBuf>,
    protected_files: Vec<FileIdentity>,
    input_identities_known: bool,
    started: Instant,
    started_unix_ms: Option<u128>,
    pub stdout: Capture,
    pub stderr: Capture,
    pub query_kind: Option<&'static str>,
    pub format: &'static str,
}

impl Usage {
    pub fn from_env() -> Option<Self> {
        let sink = std::env::var_os("MD_USAGE_LOG").filter(|value| !value.is_empty())?;
        let cwd = std::env::current_dir().ok()?;
        let sink = cwd.join(sink);
        let protected_files = stdio_identities()?;
        Some(Self {
            sink,
            cwd,
            protected_paths: Vec::new(),
            protected_files,
            input_identities_known: true,
            started: Instant::now(),
            started_unix_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .ok()
                .map(|time| time.as_millis()),
            stdout: Capture::new(MAX_CAPTURE_BYTES),
            stderr: Capture::new(MAX_CAPTURE_BYTES),
            query_kind: None,
            format: "unknown",
        })
    }

    pub fn protect_path(&mut self, path: &Path) {
        let path = self.cwd.join(path);
        match fs::metadata(&path) {
            Ok(metadata) if metadata.is_file() => {
                if let Some(identity) = identity(&metadata) {
                    self.protected_files.push(identity);
                } else {
                    self.input_identities_known = false;
                }
            }
            _ => self.input_identities_known = false,
        }
        let canonical = fs::canonicalize(&path).unwrap_or_else(|_| {
            self.input_identities_known = false;
            path.clone()
        });
        if canonical != path {
            self.protected_paths.push(canonical);
        }
        self.protected_paths.push(path);
    }

    fn sink_aliases_input_or_output(&self, opened: Option<&fs::Metadata>) -> bool {
        let sink_path = resolved_path(&self.sink);
        if self
            .protected_paths
            .iter()
            .any(|path| resolved_path(path) == sink_path)
        {
            return true;
        }
        let path_metadata = fs::metadata(&self.sink).ok();
        let Some(sink) = opened.or(path_metadata.as_ref()).and_then(identity) else {
            return false;
        };
        self.protected_files.contains(&sink)
            || self
                .protected_paths
                .iter()
                .any(|path| fs::metadata(path).ok().as_ref().and_then(identity) == Some(sink))
    }

    pub fn finish(self, command: &str, exit: u8, diagnostic: Option<DiagnosticCode>) {
        if !self.input_identities_known || self.sink_aliases_input_or_output(None) {
            return;
        }
        let command_ms = self.started.elapsed().as_secs_f64() * 1000.0;
        let counting = Instant::now();
        let (stdout, stderr) = (self.stdout.measure(), self.stderr.measure());
        let caller = match std::env::var("MD_USAGE_CALLER").as_deref() {
            Ok("codex") => "codex",
            Ok("claude") => "claude",
            Ok("pi") => "pi",
            Ok("human") => "human",
            Ok("benchmark") => "benchmark",
            _ => "unknown",
        };
        let record = Record {
            schema: "md-usage.v1",
            surface: "cli_streams",
            md_version: env!("CARGO_PKG_VERSION"),
            process_id: std::process::id(),
            started_unix_ms: self.started_unix_ms,
            command,
            query_kind: self.query_kind,
            format: self.format,
            exit,
            diagnostic,
            caller,
            session: claimed_id("MD_USAGE_SESSION"),
            run: claimed_id("MD_USAGE_RUN"),
            provenance: "explicit_environment_claims_only",
            command_ms,
            counting_ms: counting.elapsed().as_secs_f64() * 1000.0,
            tokenizer: "o200k_base",
            tokenizer_implementation: "tiktoken-rs/0.12.0",
            special_tokens: "ordinary_text",
            stdout,
            stderr,
        };
        // Measurement failure must not alter command output or its exit status.
        let _ = self.append(&record);
    }

    fn append(&self, record: &Record<'_>) -> io::Result<()> {
        // A dangling link may target a not-yet-created sink. Do not create or
        // append any sink when an input identity could not be established.
        if !self.input_identities_known {
            return Ok(());
        }
        if self.sink_aliases_input_or_output(None) {
            return Ok(());
        }
        if let Some(parent) = self
            .sink
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            let mut builder = fs::DirBuilder::new();
            builder.recursive(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(0o700);
            }
            builder.create(parent)?;
        }
        // Parent creation can make formerly missing path aliases resolvable.
        if self.sink_aliases_input_or_output(None) {
            return Ok(());
        }
        let mut options = OpenOptions::new();
        options.create(true).append(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options
                .mode(0o600)
                .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
        }
        let mut file = options.open(&self.sink)?;
        let metadata = file.metadata()?;
        // Recheck the opened inode and current paths after an atomic document replacement.
        if self.sink_aliases_input_or_output(Some(&metadata)) {
            return Ok(());
        }
        if !metadata.is_file() {
            return Err(io::Error::other("usage sink is not a regular file"));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::{MetadataExt, PermissionsExt};
            if metadata.permissions().mode() & 0o077 != 0 || metadata.nlink() != 1 {
                return Err(io::Error::other("usage sink is not private"));
            }
        }
        let mut line = serde_json::to_vec(record).map_err(io::Error::other)?;
        line.push(b'\n');
        file.write_all(&line)
    }
}

fn claimed_id(key: &str) -> Option<String> {
    let value = std::env::var(key).ok()?;
    // Accept only UUID-shaped explicit IDs, never arbitrary environment text.
    (value.len() == 36
        && value.bytes().enumerate().all(|(index, byte)| {
            if matches!(index, 8 | 13 | 18 | 23) {
                byte == b'-'
            } else {
                byte.is_ascii_hexdigit()
            }
        }))
    .then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_the_complete_stream_not_each_write_chunk() {
        let mut capture = Capture::new(100);
        let mut writer = MeasuredWriter {
            inner: Vec::new(),
            capture: Some(&mut capture),
        };
        writer.write_all(b"hel").unwrap();
        writer.write_all(b"lo").unwrap();
        assert_eq!(capture.measure().tokens, Some(1));
        assert_eq!(capture.measure().bytes, Some(5));
    }

    #[test]
    fn capture_limit_is_unavailable_not_a_partial_or_zero_count() {
        let mut capture = Capture::new(4);
        capture.accept(b"hello");
        capture.accept(b" world");
        let measured = capture.measure();
        assert_eq!(measured.bytes, Some(11));
        assert_eq!(measured.tokens, None);
        assert_eq!(measured.unavailable, Some(Unavailable::CaptureLimit));
    }

    #[test]
    fn unicode_can_cross_write_boundaries_and_empty_is_known_zero() {
        let mut capture = Capture::new(100);
        assert_eq!(capture.measure().tokens, Some(0));
        let text = "文字\r\n".as_bytes();
        capture.accept(&text[..1]);
        assert_eq!(
            capture.measure().unavailable,
            Some(Unavailable::InvalidUtf8)
        );
        capture.accept(&text[1..]);
        assert_eq!(capture.measure().bytes, Some(text.len() as u64));
        assert!(capture.measure().tokens.is_some());
    }

    #[test]
    fn captures_only_successfully_written_bytes_on_failure() {
        struct Partial;
        impl Write for Partial {
            fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
                if bytes.starts_with(b"hello") {
                    Ok(2)
                } else {
                    Err(io::ErrorKind::BrokenPipe.into())
                }
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let mut capture = Capture::new(100);
        let mut writer = MeasuredWriter {
            inner: Partial,
            capture: Some(&mut capture),
        };
        assert!(writer.write_all(b"hello").is_err());
        assert_eq!(capture.measure().bytes, Some(2));
        assert!(capture.write_failed);
        assert_eq!(
            capture.measure().unavailable,
            Some(Unavailable::WriteFailed)
        );
    }
}
