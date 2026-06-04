#!/usr/bin/env python3
"""Deterministic generator for the large-file-slice HOLDOUT (different surface).

Holdout for C-LF-01. Same structural hazard (a large doc where the edit site is
expensive to LOCATE among many ambiguous matches), DIFFERENT surface: a deeply
nested API reference (## Service > ### Endpoint > #### Parameters table), with a
recurring `max_page_size` value (100) across most endpoints. A family-win on the
search instance must GENERALIZE here — this is NOT a near-copy of the config doc.

Run: python3 generate.py   # writes input.md + expected.md
"""
from pathlib import Path

HERE = Path(__file__).parent
N_SERVICES = 24
ENDPOINTS_PER = 9
TARGET_SERVICE = 18
TARGET_ENDPOINT = 5          # 1-based within the service
TARGET_NEW = "250"
VERBS = ["GET", "POST", "PUT", "PATCH", "DELETE"]
RESOURCES = ["accounts", "orders", "invoices", "shipments", "webhooks",
             "users", "teams", "exports", "audits"]
PAGE_SIZES = ["50", "100", "200"]   # recurring; 100 is the shared/ambiguous one


def endpoint_block(s: int, e: int) -> str:
    verb = VERBS[(s + e) % len(VERBS)]
    res = RESOURCES[e % len(RESOURCES)]
    page = PAGE_SIZES[(s * 7 + e) % 3]
    rate = 60 + ((s + e) % 5) * 30
    return "\n".join([
        f"### {verb} /v2/{res}/{{id}}  (s{s}e{e})",
        "",
        f"Operates on the {res} resource for service {s}. Standard auth and "
        f"pagination apply. Responses are cursor-paginated.",
        "",
        "#### Parameters",
        "",
        "| name | type | default | notes |",
        "|------|------|---------|-------|",
        f"| cursor | string | null | pagination cursor |",
        f"| max_page_size | integer | {page} | items per page |",
        f"| rate_limit_rpm | integer | {rate} | requests per minute |",
        f"| expand | string | none | comma-separated includes |",
        "",
        "#### Responses",
        "",
        "- `200` — success",
        "- `429` — rate limited",
        "",
    ])


def service_block(s: int) -> list[str]:
    out = [f"## Service {s:02d} — Gateway {s}", "",
           f"Endpoints exposed by gateway {s}. Each is independently rate-limited "
           f"and paginated.", ""]
    for e in range(1, ENDPOINTS_PER + 1):
        out.append(endpoint_block(s, e))
    return out


HEADER = "\n".join([
    "# Public API Reference (v2)",
    "",
    "Generated endpoint reference. `max_page_size` appears in every endpoint's "
    "Parameters table; the default 100 recurs across most endpoints — always "
    "scope a change to the intended endpoint.",
    "", "",
])


def assemble(blocks):
    return (HEADER + "\n".join(blocks)).rstrip() + "\n"


if __name__ == "__main__":
    # build all service blocks; the target endpoint must carry default 100 so the
    # 100 -> 250 edit applies. Force the target endpoint's page size to 100.
    services = []
    target_marker = None
    for s in range(1, N_SERVICES + 1):
        sb = [f"## Service {s:02d} — Gateway {s}", "",
              f"Endpoints exposed by gateway {s}. Each is independently rate-limited "
              f"and paginated.", ""]
        for e in range(1, ENDPOINTS_PER + 1):
            blk = endpoint_block(s, e)
            if s == TARGET_SERVICE and e == TARGET_ENDPOINT:
                # pin the target endpoint's max_page_size to 100 (the ambiguous value)
                blk = blk.replace(
                    "| max_page_size | integer | 50 | items per page |",
                    "| max_page_size | integer | 100 | items per page |",
                ).replace(
                    "| max_page_size | integer | 200 | items per page |",
                    "| max_page_size | integer | 100 | items per page |",
                )
                target_marker = blk
            sb.append(blk)
        services.append("\n".join(sb))
    inp = assemble(services)
    assert "(s9e5)" in inp and target_marker is not None
    assert "| max_page_size | integer | 100 | items per page |" in target_marker

    # expected: bump ONLY the target endpoint's max_page_size row 100 -> 250.
    tgt_new = target_marker.replace(
        "| max_page_size | integer | 100 | items per page |",
        f"| max_page_size | integer | {TARGET_NEW} | items per page |",
        1,
    )
    exp = inp.replace(target_marker, tgt_new, 1)
    (HERE / "input.md").write_text(inp)
    (HERE / "expected.md").write_text(exp)
    print(f"wrote input.md ({inp.count(chr(10))} lines) + expected.md; "
          f"target = Service {TARGET_SERVICE} endpoint {TARGET_ENDPOINT} (s9e5)")
