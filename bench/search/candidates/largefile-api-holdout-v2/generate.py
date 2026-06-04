#!/usr/bin/env python3
"""CORRECTED large-file-slice holdout (iteration 7) — no greppable target marker.

The first holdout (largefile-api-holdout) was CONFOUNDED (N12): it gave the
target a unique inline marker `(s18e5)`, so the agent grepped straight to it and
the locate-hazard never fired (native+md never adopted md). This version removes
ALL inline markers. The target is identifiable ONLY by structural position —
"the `users` endpoint under Service 18" — and the value (100) recurs across the
doc, faithfully reproducing the C-LF-01 hazard (greppable SECTION heading, but an
ambiguous value several lines below it, no unique anchor on the target row).

`expected` is produced by modifying the target block BY INDEX (not string match),
so the one-line diff is guaranteed without needing any inline uniqueness.

Run: python3 generate.py
"""
from pathlib import Path

HERE = Path(__file__).parent
N_SERVICES = 24
ENDPOINTS_PER = 9
TARGET_SERVICE = 18          # deep; heading "## Service 18" is greppable but the
TARGET_ENDPOINT = 5          # target ROW (max_page_size) is ~6 lines below it
TARGET_NEW = "250"
VERBS = ["GET", "POST", "PUT", "PATCH", "DELETE"]
RESOURCES = ["accounts", "orders", "invoices", "shipments", "webhooks",
             "users", "teams", "exports", "audits"]
PAGE_SIZES = ["50", "100", "200"]


def endpoint_block(s: int, e: int, page_override: str = "") -> str:
    verb = VERBS[(s + e) % len(VERBS)]
    res = RESOURCES[e % len(RESOURCES)]
    page = page_override or PAGE_SIZES[(s * 7 + e) % 3]
    rate = 60 + ((s + e) % 5) * 30
    return "\n".join([
        f"### {verb} /v2/{res}/{{id}}",
        "",
        f"Operates on the {res} resource for service {s}. Standard auth and "
        f"pagination apply. Responses are cursor-paginated.",
        "",
        "#### Parameters",
        "",
        "| name | type | default | notes |",
        "|------|------|---------|-------|",
        "| cursor | string | null | pagination cursor |",
        f"| max_page_size | integer | {page} | items per page |",
        f"| rate_limit_rpm | integer | {rate} | requests per minute |",
        "| expand | string | none | comma-separated includes |",
        "",
        "#### Responses",
        "",
        "- `200` — success",
        "- `429` — rate limited",
        "",
    ])


HEADER = "\n".join([
    "# Public API Reference (v2)",
    "",
    "Generated endpoint reference. `max_page_size` appears in every endpoint's "
    "Parameters table; the default 100 recurs across most endpoints — always "
    "scope a change to the intended service and endpoint.",
    "", "",
])


def render(pin_target_to: str, target_value: str) -> str:
    parts = []
    for s in range(1, N_SERVICES + 1):
        parts.append(f"## Service {s:02d} — Gateway {s}")
        parts.append("")
        parts.append(f"Endpoints exposed by gateway {s}. Each is independently "
                     f"rate-limited and paginated.")
        parts.append("")
        for e in range(1, ENDPOINTS_PER + 1):
            if s == TARGET_SERVICE and e == TARGET_ENDPOINT:
                parts.append(endpoint_block(s, e, page_override=target_value))
            else:
                parts.append(endpoint_block(s, e))
    return (HEADER + "\n".join(parts)).rstrip() + "\n"


if __name__ == "__main__":
    # input: target endpoint pinned to the ambiguous value 100; expected: 250.
    inp = render(pin_target_to="100", target_value="100")
    exp = render(pin_target_to="100", target_value=TARGET_NEW)
    (HERE / "input.md").write_text(inp)
    (HERE / "expected.md").write_text(exp)
    # sanity: exactly one differing line
    import difflib
    diffs = [l for l in difflib.unified_diff(inp.splitlines(), exp.splitlines(), n=0)
             if l and l[0] in "+-" and not l.startswith(("+++", "---"))]
    print(f"wrote input.md ({inp.count(chr(10))} lines) + expected.md; "
          f"changed lines: {len(diffs)} (expect 2: one - one +)")
