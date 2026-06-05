# Release Notes

Public release notes for the SDK. Each version lists its highlights and any
breaking changes downstream consumers must act on.

## v3.0.0

### Highlights

- Rewritten transport layer with HTTP/2 support.

#### Breaking Changes

- The `connect()` method now returns a coroutine; callers must `await` it. Synchronous callers should use `connect_sync()`.

## v2.4.0

### Highlights

- Pluggable serializers.

#### Breaking Changes

- None.

## v2.3.0

### Highlights

- Lazy connection pooling.

#### Breaking Changes

- None.

## Breaking Change Policy

We only introduce breaking changes in major releases and always document a
migration path. A subsection reading "- None." means the release is drop-in
compatible.

## Template

When drafting a new version, start from this skeleton:

```markdown
## vX.Y.Z

### Highlights

#### Breaking Changes
- None.
```

Fill in each subsection before publishing.
