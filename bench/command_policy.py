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
import os

from bench.manifest import canonical_json, sha256_file, sha256_text


class CliCondition(str, Enum):
    NO_MD = "no-md"
    LEGACY = "legacy"
    CURRENT_COMPACT = "current-compact"


# P's complete ordinary toolkit, with jq equalized across future conditions.
UNIX_TOOLS = ("cat", "grep", "sed", "awk", "head", "tail", "wc", "tee", "mv", "cp", "mktemp")
ORDINARY_TOOLS = (*UNIX_TOOLS, "jq")

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
