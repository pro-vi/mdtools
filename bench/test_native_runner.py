from __future__ import annotations

import unittest

from bench.harness import _build_agent_cmd

ISOLATION_FLAGS = ("--disable-slash-commands", "--strict-mcp-config", "--setting-sources", "--agents")
NATIVE_MODES = ("native", "native+md", "native+md-no-md")


def _cmd(mode: str) -> list[str]:
    return _build_agent_cmd("claude", mode, "md")


def _toolset(cmd: list[str], flag: str) -> str:
    return cmd[cmd.index(flag) + 1]


class NativeRunnerToolExposureTests(unittest.TestCase):
    """U2 (FRAC-194): native* modes expose claude-cli's native file tools without
    relaxing the PR#10 isolation."""

    def test_native_modes_expose_read_edit_write(self) -> None:
        for mode in NATIVE_MODES:
            cmd = _cmd(mode)
            self.assertEqual(_toolset(cmd, "--tools"), "Bash Read Edit Write", mode)
            self.assertEqual(_toolset(cmd, "--allowedTools"), "Bash Read Edit Write", mode)

    def test_non_native_modes_stay_bash_only(self) -> None:
        for mode in ("unix", "mdtools", "hybrid", "hybrid-no-md"):
            cmd = _cmd(mode)
            self.assertEqual(_toolset(cmd, "--allowedTools"), "Bash", mode)
            self.assertEqual(_toolset(cmd, "--tools"), "Bash", mode)

    def test_isolation_flags_unchanged_for_native(self) -> None:
        cmd = _cmd("native")
        for flag in ISOLATION_FLAGS:
            self.assertIn(flag, cmd)
        self.assertEqual(_toolset(cmd, "--setting-sources"), "")   # no user/project/local settings
        self.assertEqual(_toolset(cmd, "--agents"), "{}")          # no custom agents

    def test_native_cmd_is_additive_only_vs_bash(self) -> None:
        # THE contamination-regression test: a native cmd must differ from a
        # bash-only cmd by NOTHING except the two toolset values. Any other flag
        # delta would mean enabling native tools changed the isolation surface.
        def mask_toolset(cmd: list[str]) -> list[str]:
            c = list(cmd)
            for flag in ("--tools", "--allowedTools"):
                c[c.index(flag) + 1] = "<TOOLSET>"
            return c

        self.assertEqual(mask_toolset(_cmd("native+md")), mask_toolset(_cmd("unix")))


if __name__ == "__main__":
    unittest.main()
