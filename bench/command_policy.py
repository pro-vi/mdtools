"""CLI condition names recovered from policy source P, c8e0813.

The old eager inventory load and shell-syntax guard are deliberately omitted.
Pinned producer schemas and executable staging are controller-owned. Native
containment belongs to U5; PATH staging alone is not containment.
"""

from __future__ import annotations

from enum import Enum
from dataclasses import asdict, dataclass
import hashlib
import io
import json
from pathlib import Path
import shutil
import subprocess
import tarfile
from typing import Sequence
from typing import Mapping
from types import MappingProxyType
import os
import platform
from urllib.parse import urlsplit

from bench.manifest import canonical_json, sha256_file, sha256_text


class CliCondition(str, Enum):
    NO_MD = "no-md"
    LEGACY = "legacy"
    CURRENT_COMPACT = "current-compact"


# P's complete ordinary toolkit, with jq equalized across future conditions.
UNIX_TOOLS = ("cat", "grep", "sed", "awk", "head", "tail", "wc", "tee", "mv", "cp", "mktemp")
# Claude's initialization runs `env`; freeze this runtime requirement in all arms.
ORDINARY_TOOLS = (*UNIX_TOOLS, "jq", "env")

SOURCE_PINS = {
    CliCondition.LEGACY: "4d857d2d8ab39d60613498e4a732c66a4494f4be",
    CliCondition.CURRENT_COMPACT: "6daa2d8f87497928cabd2b4f895b0b64d6b458f7",
}
UNAVAILABLE_MD = b'#!/bin/sh\nprintf "%s\\n" "md: unavailable here; use standard unix tools." >&2\nexit 1\n'


@dataclass(frozen=True)
class CommandMetadata:
    name: str
    kind: str
    summary: str


@dataclass(frozen=True)
class ConditionPin:
    condition: CliCondition
    executable: str
    source_commit: str | None
    source_sha256: str
    build_sha256: str
    binary_sha256: str
    schema_sha256: str | None
    schema_path: str | None

    def __post_init__(self) -> None:
        if not isinstance(self.condition, CliCondition):
            raise ValueError("unknown CLI condition")
        for digest in (self.source_sha256, self.build_sha256, self.binary_sha256, self.schema_sha256):
            if digest is not None and (not isinstance(digest, str) or len(digest) != 64 or any(char not in "0123456789abcdef" for char in digest)):
                raise ValueError("invalid condition digest")

    @property
    def content_identity(self) -> str:
        # Locators may change without changing executable/source/schema identity.
        return sha256_text(canonical_json({key: value for key, value in asdict(self).items()
            if key not in ("executable", "schema_path")}))


def decode_schema(payload: bytes, condition: CliCondition) -> tuple[CommandMetadata, ...]:
    """Decode only producer command metadata, never a selector or patch schema."""
    raw = json.loads(payload)
    if not isinstance(raw, dict) or not isinstance(raw.get("schema_version"), str):
        raise ValueError("invalid producer schema envelope")
    entries = raw.get("commands")
    if not isinstance(entries, list) or not entries:
        raise ValueError("producer schema has no commands")
    commands = []
    for entry in entries:
        if not isinstance(entry, dict) or not isinstance(entry.get("name"), str) or not entry["name"]:
            raise ValueError("invalid producer command name")
        if condition == CliCondition.LEGACY:
            if raw.get("binary_version") != "0.2.0" or entry.get("kind") not in ("query", "mutation"):
                raise ValueError("invalid legacy command metadata")
            if not isinstance(entry.get("args"), list) or not isinstance(entry.get("flags"), list):
                raise ValueError("invalid legacy command arguments")
            kind, summary = entry["kind"], ""
        elif condition == CliCondition.CURRENT_COMPACT:
            if raw["schema_version"] != "mdtools.v3" or type(entry.get("mutating")) is not bool:
                raise ValueError("invalid current command metadata")
            if any(not isinstance(entry.get(key), str) for key in ("summary", "input", "output")):
                raise ValueError("invalid current command description")
            kind = "mutation" if entry["mutating"] else "query"
            summary = entry["summary"]
        else:
            raise ValueError("no-md has no producer schema")
        commands.append(CommandMetadata(entry["name"], kind, summary))
    if len({command.name for command in commands}) != len(commands):
        raise ValueError("duplicate producer command")
    return tuple(commands)


def resolve_toolkit(search_path: str = "/usr/bin:/bin:/opt/homebrew/bin") -> dict[str, str]:
    """Freeze actual executable/symlink targets for the identical ordinary toolkit."""
    resolved = {}
    for name in (*ORDINARY_TOOLS, "bash", "sh"):
        found = shutil.which(name, path=search_path)
        if found is None:
            raise ValueError(f"required executable unavailable: {name}")
        executable = Path(found).resolve(strict=True)
        if not executable.is_file() or not os.access(executable, os.X_OK):
            raise ValueError(f"nonexecutable toolkit object: {name}")
        resolved[name] = str(executable)
    return resolved


def verify_condition(pin: ConditionPin) -> tuple[CommandMetadata, ...]:
    executable = Path(pin.executable).resolve(strict=True)
    if not executable.is_file() or not os.access(executable, os.X_OK) or sha256_file(executable) != pin.binary_sha256:
        raise ValueError("condition binary digest mismatch")
    if pin.condition == CliCondition.NO_MD:
        if executable.read_bytes() != UNAVAILABLE_MD or pin.schema_sha256 is not None or pin.schema_path is not None or pin.source_commit is not None:
            raise ValueError("invalid unavailable-md condition")
        if pin.source_sha256 != pin.binary_sha256 or pin.build_sha256 != pin.binary_sha256:
            raise ValueError("unavailable-md content identity mismatch")
        return ()
    if pin.source_commit != SOURCE_PINS[pin.condition] or pin.schema_path is None:
        raise ValueError("condition source pin mismatch")
    payload = Path(pin.schema_path).resolve(strict=True).read_bytes()
    if hashlib.sha256(payload).hexdigest() != pin.schema_sha256:
        raise ValueError("stored schema digest mismatch")
    build = json.loads(Path(pin.schema_path).resolve(strict=True).with_name("build.json").read_bytes())
    if not isinstance(build, dict) or sha256_text(canonical_json(build)) != pin.build_sha256 or build.get("source_sha256") != pin.source_sha256:
        raise ValueError("condition source/build digest mismatch")
    actual = subprocess.run([str(executable), "--json", "schema"], capture_output=True,
        env={"PATH": "/usr/bin:/bin"}, timeout=30, check=True).stdout
    if actual != payload:
        raise ValueError("runtime schema mismatch")
    version = subprocess.run([str(executable), "--version"], capture_output=True,
        env={"PATH": "/usr/bin:/bin"}, timeout=30, check=True).stdout.decode().strip()
    expected_version = "md 0.2.0" if pin.condition == CliCondition.LEGACY else "md 0.4.1"
    if version != expected_version:
        raise ValueError("runtime binary version mismatch")
    return decode_schema(payload, pin.condition)


def build_pinned_condition(repo: Path, condition: CliCondition, output: Path,
                           *, cargo: Path) -> ConditionPin:
    """Build a pinned Git source export; never change the active checkout."""
    if condition not in SOURCE_PINS:
        raise ValueError("only legacy/current have source builds")
    source_commit = SOURCE_PINS[condition]
    archive = subprocess.run(["git", "archive", source_commit, "Cargo.toml", "Cargo.lock", "README.crate.md", "src", "benches"],
        cwd=repo, capture_output=True, check=True).stdout
    output = output.resolve()
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    source = output / "source"
    source.mkdir(mode=0o700)
    with tarfile.open(fileobj=io.BytesIO(archive)) as bundle:
        # Trusted pinned Git export, still refuse links and path traversal.
        for member in bundle.getmembers():
            if member.issym() or member.islnk() or Path(member.name).is_absolute() or ".." in Path(member.name).parts:
                raise ValueError("unsafe source archive member")
        bundle.extractall(source)
    # rustup dispatch depends on argv[0]'s name; preserve the cargo shim path.
    cargo = cargo.absolute()
    rustc = cargo.with_name("rustc")
    command = [str(cargo), "build", "--locked", "--release", "--bin", "md"]
    build_env = {"PATH": f"{cargo.parent}:/usr/bin:/bin", "HOME": str(Path.home()), "LC_ALL": "C"}
    build = {"argv": ["cargo", *command[1:]], "environment": "cleared; PATH toolchain/system, HOME normal home, LC_ALL C",
        "cargo": subprocess.run([str(cargo), "--version"], env=build_env, capture_output=True, check=True).stdout.decode().strip(),
        "rustc": subprocess.run([str(rustc), "-vV"], env=build_env, capture_output=True, check=True).stdout.decode().strip(),
        "source_sha256": hashlib.sha256(archive).hexdigest(), "lock_sha256": sha256_file(source / "Cargo.lock")}
    completed = subprocess.run(command, cwd=source, env=build_env, capture_output=True, timeout=600)
    _write_pin_file(output / "build.stdout", completed.stdout)
    _write_pin_file(output / "build.stderr", completed.stderr)
    if completed.returncode != 0:
        raise ValueError(f"pinned build failed; evidence: {output}")
    executable = output / "md"
    shutil.copyfile(source / "target/release/md", executable)
    executable.chmod(0o500)
    schema = subprocess.run([str(executable), "--json", "schema"], capture_output=True, check=True, timeout=30).stdout
    decode_schema(schema, condition)
    _write_pin_file(output / "schema.json", schema)
    _write_pin_file(output / "build.json", canonical_json(build).encode())
    pin = ConditionPin(condition, str(executable), source_commit, build["source_sha256"],
        sha256_text(canonical_json(build)), sha256_file(executable), hashlib.sha256(schema).hexdigest(), str(output / "schema.json"))
    _write_pin_file(output / "pin.json", canonical_json(asdict(pin)).encode())
    verify_condition(pin)
    return pin


def _write_pin_file(path: Path, payload: bytes) -> None:
    with path.open("xb") as handle:
        path.chmod(0o600)
        handle.write(payload)


def stage_condition(pin: ConditionPin | None, output: Path, *, toolkit: dict[str, str]) -> ConditionPin:
    """Stage one selected md and shared utilities. This is not OS containment."""
    if set(toolkit) != set((*ORDINARY_TOOLS, "bash", "sh")):
        raise ValueError("toolkit names differ from frozen ordinary set")
    if pin is not None:
        verify_condition(pin)
    output = output.resolve()
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    for name, source in toolkit.items():
        target = Path(source).resolve(strict=True)
        if not target.is_file() or not os.access(target, os.X_OK):
            raise ValueError("invalid toolkit executable")
        (output / name).symlink_to(target)
    executable = output / "md"
    if pin is None:
        _write_pin_file(executable, UNAVAILABLE_MD)
        executable.chmod(0o500)
        digest = hashlib.sha256(UNAVAILABLE_MD).hexdigest()
        return ConditionPin(CliCondition.NO_MD, str(executable), None, digest, digest, digest, None, None)
    shutil.copyfile(Path(pin.executable).resolve(strict=True), executable)
    executable.chmod(0o500)
    return ConditionPin(pin.condition, str(executable), pin.source_commit, pin.source_sha256,
        pin.build_sha256, pin.binary_sha256, pin.schema_sha256, pin.schema_path)


def tool_reference(pin: ConditionPin) -> str:
    commands = verify_condition(pin)
    lines = ["Ordinary tools: " + ", ".join(ORDINARY_TOOLS) + ". Bash builtins are available."]
    lines.append('Use mktemp "$TMPDIR/example.XXXXXX" to create temporary files inside scratch.')
    if pin.condition == CliCondition.NO_MD:
        return "\n".join([*lines, "md is unavailable; its stub exits 1. Use ordinary tools."])
    lines.append(f"CLI condition: {pin.condition.value}; explicit --json is available.")
    for command in commands:
        help_output = subprocess.run([pin.executable, command.name, "--help"], capture_output=True,
            env={"PATH": "/usr/bin:/bin"}, timeout=30, check=True).stdout.decode()
        lines.extend([f"md {command.name} ({command.kind}): {command.summary}", help_output.rstrip()])
    return "\n".join(lines)


def classify_md_argv(argv: Sequence[str], commands: tuple[CommandMetadata, ...]) -> str | None:
    """Observed direct argv classification, never shell enforcement/mutation proof."""
    if not argv:
        return None
    names = {command.name: command.kind for command in commands}
    for token in argv[1:]:
        if token == "--json":
            continue
        return names.get(token)
    return None


def build_runner_command(command: Sequence[str], *, runner: str) -> list[str]:
    """Admit an explicitly supplied synthetic executable, never a provider default.

The synthetic command is trusted controller code, not agent-controlled code.
This validation is not a network or filesystem sandbox.
"""
    if runner != "synthetic":
        raise ValueError("unsupported runner: only synthetic execution is available")
    if isinstance(command, (str, bytes)) or not isinstance(command, Sequence) or not command:
        raise ValueError("synthetic command must be a nonempty argv sequence")
    if any(not isinstance(arg, str) or "\0" in arg for arg in command):
        raise ValueError("synthetic argv entries must be strings without NUL")
    executable = Path(command[0])
    if any(component.is_symlink() for component in (executable, *executable.parents)):
        raise ValueError("symlink component in synthetic executable")
    if not executable.is_absolute() or not executable.is_file() or not os.access(executable, os.X_OK):
        raise ValueError("synthetic executable must be an absolute executable file")
    return list(command)


CLAUDE_VERSION = "2.1.272"
CLAUDE_SHA256 = "195e24e8e1f9bf46f1eaee72d434a33e18f9f5796f29a6348a00d16c5f8aee75"
DYLD_PROFILE = Path("/System/Library/Sandbox/Profiles/dyld-support.sb")
CLAUDE_FLAGS = (
    "--print", "--safe-mode", "--restricted", "--tools", "Bash", "--allowedTools", "Bash",
    "--permission-mode", "dontAsk", "--permission-prompts", "none", "--disable-slash-commands",
    "--strict-mcp-config", "--mcp-config", '{"mcpServers":{}}', "--setting-sources", "",
    "--settings", '{"sandbox":{"enabled":false},"disableAllHooks":true}',
    "--no-session-persistence", "--no-chrome", "--output-format", "stream-json", "--verbose",
)


@dataclass(frozen=True)
class LocalClaudeRunner:
    """Pinned CLI with a caller-owned loopback scripted endpoint, never a model.

This is offline integration configuration, not a paid-run grant. Real provider
launches remain held; the synthetic key and empty config cannot reuse user auth.
"""
    executable: str
    endpoint: str
    model: str
    effort: str | None
    thinking_policy: str
    max_turns: int = 30
    attempt_usd: float = 0.1

    def __post_init__(self) -> None:
        import math
        endpoint = urlsplit(self.endpoint)
        if (endpoint.scheme != "http" or endpoint.hostname != "127.0.0.1" or
            endpoint.port is None or endpoint.username is not None or endpoint.password is not None or
            endpoint.path not in ("", "/") or endpoint.query or endpoint.fragment):
            raise ValueError("scripted endpoint must be explicit IPv4 loopback HTTP")
        choices = {"claude-sonnet-5": ("high", "adaptive"),
                   "claude-haiku-4-5-20251001": (None, "disabled")}
        if choices.get(self.model) != (self.effort, self.thinking_policy):
            raise ValueError("unverified model/effort/thinking configuration")
        if type(self.max_turns) is not int or self.max_turns <= 0:
            raise ValueError("positive integer turn limit required")
        if type(self.attempt_usd) not in (int, float) or not math.isfinite(self.attempt_usd) or self.attempt_usd <= 0:
            raise ValueError("positive finite attempt cap required")

    def verify(self) -> None:
        if platform.system() != "Darwin":
            raise ValueError("native runner requires macOS")
        build_runner_command([self.executable], runner="synthetic")
        if sha256_file(self.executable) != CLAUDE_SHA256:
            raise ValueError("preserved Claude binary changed; no fallback")
        version = subprocess.run([self.executable, "--version"], env={"PATH": "/usr/bin:/bin", "DISABLE_AUTOUPDATER": "1"},
            capture_output=True, check=True, timeout=10).stdout.decode().strip()
        if version != CLAUDE_VERSION + " (Claude Code)":
            raise ValueError("preserved Claude version mismatch")

    def command(self) -> list[str]:
        argv = [self.executable, *CLAUDE_FLAGS, "--model", self.model,
                "--max-turns", str(self.max_turns), "--max-budget-usd", str(self.attempt_usd)]
        if self.effort is not None:
            argv.extend(["--effort", self.effort])
        return argv


def native_profile(workspace: Path, toolkit: dict[str, str]) -> str:
    """Render one default-deny Bash profile; no shell-syntax enforcement."""
    def literal(path: str | Path) -> str:
        return "(literal " + json.dumps(str(path)) + ")"
    def subpath(path: str | Path) -> str:
        return "(subpath " + json.dumps(str(path)) + ")"
    executables = [*toolkit.values(), str(workspace / "bin/md")]
    staged_tools = [workspace / "bin" / name for name in toolkit]
    writable = [workspace / "fixtures", workspace / "scratch", workspace / "control/config/shell-snapshots"]
    ancestors = {parent for target in [*writable, *(Path(p) for p in executables)] for parent in target.parents}
    return "\n".join([
        "(version 1) (deny default)",
        '(import "/System/Library/Sandbox/Profiles/dyld-support.sb")',
        # sysconf(_SC_PAGESIZE) returns -1 without this read; Rust then aborts
        # while mapping its stack guard. Other process/kernel data stays denied.
        '(allow sysctl-read (sysctl-name "hw.pagesize_compat"))',
        "(allow process-fork)",
        "(allow process-exec " + " ".join(map(literal, executables)) + ")",
        "(allow file-read* " + " ".join(map(subpath, ["/System/Library", "/usr/lib", "/usr/share", *writable])) +
            " " + " ".join(map(literal, [*executables, *staged_tools, "/dev/null", "/dev/random", "/dev/urandom"])) + ")",
        "(allow file-read-metadata " + " ".join(map(literal, sorted(ancestors))) + ")",
        "(allow file-write* " + " ".join(map(subpath, writable)) + ' (literal "/dev/null"))', "",
    ])


@dataclass(frozen=True)
class NativeBoundary:
    workspace: Path
    launcher: Path
    registry: Path
    hashes: Mapping[str, str]
    modes: Mapping[str, int]
    platform_identity: str

    def __post_init__(self) -> None:
        if set(self.hashes) != set(self.modes):
            raise ValueError("boundary hash/mode asset sets differ")
        object.__setattr__(self, "hashes", MappingProxyType(dict(self.hashes)))
        object.__setattr__(self, "modes", MappingProxyType(dict(self.modes)))

    def verify(self) -> None:
        if platform.platform() != self.platform_identity:
            raise ValueError("native platform identity changed")
        for name, expected in self.hashes.items():
            path = Path(name)
            if (any(part.is_symlink() for part in (path, *path.parents)) or sha256_file(path) != expected or
                path.stat().st_mode & 0o777 != self.modes[name]):
                raise ValueError("native boundary asset changed")
        if not os.access(self.launcher, os.X_OK) or not self.registry.is_dir():
            raise ValueError("native launcher/registry unavailable")

    def parent_environment(self, runner: LocalClaudeRunner) -> dict[str, str]:
        # The CLI writes its own PATH into the generated shell snapshot. Parent
        # and child must agree or sourcing that snapshot silently removes md.
        env = {"PATH": str(self.workspace / "bin"), "SHELL": "/bin/bash", "LC_ALL": "C",
            "TMPDIR": str(self.workspace / "scratch"), "CLAUDE_CODE_TMPDIR": str(self.workspace / "scratch"),
            "CLAUDE_CONFIG_DIR": str(self.workspace / "control/config"), "CLAUDE_CODE_SHELL": str(self.launcher),
            "ANTHROPIC_BASE_URL": runner.endpoint, "ANTHROPIC_API_KEY": "synthetic-local-only",
            "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC": "1", "DISABLE_AUTOUPDATER": "1"}
        if runner.thinking_policy == "disabled":
            env["MAX_THINKING_TOKENS"] = "0"
        return env


def prepare_native_boundary(workspace: Path, condition: ConditionPin | None,
                            *, toolkit: dict[str, str]) -> NativeBoundary:
    """Create controller-private launcher/config and stage only the selected md.

The standalone Python file is a separate boundary because it must run before
the Bash sandbox without importing agent-writable or third-party Python code.
"""
    if platform.system() != "Darwin":
        raise ValueError("native containment requires macOS")
    workspace = workspace.resolve(strict=True)
    if workspace.stat().st_mode & 0o077 or workspace.stat().st_uid != os.getuid():
        raise ValueError("native workspace must be private and controller-owned")
    for name in ("fixtures", "scratch"):
        target = workspace / name
        if target.is_symlink() or not target.is_dir():
            raise ValueError("native workspace requires private fixture/scratch directories")
    if toolkit.get("bash") != "/bin/bash" or toolkit.get("sh") != "/bin/sh":
        raise ValueError("native launcher requires verified system Bash/sh")
    stage_condition(condition, workspace / "bin", toolkit=toolkit)
    control = workspace / "control"
    control.mkdir(mode=0o700)
    for name in ("registry", "config", "config/shell-snapshots"):
        (control / name).mkdir(mode=0o700)
    _write_pin_file(control / "registry/.lock", b"")
    source = Path(__file__).with_name("claude_shell.py")
    launcher = control / "bash-eval-launcher"
    _write_pin_file(launcher, source.read_bytes())
    launcher.chmod(0o500)
    profile = control / "profile.sb"
    _write_pin_file(profile, native_profile(workspace, toolkit).encode())
    child_env = {"PATH": str(workspace / "bin"), "SHELL": "/bin/bash", "LC_ALL": "C",
                 "TMPDIR": str(workspace / "scratch"), "CLAUDE_CODE_TMPDIR": str(workspace / "scratch")}
    config = control / "shell.json"
    _write_pin_file(config, canonical_json({"profile": str(profile), "profile_sha256": sha256_file(profile),
        "registry": str(control / "registry"), "bash": "/bin/bash", "environment": child_env}).encode())
    assets = [launcher, profile, config, DYLD_PROFILE, Path("/usr/bin/python3"), Path("/usr/bin/sandbox-exec"),
              workspace / "bin/md", *(Path(p) for p in toolkit.values())]
    boundary = NativeBoundary(workspace, launcher, control / "registry",
        {str(path): sha256_file(path) for path in assets},
        {str(path): path.stat().st_mode & 0o777 for path in assets}, platform.platform())
    boundary.verify()
    _write_pin_file(control / "boundary.json", canonical_json({"hashes": dict(boundary.hashes), "modes": dict(boundary.modes),
        "platform_identity": boundary.platform_identity, "profile_template_sha256":
        sha256_text(native_profile(Path("/@attempt"), toolkit))}).encode())
    return boundary
