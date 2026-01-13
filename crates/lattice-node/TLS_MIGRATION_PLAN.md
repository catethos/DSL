# TLS Migration Plan: OpenSSL → rustls

## Goal

Migrate the `lattice` crate's TLS stack from OpenSSL (via `native-tls`) to `rustls` (pure Rust) to enable clean cross-compilation from macOS to Linux using `cargo-zigbuild`.

## Current State

**Source of OpenSSL dependency:** `reqwest` in `crates/lattice/Cargo.toml` (line 16)

```toml
reqwest = { version = "0.12", features = ["json", "stream"] }
```

By default, `reqwest` uses `native-tls`, which links to:
- OpenSSL on Linux
- Security.framework on macOS
- SChannel on Windows

This causes cross-compilation failures because `openssl-sys` requires Linux headers when targeting `x86_64-unknown-linux-gnu` from macOS.

## Migration Steps

### Step 1: Update reqwest Configuration

**File:** `crates/lattice/Cargo.toml`

```toml
# Before (line 16):
reqwest = { version = "0.12", features = ["json", "stream"] }

# After:
reqwest = { version = "0.12", default-features = false, features = ["json", "stream", "rustls-tls"] }
```

**Why:** 
- `default-features = false` disables `native-tls` (OpenSSL)
- `rustls-tls` enables pure-Rust TLS via rustls + ring
- No system TLS libraries required

### Step 2: Verify Build

```bash
# Native build
cargo build --release --features sql -p lattice

# Cross-compile (should now work without OpenSSL errors)
cargo zigbuild --release --features sql --target x86_64-unknown-linux-gnu -p lattice-node
```

### Step 3: Run Tests

```bash
cargo test -p lattice
cargo test -p lattice-node
```

Verify HTTP/TLS functionality works correctly with rustls backend.

### Step 4: Update Documentation

Remove or update `CROSS_COMPILE_ISSUE.md` to reflect that the OpenSSL issue is resolved.

## Other Dependencies

| Crate | TLS Impact | Notes |
|-------|------------|-------|
| `duckdb` | None | Uses `bundled` feature, compiles own C code, no TLS |
| `tokio` | None | Async runtime, no TLS |
| `serde_json` | None | Serialization only |

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| rustls only supports TLS 1.2+ | Legacy servers may fail | Most modern APIs require TLS 1.2+ anyway |
| Corporate proxies with unusual certs | Connection failures | Can add custom root certs if needed |
| ring build issues on exotic platforms | Build failures | ring has broad platform support; fallback to `aws-lc-rs` backend if needed |

## Rollback Plan

If issues arise, revert to OpenSSL with vendored build:

```toml
# Cargo.toml
reqwest = { version = "0.12", features = ["json", "stream", "native-tls-vendored"] }
```

This statically compiles OpenSSL, avoiding the cross-compilation header issue while keeping the native-tls backend.

## Success Criteria

- [ ] `cargo zigbuild --target x86_64-unknown-linux-gnu` succeeds without OpenSSL errors
- [ ] All existing tests pass
- [ ] HTTP requests to LLM APIs work correctly
- [ ] No runtime TLS handshake failures

## Timeline

| Task | Estimate |
|------|----------|
| Update Cargo.toml | 5 min |
| Test native build | 5 min |
| Test cross-compilation | 10 min |
| Verify functionality | 15 min |
| **Total** | ~35 min |
