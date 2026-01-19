# Lattice NIF - Elixir Bindings

Rustler NIF bindings for embedding the Lattice runtime in Elixir.

## Project Structure

```
lattice-nif/
├── src/lib.rs          # Rust NIF implementation
├── Cargo.toml          # Rust crate config
└── elixir_package/     # Elixir package for Hex
    ├── mix.exs
    ├── lib/
    │   ├── lattice.ex
    │   └── lattice/native.ex
    └── priv/native/    # Precompiled binaries go here
```

## Building Precompiled NIFs

### Prerequisites

Install the cross-compilation toolchain:

```bash
# Install Zig (used as C cross-compiler)
brew install zig

# Install cargo-zigbuild
cargo install cargo-zigbuild

# Add Rust targets
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-gnu
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin
```

### Build All Targets

Run from the workspace root:

```bash
./crates/lattice-nif/build_precompiled.sh
```

This builds for:
- `aarch64-apple-darwin` (macOS Apple Silicon)
- `x86_64-unknown-linux-gnu` (Linux x86_64)

Output goes to `crates/lattice-nif/precompiled/`.

### Build Single Target (Development)

For local development, build just your current platform:

```bash
# From workspace root
cargo build --package lattice-nif --features sql --release

# Copy to elixir package
cp target/release/liblattice_nif.dylib \
   crates/lattice-nif/elixir_package/priv/native/liblattice_nif.so
```

Note: Erlang expects `.so` extension even on macOS.

## Release Automation (Justfile)

The `elixir_package/justfile` automates the entire release process:

```bash
cd crates/lattice-nif/elixir_package

# Show available commands
just

# Check release readiness
just check

# Full release (bump version, build, tag, push, upload to GitHub)
just release 0.1.9

# Or step by step:
just bump 0.1.9      # Update version in all files
just build           # Build precompiled NIFs
just test            # Run tests
just tag             # Create git tag
just push            # Push to remote with tags
just github-release  # Upload to GitHub releases
just hex-publish     # Publish to Hex.pm
```

## Publishing to Hex

### 1. Host Precompiled Binaries

Upload the precompiled binaries to a publicly accessible URL. Options:
- GitHub Releases (recommended)
- S3 bucket
- Any HTTP server

The files should be named:
```
liblattice_nif-nif-<NIF_VERSION>-<TARGET>.<EXT>
```

Example:
```
liblattice_nif-nif-2.16-aarch64-apple-darwin.dylib
liblattice_nif-nif-2.16-x86_64-unknown-linux-gnu.so
```

### 2. Generate Checksums

```bash
cd precompiled
shasum -a 256 liblattice_nif-* > checksum-Elixir.Lattice.Native.exs
```

### 3. Update mix.exs

Ensure `mix.exs` has the required Hex metadata:

```elixir
def project do
  [
    app: :lattice,
    version: "0.1.8",
    # ... other config
    description: "Lattice DSL runtime bindings for Elixir",
    package: package(),
  ]
end

defp package do
  [
    name: "lattice",
    files: ~w(lib priv .formatter.exs mix.exs README* LICENSE* checksum-*.exs),
    licenses: ["MIT"],
    links: %{"GitHub" => "https://github.com/YOUR_USERNAME/lattice"}
  ]
end
```

### 4. Configure rustler_precompiled (Optional)

For automatic binary downloads, update `lib/lattice/native.ex`:

```elixir
defmodule Lattice.Native do
  use RustlerPrecompiled,
    otp_app: :lattice,
    crate: "lattice_nif",
    base_url: "https://github.com/USER/REPO/releases/download/v0.1.8",
    version: "0.1.8",
    targets: ~w(
      aarch64-apple-darwin
      x86_64-unknown-linux-gnu
    )
```

### 5. Publish

```bash
cd crates/lattice-nif/elixir_package

# Authenticate (first time only)
mix hex.user register
# or
mix hex.user auth

# Publish
mix hex.publish
```

## NIF Version Compatibility

Check your Erlang NIF version:

```bash
elixir -e "IO.puts :erlang.system_info(:nif_version)"
```

Common NIF versions:
- OTP 24: 2.16
- OTP 25: 2.16
- OTP 26: 2.17

## Troubleshooting

### NIF not loading

```
** (UndefinedFunctionError) function Lattice.Native.new_runtime/0 is undefined
```

Check that the binary exists and has correct name:
```bash
ls -la crates/lattice-nif/elixir_package/priv/native/
```

### Cross-compilation errors

If zigbuild fails, ensure you have the correct glibc version:

```bash
# Target older glibc for broader compatibility
cargo zigbuild --target x86_64-unknown-linux-gnu.2.17 --release
```

### Symbol errors on Linux

If you see `undefined symbol: _enif_*`, the NIF wasn't compiled as a cdylib:

```bash
# Verify crate-type in Cargo.toml
[lib]
crate-type = ["cdylib"]
```
