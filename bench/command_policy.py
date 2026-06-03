from __future__ import annotations

import os
import shlex
import shutil
from dataclasses import dataclass
from pathlib import Path
from typing import Literal


BenchMode = Literal[
    "unix", "mdtools", "hybrid", "hybrid-no-md",
    # native-rooted arm (claude-cli only; see harness._build_agent_cmd + FRAC-194):
    # native Read/Edit/Write are exposed as claude-cli built-ins, NOT shell commands,
    # so the shell allowlists below mirror the POSIX arm (native↔unix, native+md↔hybrid,
    # native+md-no-md↔hybrid-no-md).
    "native", "native+md", "native+md-no-md",
]

UNIX_TOOLS = ["cat", "grep", "sed", "awk", "head", "tail", "wc", "tee", "mv", "cp", "mktemp"]
MDTOOLS_TOOLS = ["md", "cat", "jq"]

QUERY_MD_COMMANDS = {
    "outline",
    "blocks",
    "block",
    "section",
    "search",
    "links",
    "frontmatter",
    "stats",
    "table",
    "tasks",
}
MUTATION_MD_COMMANDS = {
    "replace-section",
    "delete-section",
    "replace-block",
    "insert-block",
    "delete-block",
    "set",
    "set-task",
    "move-section",
}


@dataclass
class RestrictedShellEnv:
    env: dict[str, str]
    bin_dir: Path
    guard_path: Path
    guard_log_path: Path


@dataclass
class GuardEvent:
    decision: str
    base_command: str
    raw_command: str

    @property
    def kind(self) -> str | None:
        return classify_command_kind(self.raw_command, self.base_command)

    @property
    def verb(self) -> str | None:
        return classify_command_verb(self.raw_command, self.base_command)


def allowed_commands_for_mode(mode: BenchMode) -> list[str]:
    if mode == "unix":
        return UNIX_TOOLS.copy()
    if mode == "mdtools":
        return MDTOOLS_TOOLS.copy()
    if mode == "hybrid-no-md":
        # clean-ablation baseline: md is on the PATH but a SOFT STUB (counts the
        # probe, says "use unix", exits 1) — md is "allowed" so the guard doesn't
        # hard-kill; the agent falls back to unix like a competent user. Isolates
        # md's causal value (hybrid vs hybrid-no-md = md-lift). See BENCH_V2_CLEAN_ABLATION.md.
        # jq is retained (it lives in MDTOOLS_TOOLS, advertised by the shared hybrid
        # prompt) so the ONLY difference from hybrid is md-the-tool — otherwise the
        # ablation would remove md AND jq and the lift wouldn't isolate md.
        return UNIX_TOOLS + ["md", "jq"]
    # --- native-rooted arm (mirrors the POSIX triple, + native file tools via claude-cli) ---
    if mode == "native":
        # baseline: native Edit + POSIX shell, NO md — the realistic frontier
        # alternative to md. Shell set mirrors `unix`.
        return UNIX_TOOLS.copy()
    if mode == "native+md-no-md":
        # clean ablation for the native root: same shell set as native+md but md is
        # the SOFT STUB (see create_restricted_shell_env). Mirrors hybrid-no-md.
        return UNIX_TOOLS + ["md", "jq"]
    if mode == "native+md":
        # treatment: native Edit + POSIX + real md. Shell set mirrors `hybrid`.
        return sorted(set(UNIX_TOOLS) | set(MDTOOLS_TOOLS))
    return sorted(set(UNIX_TOOLS) | set(MDTOOLS_TOOLS))


def create_restricted_shell_env(
    workdir: Path,
    mode: BenchMode,
    md_binary_path: Path,
) -> RestrictedShellEnv:
    bin_dir = workdir / ".bench-bin"
    bin_dir.mkdir(exist_ok=True)

    for command in allowed_commands_for_mode(mode):
        target = bin_dir / command
        if target.exists() or target.is_symlink():
            target.unlink()

        if command == "md" and mode in ("hybrid-no-md", "native+md-no-md"):
            # clean-ablation soft stub: count the probe, tell the agent md is
            # unavailable here, exit non-zero so it falls back to unix. No hard-kill.
            target.write_text(_md_ablation_stub())
            target.chmod(0o755)
            continue

        if command == "md":
            source = md_binary_path
        else:
            found = shutil.which(command)
            if not found:
                raise FileNotFoundError(f"required benchmark command not found: {command}")
            source = Path(found)
        target.symlink_to(source)

    guard_log_path = workdir / ".bench-guard.log"
    guard_path = workdir / ".bench-guard.sh"
    guard_path.write_text(_guard_script())

    env = os.environ.copy()
    env["BASH_ENV"] = str(guard_path)
    env["BENCH_ALLOWED_COMMANDS"] = ":".join(allowed_commands_for_mode(mode))
    env["BENCH_GUARD_LOG"] = str(guard_log_path)
    env["BENCH_MODE"] = mode
    env["BENCH_RESTRICTED_PATH"] = str(bin_dir)
    env["BENCH_MD_PROBE_LOG"] = str(workdir / ".md-probe.log")

    return RestrictedShellEnv(
        env=env,
        bin_dir=bin_dir,
        guard_path=guard_path,
        guard_log_path=guard_log_path,
    )


def load_guard_events(path: Path) -> list[GuardEvent]:
    if not path.exists():
        return []

    events: list[GuardEvent] = []
    for line in path.read_text().splitlines():
        parts = line.split("\t", 2)
        if len(parts) != 3:
            continue
        decision, base_command, raw_command = parts
        events.append(
            GuardEvent(
                decision=decision,
                base_command=base_command,
                raw_command=raw_command,
            )
        )
    return events


def classify_command_kind(raw_command: str, base_command: str | None = None) -> str | None:
    if not base_command:
        base_command = extract_base_command(raw_command)
    if not base_command:
        return None

    normalized = normalize_command(base_command)
    tokens = safe_tokenize_shell(raw_command)

    if normalized == "md":
        subcommand = _extract_md_subcommand(tokens)
        if subcommand in QUERY_MD_COMMANDS:
            return "query"
        if subcommand in MUTATION_MD_COMMANDS:
            return "mutation"
        return None

    if normalized in {"cat", "grep", "awk", "head", "tail", "wc", "jq"}:
        return "query"

    if normalized == "sed":
        if "-i" in tokens or has_output_redirection(tokens):
            return "mutation"
        return "query"

    if normalized in {"tee", "mv", "cp"}:
        return "mutation"

    return None


def classify_command_verb(raw_command: str, base_command: str | None = None) -> str | None:
    """Per-call tool/verb label for tool-mix accounting.

    Finer than classify_command_kind (query/mutation), coarser than the raw
    command: returns e.g. "md outline", "md replace-block" for mdtools verbs
    and the normalized base ("sed", "grep", "jq") for unix tools. Returns None
    when no base command can be extracted. Guard `allow` events are always an
    allowed tool, so this labels which one the agent actually chose.
    """
    if not base_command:
        base_command = extract_base_command(raw_command)
    if not base_command:
        return None

    normalized = normalize_command(base_command)
    if normalized == "md":
        subcommand = _extract_md_subcommand(safe_tokenize_shell(raw_command))
        return f"md {subcommand}" if subcommand else "md"
    return normalized


def extract_base_command(raw_command: str) -> str | None:
    tokens = safe_tokenize_shell(raw_command)
    if not tokens:
        return None

    for token in tokens:
        if token in {"|", ">", ">>", "<"}:
            continue
        if token.startswith("-"):
            continue
        if "=" in token and not token.startswith("./") and "/" not in token and token.count("=") == 1:
            key, value = token.split("=", 1)
            if key and value:
                continue
        return normalize_command(token)
    return None


def has_output_redirection(tokens: list[str]) -> bool:
    return any(token in {">", ">>"} for token in tokens)


def tokenize_shell(raw_command: str) -> list[str]:
    lexer = shlex.shlex(raw_command, posix=True, punctuation_chars="|<>")
    lexer.whitespace_split = True
    return list(lexer)


def safe_tokenize_shell(raw_command: str) -> list[str]:
    try:
        return tokenize_shell(raw_command)
    except ValueError:
        return fallback_tokenize_shell(raw_command)


def fallback_tokenize_shell(raw_command: str) -> list[str]:
    append_marker = "\0APPEND_REDIRECT\0"
    text = raw_command.replace(">>", f" {append_marker} ")
    for punct in ("|", "<", ">"):
        text = text.replace(punct, f" {punct} ")
    return [">>" if token == append_marker else token for token in text.split()]


def normalize_command(token: str) -> str:
    return "md" if token == "./md" else token


def _extract_md_subcommand(tokens: list[str]) -> str | None:
    found_md = False
    for token in tokens:
        if token in {"|", ">", ">>", "<"}:
            if found_md:
                break
            continue
        if not found_md:
            if normalize_command(token) == "md":
                found_md = True
            continue
        return token
    return None


def _md_ablation_stub() -> str:
    # Soft canonical md ablation (hybrid-no-md): count the probe, tell the agent
    # md is unavailable here, exit non-zero so it falls back to unix tools.
    # NOTE: the message must NOT name "ablation"/"benchmark" — a loop-editable
    # prompt could branch on that string and behave differently in the
    # counterfactual. Keep it indistinguishable from a plain "command not found"
    # recovery hint. See BENCH_V2_CLEAN_ABLATION.md.
    return (
        "#!/bin/sh\n"
        'printf "probe\\n" >> "${BENCH_MD_PROBE_LOG:-/dev/null}"\n'
        'echo "md: unavailable here; use standard unix tools '
        '(grep/sed/awk/cat/head/tail/...) instead." >&2\n'
        "exit 1\n"
    )


def _guard_script() -> str:
    return """\
if [[ -n "${BENCH_GUARD_ACTIVE:-}" ]]; then
  return 0
fi

BENCH_GUARD_ACTIVE=1
export PATH="${BENCH_RESTRICTED_PATH:-$PATH}"
__bench_allowed_string=":${BENCH_ALLOWED_COMMANDS:-}:"

__bench_guard_log() {
  local decision="$1"
  local base="$2"
  local raw="$3"
  if [[ -n "${BENCH_GUARD_LOG:-}" ]]; then
    builtin printf '%s\\t%s\\t%s\\n' "$decision" "$base" "$raw" >> "$BENCH_GUARD_LOG"
  fi
}

__bench_extract_base() {
  local raw="$1"
  if [[ "$raw" =~ ^[[:space:]]*([^[:space:]<>|;&()]+) ]]; then
    local token="${BASH_REMATCH[1]}"
    if [[ "$token" == "./md" ]]; then
      builtin printf 'md'
    else
      builtin printf '%s' "$token"
    fi
    return 0
  fi
  return 1
}

__bench_guard() {
  if [[ "${__BENCH_GUARD_RUNNING:-0}" == "1" ]]; then
    return 0
  fi
  __BENCH_GUARD_RUNNING=1

  local raw="$BASH_COMMAND"
  if [[ "$raw" =~ ^[[:space:]]*$ || "$raw" =~ ^[[:space:]]*# ]]; then
    __BENCH_GUARD_RUNNING=0
    return 0
  fi

  local base=""
  local trimmed="${raw#"${raw%%[![:space:]]*}"}"
  if ! base="$(__bench_extract_base "$raw")"; then
    if [[ "$trimmed" == ">"* || "$trimmed" == "<"* ]]; then
      __bench_guard_log "deny" "<redir>" "$raw"
      builtin printf '[bench-guard] denied shell redirection without an allowed command in %s mode: %s\\n' "${BENCH_MODE:-unknown}" "$raw" >&2
      kill -TERM "$$"
    fi
    __BENCH_GUARD_RUNNING=0
    return 0
  fi

  if [[ "$base" == */* || "$__bench_allowed_string" != *":$base:"* ]]; then
    __bench_guard_log "deny" "$base" "$raw"
    builtin printf '[bench-guard] denied command in %s mode: %s\\n' "${BENCH_MODE:-unknown}" "$raw" >&2
    kill -TERM "$$"
  fi

  __bench_guard_log "allow" "$base" "$raw"
  __BENCH_GUARD_RUNNING=0
}

trap '__bench_guard' DEBUG
"""
