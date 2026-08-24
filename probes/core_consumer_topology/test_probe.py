import copy
import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path

SPEC = importlib.util.spec_from_file_location("core_consumer_probe", Path(__file__).with_name("probe.py"))
assert SPEC is not None and SPEC.loader is not None
probe = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = probe
SPEC.loader.exec_module(probe)


def passing_payload():
    return {
        "schema_version": "core-consumer-topology.v1",
        "baseline_commit": probe.BASELINE_COMMIT,
        "candidate_commit": "deadbeef",
        "source_clean": True,
        "lanes": {
            "cli_preservation": {
                "verdict": "pass",
                "suite_exit": 0,
                "mutation_match": True,
                "comparisons": [
                    {"args": args, "match": True} for args in probe.READ_PARITY_CASES
                ],
                "mutation_comparisons": [
                    {"name": name, "match": True} for name in probe.MUTATION_PARITY_NAMES
                ],
            },
            "packaged_api_adoption": {
                "verdict": "pass",
                "consumer_exit": 0,
                "package_clean": True,
                "package_content_sha256": "a" * 64,
                "package_check_exit": 0,
            },
            "library_only_composition": {
                "verdict": "pass",
                "consumer_exit": 0,
                "tree_exit": 0,
                "package_clean": True,
                "cli_only_dependencies": [],
                "package_content_sha256": "a" * 64,
                "package_check_exit": 0,
            },
        },
        "overall": "foundation_validated",
    }


class RecordedEvidenceTests(unittest.TestCase):
    def test_package_content_hash_ignores_generated_vcs_identity(self):
        with tempfile.TemporaryDirectory() as raw_workspace:
            workspace = Path(raw_workspace)
            first = workspace / "first"
            second = workspace / "second"
            for root, commit in ((first, "a" * 40), (second, "b" * 40)):
                root.mkdir()
                (root / "src").mkdir()
                (root / "src/lib.rs").write_text("pub fn parse() {}\n", encoding="utf-8")
                (root / ".cargo_vcs_info.json").write_text(
                    f'{{"git":{{"sha1":"{commit}"}},"path_in_vcs":""}}\n',
                    encoding="utf-8",
                )

            self.assertEqual(
                probe.package_content_sha256(first),
                probe.package_content_sha256(second),
            )
            (second / "src/lib.rs").write_text("pub fn parse() { todo!() }\n", encoding="utf-8")
            self.assertNotEqual(
                probe.package_content_sha256(first),
                probe.package_content_sha256(second),
            )

    def test_pass_evidence_must_be_internally_consistent(self):
        probe.validate_recorded_evidence(passing_payload())
        mutations = [
            ("cli exit", lambda value: value["lanes"]["cli_preservation"].update(suite_exit=1)),
            ("comparison", lambda value: value["lanes"]["cli_preservation"]["comparisons"][0].update(match=False)),
            ("library-only exit", lambda value: value["lanes"]["library_only_composition"].update(consumer_exit=1)),
            ("CLI dependency", lambda value: value["lanes"]["library_only_composition"].update(cli_only_dependencies=["clap"])),
            ("dirty source", lambda value: value.update(source_clean=False)),
            ("package mismatch", lambda value: value["lanes"]["library_only_composition"].update(package_content_sha256="b" * 64)),
            ("truncated reads", lambda value: value["lanes"]["cli_preservation"].update(comparisons=value["lanes"]["cli_preservation"]["comparisons"][:1])),
            ("truncated mutations", lambda value: value["lanes"]["cli_preservation"].update(mutation_comparisons=value["lanes"]["cli_preservation"]["mutation_comparisons"][:1])),
        ]
        for name, mutate in mutations:
            with self.subTest(name=name):
                payload = copy.deepcopy(passing_payload())
                mutate(payload)
                with self.assertRaises(ValueError):
                    probe.validate_recorded_evidence(payload)


if __name__ == "__main__":
    unittest.main()
