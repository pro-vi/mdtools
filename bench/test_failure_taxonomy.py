import json
import unittest
from collections import Counter
from pathlib import Path


REPO = Path(__file__).resolve().parent.parent
TAXONOMY = REPO / "bench" / "v3" / "failure_taxonomy.json"


class FailureTaxonomyTests(unittest.TestCase):
    def test_taxonomy_labels_every_failed_in_scope_trial_once(self) -> None:
        taxonomy = json.loads(TAXONOMY.read_text())
        classes = set(taxonomy["classes"])
        labels = taxonomy["labels"]
        label_keys = [
            (item["bundle"], item["mode"], item["task_id"], item["run_index"])
            for item in labels
        ]
        self.assertEqual(len(label_keys), len(set(label_keys)))
        self.assertTrue({item["class"] for item in labels}.issubset(classes))

        expected_keys = []
        for scope in taxonomy["scope"]:
            bundle = scope["bundle"]
            modes = set(scope["modes"])
            rows = json.loads((REPO / "bench" / "runs" / bundle / "results.json").read_text())
            for row in rows:
                if row.get("mode") in modes and row.get("correct") is not True:
                    expected_keys.append((bundle, row["mode"], row["task_id"], row["run_index"]))

        self.assertEqual(sorted(label_keys), sorted(expected_keys))

    def test_taxonomy_counts_and_double_label_agreement_are_consistent(self) -> None:
        taxonomy = json.loads(TAXONOMY.read_text())
        classes = taxonomy["classes"]
        labels = taxonomy["labels"]
        count_lookup = {
            (item["bundle"], item["runner"], item["model"], item["mode"]): item
            for item in taxonomy["counts"]
        }
        observed: dict[tuple[str, str, str, str], Counter[str]] = {}
        for item in labels:
            key = (item["bundle"], item["runner"], item["model"], item["mode"])
            observed.setdefault(key, Counter())[item["class"]] += 1

        self.assertEqual(set(observed), set(count_lookup))
        for key, class_counts in observed.items():
            reported = count_lookup[key]
            self.assertEqual(reported["failed_trials"], sum(class_counts.values()))
            self.assertEqual(
                reported["classes"],
                {name: class_counts.get(name, 0) for name in classes},
            )

        double = taxonomy["double_label"]
        self.assertGreaterEqual(double["agreement_rate"], double["target_agreement_rate"])
        self.assertEqual(double["disagreements"], 0)


if __name__ == "__main__":
    unittest.main()
