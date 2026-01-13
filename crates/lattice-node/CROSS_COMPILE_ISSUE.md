# Cross-Compilation Issue: OpenSSL for Linux from macOS

## Goal

Cross-compile `lattice-node` (a Neon/Node.js native binding written in Rust) from macOS ARM64 to Linux x86_64 using `cargo-zigbuild`, so we can publish prebuilt Linux binaries to npm without requiring users to compile.

## The Problem

When running:
```bash
cargo zigbuild --release --features sql --target x86_64-unknown-linux-gnu
```

The build fails with:
```
error: failed to run custom build command for `openssl-sys v0.9.111`

Could not find directory of OpenSSL installation, and this `-sys` crate cannot
proceed without this knowledge.

$HOST = aarch64-apple-darwin
$TARGET = x86_64-unknown-linux-gnu
```

## Root Cause

The `sql` feature in `lattice-node` enables SQL support in the `lattice` crate, which depends on crates that use OpenSSL (likely for TLS/database connections). The `openssl-sys` crate is a `-sys` crate that links to the native OpenSSL library.

When cross-compiling:
1. `openssl-sys` needs OpenSSL headers and libraries for the **target** platform (Linux x86_64)
2. macOS does not have Linux OpenSSL headers installed
3. `pkg-config` cannot find Linux OpenSSL because it's configured for the host system (macOS)

This is a fundamental limitation of cross-compiling native code that links to system libraries.

## Why cargo-zigbuild Alone Can't Fix This

`cargo-zigbuild` uses Zig's cross-compilation toolchain to provide:
- A C/C++ cross-compiler
- Target-specific libc

However, it does **not** provide:
- Target-specific system libraries (like OpenSSL)
- Headers for those libraries

The OpenSSL headers and libraries must be explicitly provided for the target platform.

## Potential Solutions

### 1. Docker (Recommended for simplicity)

Build inside a Linux container where OpenSSL is available natively:

```bash
docker run --rm -v "$(pwd):/workspace" -w /workspace \
  rust:latest \
  bash -c "cargo build --release --features sql -p lattice-node"
```

**Pros**: Works reliably, matches actual Linux environment
**Cons**: Requires Docker, slower than native compilation

### 2. Vendored OpenSSL

If the dependency tree supports it, use the `openssl/vendored` feature to compile OpenSSL from source:

```bash
OPENSSL_STATIC=1 cargo zigbuild --release --features sql --target x86_64-unknown-linux-gnu
```

This requires adding `openssl = { version = "...", features = ["vendored"] }` somewhere in the dependency tree.

**Pros**: No external dependencies needed
**Cons**: Slower builds, may not work if a dependency pins non-vendored OpenSSL

### 3. Use rustls Instead of OpenSSL

If the crates support it, switch to `rustls` (pure Rust TLS) instead of OpenSSL:
- Many crates offer feature flags like `native-tls` vs `rustls-tls`
- `rustls` cross-compiles cleanly without system dependencies

**Pros**: Clean cross-compilation, no native dependencies
**Cons**: Requires changes to dependencies, not all crates support it

### 4. Install Cross-Compilation Sysroot

Set up a Linux sysroot with OpenSSL headers:

```bash
# Example: using a pre-built sysroot
export PKG_CONFIG_SYSROOT_DIR=/path/to/linux-sysroot
export PKG_CONFIG_PATH=/path/to/linux-sysroot/usr/lib/pkgconfig
```

**Pros**: Works with cargo-zigbuild
**Cons**: Complex setup, need to maintain sysroot

### 5. GitHub Actions CI

Use GitHub Actions to build on actual Linux runners:

```yaml
jobs:
  build-linux:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo build --release --features sql
```

**Pros**: Native Linux build, free for public repos
**Cons**: Requires CI setup, not local

## Recommendation

For a single developer workflow:
1. **Docker** is the most reliable approach for Linux builds
2. Consider **removing OpenSSL dependency** if possible (switch to rustls)

For automated releases:
- Use **GitHub Actions** to build on native Linux/Windows/macOS runners

## Questions for Review

1. Is OpenSSL strictly required, or can we use rustls/native-tls alternatives?
2. Is there a vendored OpenSSL option available in the dependency tree?
3. Would GitHub Actions be acceptable for building release binaries?
