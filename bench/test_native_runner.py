from __future__ import annotations

import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

from bench.harness import (
    NoMdLeakError,
    _assert_no_md_reachable,
    _build_agent_cmd,
    _prepend_workdir_to_path,
    _requeried_from_sequence,
    native_runner_error,
    parse_agent_output,
)

ISOLATION_FLAGS = ("--disable-slash-commands", "--strict-mcp-config", "--setting-sources", "--agents")
NATIVE_MODES = ("native", "native+md", "native+md-no-md")


def _cmd(mode: str) -> list[str]:
    return _build_agent_cmd("claude", mode)


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


class NativeRunnerGuardTests(unittest.TestCase):
    """U3 (FRAC-194): native* modes are claude-cli-only; reject them on local runners."""

    def test_native_on_local_runner_is_rejected(self) -> None:
        for runner in ("oai-loop", "pi-json"):
            for mode in NATIVE_MODES:
                err = native_runner_error([mode], runner)
                self.assertIsNotNone(err, (mode, runner))
                self.assertIn("requires --runner claude-cli", err)
                self.assertIn(mode, err)

    def test_native_on_claude_cli_is_allowed(self) -> None:
        for mode in NATIVE_MODES:
            self.assertIsNone(native_runner_error([mode], "claude-cli"), mode)

    def test_posix_modes_unaffected_on_any_runner(self) -> None:
        for runner in ("oai-loop", "pi-json", "claude-cli"):
            self.assertIsNone(native_runner_error(["unix", "hybrid", "hybrid-no-md"], runner), runner)

    def test_mixed_modes_flag_the_native_one(self) -> None:
        err = native_runner_error(["unix", "native+md"], "oai-loop")
        self.assertIsNotNone(err)
        self.assertIn("native+md", err)


def _transcript(tool_uses: list[tuple[str, dict]]) -> str:
    blocks = [{"type": "tool_use", "name": n, "input": i} for n, i in tool_uses]
    blocks.append({"type": "text", "text": "done"})
    return json.dumps([
        {"type": "assistant", "message": {"content": blocks}},
        {"type": "result", "num_turns": len(tool_uses), "usage": {"output_tokens": 1}, "result": "ok"},
    ])


class NativeToolAdoptionTests(unittest.TestCase):
    """U4 (FRAC-194): native Read/Edit/Write calls bypass the Bash guard; parse them
    from the claude-cli transcript so the native-vs-md choice is observable."""

    def test_transcript_mix_classifies_native_and_bash(self) -> None:
        # The Bash guard can't see claude-cli's Bash, so the transcript parse counts
        # BOTH native Read/Edit/Write AND classified Bash verbs — the full per-cell
        # native-vs-md adoption signal.
        parsed = parse_agent_output(_transcript([
            ("Read", {}), ("Edit", {}), ("Edit", {}),
            ("Bash", {"command": "md outline x.md"}),                       # query verb
            ("Bash", {"command": "md set-task 9.3 x.md -i --status done"}), # mutation verb
        ]))
        self.assertEqual(parsed.transcript_tool_mix,
                         {"Read": 1, "Edit": 2, "md outline": 1, "md set-task": 1})
        # FRAC-194 #4: native Edit/Write are mutations too (not just Bash mutators) —
        # 2 Edits + 1 `md set-task -i` = 3, not 1.
        self.assertEqual(parsed.transcript_mutations, 3)
        # ordered trajectory: Read=query, Edit×2=mutation, md outline=query, set-task=mutation
        self.assertEqual(parsed.transcript_call_sequence,
                         ["query", "mutation", "mutation", "query", "mutation"])
        self.assertEqual(parsed.tool_calls, 5)             # tool_calls counts ALL tool_use

    def test_transcript_classifies_sed_mutation(self) -> None:
        parsed = parse_agent_output(_transcript([("Bash", {"command": "sed -i s/a/b/ x"})]))
        self.assertEqual(parsed.transcript_tool_mix, {"sed": 1})
        self.assertEqual(parsed.transcript_mutations, 1)
        self.assertEqual(parsed.transcript_call_sequence, ["mutation"])

    def test_transcript_requery_trajectory_native_edit(self) -> None:
        # FRAC-194 #4: a native Read AFTER a native Edit is a requery — the ordered
        # sequence must capture it (the run-level scan turns query-after-mutation into
        # requeried=True). Before the fix, native Edit produced no sequence entry, so
        # claude-cli requery_rate was always 0.
        parsed = parse_agent_output(_transcript([("Read", {}), ("Edit", {}), ("Read", {})]))
        self.assertEqual(parsed.transcript_call_sequence, ["query", "mutation", "query"])
        self.assertEqual(parsed.transcript_mutations, 1)

    def test_malformed_transcript_does_not_crash(self) -> None:
        parsed = parse_agent_output("not json")
        self.assertEqual(parsed.transcript_tool_mix, {})
        self.assertEqual(parsed.transcript_mutations, 0)


class RequerySequenceTests(unittest.TestCase):
    """FRAC-194 #4: requery = a query AFTER a mutation. Pinned at the pure-logic seam
    (run_agent folds the transcript sequence into call_sequence, then calls this)."""

    def test_query_after_mutation_is_requery(self) -> None:
        self.assertTrue(_requeried_from_sequence(["query", "mutation", "query"]))
        self.assertTrue(_requeried_from_sequence(["mutation", "query"]))

    def test_no_query_after_mutation_is_not_requery(self) -> None:
        self.assertFalse(_requeried_from_sequence(["query", "mutation"]))   # query BEFORE only
        self.assertFalse(_requeried_from_sequence(["query", "query"]))
        self.assertFalse(_requeried_from_sequence(["mutation", "mutation"]))
        self.assertFalse(_requeried_from_sequence([]))

    def test_parsed_native_transcript_feeds_requery(self) -> None:
        # end-to-end seam: a native Edit then a re-Read is a requery once the parsed
        # transcript sequence reaches _requeried_from_sequence (what run_agent does).
        seq = parse_agent_output(_transcript([("Read", {}), ("Edit", {}), ("Read", {})])).transcript_call_sequence
        self.assertTrue(_requeried_from_sequence(seq))


class NativeArmPathBypassTests(unittest.TestCase):
    """FRAC-194 #8 — the PATH-axis recurrence of the ./md-bypass family. The native arm
    runs on claude-cli, whose Bash never sources BASH_ENV, so the guard's PATH
    restriction never fires and bare `md` (the form NATIVE_MD_DOCS advertises) escapes to
    the real md on the system PATH. _prepend_workdir_to_path makes bare `md` resolve to
    the workdir copy (the stub for md-free/ablation modes) instead."""

    def _stub_workdir(self) -> tuple[str, Path]:
        tmp = tempfile.TemporaryDirectory(prefix="bench_path_bypass_")
        self.addCleanup(tmp.cleanup)
        workdir = tmp.name
        stub = Path(workdir) / "md"                       # the ./md stub FIX #2 installs
        stub.write_text("#!/bin/sh\necho STUB_MD >&2\nexit 1\n")
        stub.chmod(0o755)
        fake_real_dir = Path(workdir) / "realbin"          # stands in for ~/.local/bin
        fake_real_dir.mkdir()
        fake_real = fake_real_dir / "md"
        fake_real.write_text("#!/bin/sh\necho REAL_MD\nexit 0\n")
        fake_real.chmod(0o755)
        return workdir, fake_real_dir

    def test_prepend_makes_bare_md_resolve_to_workdir_stub(self) -> None:
        workdir, fake_real_dir = self._stub_workdir()
        # claude-cli's view: full PATH with the real md, BASH_ENV unset, no guard
        child_env = {"PATH": str(fake_real_dir) + os.pathsep + "/usr/bin:/bin"}
        _prepend_workdir_to_path(child_env, workdir)
        self.assertTrue(child_env["PATH"].startswith(os.path.abspath(workdir) + os.pathsep))
        proc = subprocess.run(["/bin/sh", "-c", "md"], cwd=workdir, env=child_env,
                              text=True, capture_output=True)
        self.assertEqual(proc.returncode, 1)            # the stub exits 1
        self.assertIn("STUB_MD", proc.stderr)
        self.assertNotIn("REAL_MD", proc.stdout)        # the real md never ran

    def test_without_prepend_bare_md_escapes_to_real(self) -> None:
        # the bug, pinned hermetically (fake 'real' md, not the system one): WITHOUT the
        # prepend, bare `md` resolves to the real binary on PATH — exactly the bypass.
        workdir, fake_real_dir = self._stub_workdir()
        child_env = {"PATH": str(fake_real_dir) + os.pathsep + "/usr/bin:/bin"}
        proc = subprocess.run(["/bin/sh", "-c", "md"], cwd=workdir, env=child_env,
                              text=True, capture_output=True)
        self.assertEqual(proc.returncode, 0)
        self.assertIn("REAL_MD", proc.stdout)           # demonstrates the prepend is load-bearing


class NoMdPreflightTests(unittest.TestCase):
    """FRAC-194 #8 hardening: the fail-closed preflight proves md is unreachable for a
    no-md/ablation mode BEFORE the billed agent runs — the durable guard against the
    bypass family recurring on a future axis."""

    def _stub_workdir(self) -> str:
        tmp = tempfile.TemporaryDirectory(prefix="bench_preflight_")
        self.addCleanup(tmp.cleanup)
        workdir = tmp.name
        stub = Path(workdir) / "md"
        # mirror the real ablation stub: probe to BENCH_MD_PROBE_LOG, exit non-zero
        stub.write_text("#!/bin/sh\nprintf 'probe\\n' >> \"${BENCH_MD_PROBE_LOG:-/dev/null}\"\n"
                        "echo 'md: unavailable here' >&2\nexit 1\n")
        stub.chmod(0o755)
        return workdir

    def test_preflight_passes_when_only_stub_is_reachable(self) -> None:
        workdir = self._stub_workdir()
        probe_log = Path(workdir) / ".md-probe.log"
        child_env = {"PATH": "/usr/bin:/bin", "BENCH_MD_PROBE_LOG": str(probe_log)}
        _prepend_workdir_to_path(child_env, workdir)
        _assert_no_md_reachable(child_env, workdir)            # must NOT raise
        # the proof calls go to /dev/null, so md_probe_count is not inflated
        self.assertFalse(probe_log.exists(), "preflight must not write the real probe log")

    def test_preflight_raises_when_real_md_is_reachable(self) -> None:
        workdir = self._stub_workdir()
        fake_real_dir = Path(workdir) / "realbin"
        fake_real_dir.mkdir()
        (fake_real_dir / "md").write_text("#!/bin/sh\necho 'md 1.2.3'\nexit 0\n")
        (fake_real_dir / "md").chmod(0o755)
        # the bypass: real md on PATH, workdir NOT prepended → bare `md` answers
        child_env = {"PATH": str(fake_real_dir) + os.pathsep + "/usr/bin:/bin"}
        with self.assertRaises(NoMdLeakError):
            _assert_no_md_reachable(child_env, workdir)


if __name__ == "__main__":
    unittest.main()
