# Release-version audit

Only the local mdtools package version changes from 0.4.0 to 0.4.1 in
Cargo.toml and Cargo.lock. No third-party package version changes in this
release-preparation diff. The earlier tokenizer addition remains covered by
package-update-2026-09-14.md.

The unreleased changelog names the breaking default CLI presentation and the
explicit --json migration. Rust library compatibility and the full v3 protocol
remain unchanged. This prepares a release; no tag or registry publication is
performed by the version edit.
