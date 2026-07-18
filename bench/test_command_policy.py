from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path

import typing

from bench.command_policy import (
    MD_COMMANDS,
    MD_DISPLAY_COMMANDS,
    MD_PROMPT_VISIBLE_COMMANDS,
    MD_REAL_MODES,
    MUTATION_MD_COMMANDS,
    QUERY_MD_COMMANDS,
    UNIX_TOOLS,
    allowed_commands_for_mode,
    classify_command_kind,
    create_restricted_shell_env,
    load_md_inventory,
    load_guard_events,
    md_workdir_must_be_stub,
)


class MdInventoryValidationTests(unittest.TestCase):
    def _write_inventory(self, payload: object) -> Path:
        tmp = tempfile.NamedTemporaryFile(
            mode="w",
            encoding="utf-8",
            prefix="bench_md_inventory_",
            suffix=".json",
            delete=False,
        )
        self.addCleanup(lambda: Path(tmp.name).unlink(missing_ok=True))
        with tmp:
            json.dump(payload, tmp)
        return Path(tmp.name)

    def test_inventory_derives_command_sets_and_display_list(self) -> None:
        inventory = load_md_inventory()
        self.assertEqual(inventory.commands, MD_COMMANDS)
        self.assertEqual(inventory.display_commands, MD_DISPLAY_COMMANDS)
        self.assertEqual(inventory.query_commands, QUERY_MD_COMMANDS)
        self.assertEqual(inventory.mutation_commands, MUTATION_MD_COMMANDS)
        self.assertIn("move-section", inventory.mutation_commands)
        self.assertNotIn("move-section", inventory.query_commands)

    def test_full_inventory_is_distinct_from_the_prompt_visible_subset(self) -> None:
        # `schema` is metadata tooling: it stays in the FULL inventory (so an
        # agent's `md schema` still classifies as a query, never unknown) but is
        # NOT prompt-visible, so it is neither advertised nor required in prompts.
        inventory = load_md_inventory()
        self.assertIn("schema", inventory.commands)
        self.assertIn("schema", inventory.query_commands)
        self.assertNotIn("schema", inventory.prompt_visible)
        self.assertNotIn("md schema", inventory.display_commands)
        # Every prompt-visible command is a real inventory command.
        self.assertTrue(set(inventory.prompt_visible).issubset(set(inventory.commands)))
        # classification still recognizes `md schema` as a query.
        self.assertEqual(classify_command_kind("md schema --json"), "query")

    def test_prompt_visible_defaults_true_when_omitted(self) -> None:
        path = self._write_inventory(
            {"schema_version": 1, "commands": [{"name": "outline", "kind": "query"}]}
        )
        inventory = load_md_inventory(path)
        self.assertIn("outline", inventory.prompt_visible)
        self.assertEqual(inventory.display_commands, ("md outline",))

    def test_prompt_visible_false_hides_from_display_only(self) -> None:
        path = self._write_inventory(
            {
                "schema_version": 1,
                "commands": [
                    {"name": "outline", "kind": "query"},
                    {"name": "schema", "kind": "query", "prompt_visible": False},
                ],
            }
        )
        inventory = load_md_inventory(path)
        self.assertEqual(inventory.commands, ("outline", "schema"))
        self.assertIn("schema", inventory.query_commands)
        self.assertEqual(inventory.display_commands, ("md outline",))

    def test_inventory_rejects_non_boolean_prompt_visible(self) -> None:
        path = self._write_inventory(
            {
                "schema_version": 1,
                "commands": [{"name": "outline", "kind": "query", "prompt_visible": "yes"}],
            }
        )
        with self.assertRaisesRegex(ValueError, "prompt_visible"):
            load_md_inventory(path)

    def test_inventory_rejects_wrong_schema_version(self) -> None:
        path = self._write_inventory({"schema_version": 2, "commands": []})
        with self.assertRaisesRegex(ValueError, "schema_version"):
            load_md_inventory(path)

    def test_inventory_rejects_boolean_schema_version(self) -> None:
        path = self._write_inventory({"schema_version": True, "commands": []})
        with self.assertRaisesRegex(ValueError, "schema_version"):
            load_md_inventory(path)

    def test_inventory_rejects_malformed_entries(self) -> None:
        path = self._write_inventory({"schema_version": 1, "commands": ["outline"]})
        with self.assertRaisesRegex(ValueError, "entry 0 must be an object"):
            load_md_inventory(path)

    def test_inventory_rejects_empty_names(self) -> None:
        path = self._write_inventory(
            {"schema_version": 1, "commands": [{"name": "", "kind": "query"}]}
        )
        with self.assertRaisesRegex(ValueError, "empty name"):
            load_md_inventory(path)

    def test_inventory_rejects_duplicate_names(self) -> None:
        path = self._write_inventory(
            {
                "schema_version": 1,
                "commands": [
                    {"name": "outline", "kind": "query"},
                    {"name": "outline", "kind": "query"},
                ],
            }
        )
        with self.assertRaisesRegex(ValueError, "appears more than once"):
            load_md_inventory(path)

    def test_inventory_rejects_unknown_kind(self) -> None:
        path = self._write_inventory(
            {"schema_version": 1, "commands": [{"name": "outline", "kind": "read"}]}
        )
        with self.assertRaisesRegex(ValueError, "unknown kind"):
            load_md_inventory(path)

    def test_inventory_rejects_overlap_between_query_and_mutation(self) -> None:
        path = self._write_inventory(
            {
                "schema_version": 1,
                "commands": [
                    {"name": "outline", "kind": "query"},
                    {"name": "outline", "kind": "mutation"},
                ],
            }
        )
        with self.assertRaisesRegex(ValueError, "more than one class"):
            load_md_inventory(path)


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

    def test_replace_table_row_is_classified_as_a_mutation(self) -> None:
        self.assertEqual(
            classify_command_kind("md replace-table-row 3 1 doc.md -i", "md"),
            "mutation",
        )

    def test_delete_table_row_is_classified_as_a_mutation(self) -> None:
        self.assertEqual(
            classify_command_kind("md delete-table-row 3 1 doc.md -i", "md"),
            "mutation",
        )

    def test_collect_is_classified_as_a_query(self) -> None:
        self.assertEqual(
            classify_command_kind("md collect --field title docs/ -r --json", "md"),
            "query",
        )

    def test_collect_remains_a_query_subcommand_registration(self) -> None:
        self.assertIn("collect", QUERY_MD_COMMANDS)
        self.assertNotIn("collect", MUTATION_MD_COMMANDS)

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


class NativeModeRegistrationTests(unittest.TestCase):
    """U1 (FRAC-194): the native-rooted arm mirrors the POSIX triple on the shell
    side; native file tools are claude-cli built-ins (enabled in U2), not shell."""

    def test_native_allowlist_is_unix_no_md(self) -> None:
        # baseline: native Edit + POSIX shell, NO md — mirrors `unix`.
        self.assertEqual(allowed_commands_for_mode("native"), UNIX_TOOLS)
        self.assertNotIn("md", allowed_commands_for_mode("native"))

    def test_native_md_allowlist_mirrors_hybrid(self) -> None:
        self.assertEqual(allowed_commands_for_mode("native+md"), allowed_commands_for_mode("hybrid"))
        self.assertIn("md", allowed_commands_for_mode("native+md"))

    def test_native_md_no_md_allowlist_mirrors_hybrid_no_md(self) -> None:
        # md is "allowed" (so the guard doesn't hard-kill) but installed as the stub.
        self.assertEqual(
            allowed_commands_for_mode("native+md-no-md"),
            allowed_commands_for_mode("hybrid-no-md"),
        )
        self.assertIn("md", allowed_commands_for_mode("native+md-no-md"))

    def test_native_md_no_md_installs_soft_stub(self) -> None:
        # md on PATH is the soft stub: probes + exits 1, exactly like hybrid-no-md.
        tmp = tempfile.TemporaryDirectory(prefix="bench_native_stub_")
        self.addCleanup(tmp.cleanup)
        workdir = Path(tmp.name)
        real_md = workdir / "md"
        real_md.write_text("#!/bin/sh\necho REAL\nexit 0\n")
        real_md.chmod(0o755)
        (workdir / "doc.md").write_text("# Title\n\n- [ ] x\n")
        env_info = create_restricted_shell_env(workdir, "native+md-no-md", real_md)
        proc = subprocess.run(
            ["/bin/bash", "-lc", "md tasks doc.md"],        # PATH → .bench-bin/md (the stub)
            cwd=workdir, env=env_info.env, text=True, capture_output=True,
        )
        self.assertEqual(proc.returncode, 1)               # stub exits 1 (allowed → no TERM)
        self.assertNotIn("REAL", proc.stdout)              # the real md never ran
        self.assertIn("unavailable here", proc.stderr)
        self.assertTrue((workdir / ".md-probe.log").exists())  # probe counted

    def test_md_workdir_stub_predicate_covers_every_mode(self) -> None:
        # FRAC-194 review #2 + anti-bypass: the workdir ./md copy is REAL only for the
        # three real-md modes; every other mode (incl. unix + native baselines, both
        # clean ablations) must be the stub. Fail-closed is the whole point.
        from bench.harness import BenchMode
        real = {"mdtools", "hybrid", "native+md"}
        self.assertEqual(MD_REAL_MODES, frozenset(real))
        for mode in typing.get_args(BenchMode):
            expect_stub = mode not in real
            self.assertEqual(md_workdir_must_be_stub(mode), expect_stub, mode)
        # the two baselines and both ablations are stub; nothing else is real
        for mode in ("unix", "native", "hybrid-no-md", "native+md-no-md"):
            self.assertTrue(md_workdir_must_be_stub(mode), mode)

    def test_md_workdir_stub_fails_closed_for_unknown_mode(self) -> None:
        # a hypothetical newly-added mode defaults to STUB — the property that would
        # have prevented all 4 ./md-bypass recurrences.
        self.assertTrue(md_workdir_must_be_stub("native+md+something-new"))

    def test_benchmode_literals_in_sync(self) -> None:
        # The two BenchMode Literals (command_policy + harness) must stay identical.
        from bench.command_policy import BenchMode as CMode
        from bench.harness import BenchMode as HMode
        self.assertEqual(set(typing.get_args(CMode)), set(typing.get_args(HMode)))

    def test_native_md_prompt_byte_identical_treatment_and_ablation(self) -> None:
        # Anti-gaming invariant: native+md and native+md-no-md get the SAME prompt
        # (only md availability differs), so the ablation isn't distinguishable from
        # the prompt — exactly the hybrid/hybrid-no-md discipline. Load-bearing.
        from bench.harness import BenchTask, StructuralDiffPolicy, build_prompt
        task = BenchTask(
            id="C-NATIVE", description="Edit the doc.", input_files=["inputs/doc.md"],
            expected_output="expected.md", expected_artifact="file_contents",
            difficulty="intermediate",
            scorer=StructuralDiffPolicy(
                kind="normalized_text", normalize_line_endings=True,
                ignore_trailing_whitespace=True, compare_frontmatter_json=False,
                compare_heading_tree=True, compare_block_order=True,
                compare_link_destinations=False, compare_block_text=True,
            ),
        )
        wd = "/tmp/native-prompt-wd"
        self.assertEqual(
            build_prompt(task, "native+md", wd),
            build_prompt(task, "native+md-no-md", wd),
        )
        # and `native` (no md) is a DIFFERENT prompt that never advertises md
        native_prompt = build_prompt(task, "native", wd)
        self.assertNotEqual(native_prompt, build_prompt(task, "native+md", wd))
        self.assertNotIn("md outline", native_prompt)


if __name__ == "__main__":
    unittest.main()
