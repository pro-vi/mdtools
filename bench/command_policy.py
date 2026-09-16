"""CLI condition names recovered from policy source P, c8e0813.

The old eager inventory load and shell-syntax guard are deliberately omitted.
Executable staging and native containment belong to subsequent recovery units.
"""

from __future__ import annotations

from enum import Enum
from pathlib import Path
from typing import Sequence
import os


class CliCondition(str, Enum):
    NO_MD = "no-md"
    LEGACY = "legacy"
    CURRENT_COMPACT = "current-compact"


# P's complete ordinary toolkit, with jq equalized across future conditions.
UNIX_TOOLS = ("cat", "grep", "sed", "awk", "head", "tail", "wc", "tee", "mv", "cp", "mktemp")
ORDINARY_TOOLS = (*UNIX_TOOLS, "jq")


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
