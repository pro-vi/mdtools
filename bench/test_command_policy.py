from __future__ import annotations

import subprocess
import tempfile
import unittest
from pathlib import Path

from bench.command_policy import classify_command_kind, create_restricted_shell_env, load_guard_events


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

    def test_classify_command_kind_tolerates_partial_multiline_guard_entries(self) -> None:
        # The DEBUG trap logs raw BASH_COMMAND values. Multiline awk/sed scripts
        # can contain embedded newlines, so the line-oriented guard log may expose
        # a partial command like "awk '". Classification must not crash the run.
        self.assertEqual(classify_command_kind("awk '", "awk"), "query")
        self.assertEqual(classify_command_kind("sed -i 's/a/b", "sed"), "mutation")

    def test_mdtools_mode_denies_absolute_path_md(self) -> None:
        # Regression guard for the magnum-v4-123b-4bit failure mode observed on
        # the extraction pilot: the model invoked md via an absolute workdir
        # path (e.g. "/tmp/workdir/md outline ...") and got stuck in a deny loop.
        # The guard script rejects any base command containing "/" so only
        # PATH-resolved "md" or "./md" are accepted; this test locks that in.
        tmp, workdir = self._make_workdir()
        self.addCleanup(tmp.cleanup)
        env_info = create_restricted_shell_env(workdir, "mdtools", workdir / "md")
        abs_md = workdir / "md"
        proc = subprocess.run(
            ["/bin/bash", "-lc", f"{abs_md} outline doc.md --json >/dev/null"],
            cwd=workdir,
            env=env_info.env,
            text=True,
            capture_output=True,
        )
        events = load_guard_events(env_info.guard_log_path)
        self.assertEqual(proc.returncode, -15)
        self.assertIn("denied command in mdtools mode", proc.stderr)
        self.assertEqual(len(events), 1)
        self.assertEqual(events[0].decision, "deny")
        self.assertEqual(events[0].base_command, str(abs_md))


if __name__ == "__main__":
    unittest.main()
