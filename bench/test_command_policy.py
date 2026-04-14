from __future__ import annotations

import subprocess
import tempfile
import unittest
from pathlib import Path

from bench.command_policy import create_restricted_shell_env, load_guard_events


class CommandPolicyGuardTests(unittest.TestCase):
    def _make_workdir(self) -> tuple[tempfile.TemporaryDirectory[str], Path]:
        tmp = tempfile.TemporaryDirectory(prefix="bench_guard_test_")
        workdir = Path(tmp.name)
        md = workdir / "md"
        md.write_text("#!/bin/sh\nexit 0\n")
        md.chmod(0o755)
        (workdir / "doc.md").write_text("# Title\n\nBody\n")
        return tmp, workdir

    def _run(self, mode: str, command: str) -> tuple[subprocess.CompletedProcess[str], list]:
        tmp, workdir = self._make_workdir()
        self.addCleanup(tmp.cleanup)
        env_info = create_restricted_shell_env(workdir, mode, workdir / "md")
        proc = subprocess.run(
            ["/bin/bash", "-lc", command],
            cwd=workdir,
            env=env_info.env,
            text=True,
            capture_output=True,
        )
        return proc, load_guard_events(env_info.guard_log_path)

    def test_unix_mode_denies_python(self) -> None:
        proc, events = self._run("unix", 'python3 -c "print(1)"')
        self.assertEqual(proc.returncode, -15)
        self.assertIn("denied command in unix mode", proc.stderr)
        self.assertEqual([(event.decision, event.base_command) for event in events], [("deny", "python3")])

    def test_unix_mode_denies_pipeline_stage(self) -> None:
        proc, events = self._run("unix", 'cat /dev/null | python3 -c "print(1)"')
        self.assertEqual(proc.returncode, -15)
        self.assertEqual(
            [(event.decision, event.base_command) for event in events],
            [("allow", "cat"), ("deny", "python3")],
        )

    def test_mdtools_mode_allows_md_and_denies_grep(self) -> None:
        proc, events = self._run("mdtools", "./md outline doc.md --json >/dev/null")
        self.assertEqual(proc.returncode, 0)
        self.assertEqual([(event.decision, event.base_command) for event in events], [("allow", "md")])

        proc, events = self._run("mdtools", "grep Title doc.md")
        self.assertEqual(proc.returncode, -15)
        self.assertEqual([(event.decision, event.base_command) for event in events], [("deny", "grep")])


if __name__ == "__main__":
    unittest.main()
