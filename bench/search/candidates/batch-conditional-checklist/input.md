# Deployment Readiness Checklist

Track readiness for the v4.2 production rollout. Automated phases are gated by
CI; the manual phase requires a human sign-off. Mark a task done only when its
gate has actually passed.

## Phase 1 — Build & Test (automated)

- [x] Compile release artifacts
- [ ] Run unit suite
- [ ] Run smoke tests
- [ ] Publish coverage report

## Phase 2 — Security Scan (automated)

- [ ] SAST scan
- [ ] Dependency audit
- [ ] Secret-leak scan
- [ ] License compliance check

## Phase 3 — Performance Gate (automated)

- [ ] Load test at 2x peak
- [ ] p99 latency under 200ms
- [ ] Memory ceiling check

## Phase 4 — Manual Sign-off (manual)

- [ ] Product owner approval
- [ ] Run smoke tests
- [ ] Security lead sign-off

## Notes

The CI pipeline updates the automated phases. The "- [ ] Run smoke tests" item
also appears under the manual sign-off, where a human re-runs them by hand — do
not conflate the two.
