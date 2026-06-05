#!/usr/bin/env python3
"""Deterministic generator for the large-file-slicing entropy family.

Provenance for the de-bias guard: the fixture is MACHINE-generated, not
hand-tuned to flatter md. Emits a ~2400-line configuration reference with 60
modules. The `timeout_ms` key appears in EVERY module's `### Settings` table,
and the value 3000 recurs across ~20 modules — so a naive `grep timeout_ms` /
sed-replace is ambiguous. The editor must locate the target module (37), then
change ONE value in its Settings table (leaving that module's Examples block and
every other module untouched).

Run: python3 generate.py   # writes input.md + expected.md
"""
from pathlib import Path

HERE = Path(__file__).parent
N_MODULES = 150
TARGET_MODULE = 118
TARGET_NEW = "5000"
MODULE_NAMES = [
    "Ingestion", "Normalization", "Indexing", "Query Planner", "Executor",
    "Cache", "Replication", "Sharding", "Compaction", "Backup",
]
TIMEOUTS = ["2000", "3000", "5000"]  # n % 3 -> recurring values; 3000 is shared


def module_block(n: int) -> str:
    name = MODULE_NAMES[n % len(MODULE_NAMES)]
    timeout = TIMEOUTS[n % 3]
    pool = 8 + (n % 5) * 4
    retries = 1 + (n % 4)
    return "\n".join([
        f"## Module {n:02d} — {name} {n}",
        "",
        "### Overview",
        "",
        f"The {name} {n} subsystem handles stage {n} of the pipeline. It is "
        f"configured independently and exposes its own tuning surface. Operators "
        f"tune these values per environment; the defaults below target a mid-size "
        f"deployment and are safe to start from.",
        "",
        f"Throughput scales with the worker pool; latency is bounded by the "
        f"`timeout_ms` ceiling. See the platform guide for cross-module effects.",
        "",
        "### Settings",
        "",
        "| key | default | notes |",
        "|-----|---------|-------|",
        f"| worker_pool | {pool} | concurrent workers |",
        f"| timeout_ms | {timeout} | per-request ceiling |",
        f"| max_retries | {retries} | on transient failure |",
        f"| queue_depth | {pool * 16} | backpressure threshold |",
        f"| flush_interval_s | {2 + n % 7} | periodic flush |",
        "",
        "### Examples",
        "",
        "```yaml",
        f"{name.lower().replace(' ', '_')}_{n}:",
        f"  worker_pool: {pool}",
        f"  timeout_ms: {timeout}",
        f"  max_retries: {retries}",
        "```",
        "",
    ])


HEADER = "\n".join([
    "# Platform Configuration Reference",
    "",
    "Generated reference for all pipeline modules. Each module is tuned "
    "independently. The `timeout_ms` key appears in every module's Settings "
    "table — always scope a change to the intended module.",
    "",
    "",
])


def assemble(blocks: list[str]) -> str:
    return (HEADER + "\n".join(blocks)).rstrip() + "\n"


if __name__ == "__main__":
    blocks = [module_block(n) for n in range(1, N_MODULES + 1)]
    inp = assemble(blocks)

    # expected: in Module 37's block ONLY, bump the Settings-table timeout_ms row
    # (3000 -> 5000). The Examples yaml line stays 3000. Other modules untouched.
    tgt = blocks[TARGET_MODULE - 1]
    assert "| timeout_ms | 3000 | per-request ceiling |" in tgt, "target row missing"
    tgt_new = tgt.replace(
        "| timeout_ms | 3000 | per-request ceiling |",
        f"| timeout_ms | {TARGET_NEW} | per-request ceiling |",
        1,
    )
    blocks_exp = list(blocks)
    blocks_exp[TARGET_MODULE - 1] = tgt_new
    exp = assemble(blocks_exp)

    (HERE / "input.md").write_text(inp)
    (HERE / "expected.md").write_text(exp)
    print(f"wrote input.md ({inp.count(chr(10))} lines) + expected.md "
          f"({exp.count(chr(10))} lines); target=Module {TARGET_MODULE}")
