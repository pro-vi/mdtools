from __future__ import annotations

import os
import shlex
import shutil
from dataclasses import dataclass
from pathlib import Path
from typing import Literal


BenchMode = Literal["unix", "mdtools", "hybrid"]

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


def allowed_commands_for_mode(mode: BenchMode) -> list[str]:
    if mode == "unix":
        return UNIX_TOOLS.copy()
    if mode == "mdtools":
        return MDTOOLS_TOOLS.copy()
    return sorted(set(UNIX_TOOLS) | set(MDTOOLS_TOOLS))


def create_restricted_shell_env(
    workdir: Path,
    mode: BenchMode,
    md_binary_path: Path,
) -> RestrictedShellEnv:
    bin_dir = workdir / ".bench-bin"
    bin_dir.mkdir(exist_ok=True)

    for command in allowed_commands_for_mode(mode):
        if command == "md":
            source = md_binary_path
        else:
            found = shutil.which(command)
            if not found:
                raise FileNotFoundError(f"required benchmark command not found: {command}")
            source = Path(found)

        target = bin_dir / command
        if target.exists() or target.is_symlink():
            target.unlink()
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
    tokens = tokenize_shell(raw_command)

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


def extract_base_command(raw_command: str) -> str | None:
    tokens = tokenize_shell(raw_command)
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
