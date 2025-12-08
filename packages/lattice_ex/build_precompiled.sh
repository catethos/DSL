#!/bin/bash
# Build precompiled NIF for macOS Apple Silicon
#
# This script builds the NIF and creates the checksum file needed for
# rustler_precompiled. The output files should be uploaded to GitHub Releases.
#
# Usage:
#   ./build_precompiled.sh
#
# Output:
#   - priv/native/liblattice_nif-v{VERSION}-nif-{NIF_VERSION}-aarch64-apple-darwin.so
#   - liblattice_nif-v{VERSION}-nif-{NIF_VERSION}-aarch64-apple-darwin.so.tar.gz (for GitHub release)
#   - checksum-Elixir.Lattice.Native.exs

set -e

VERSION="0.1.6"
NIF_VERSION="2.17"  # Erlang NIF ABI version - DO NOT change this with package version
CRATE_PATH="../../crates/lattice-nif"
FILENAME="liblattice_nif-v${VERSION}-nif-${NIF_VERSION}-aarch64-apple-darwin.so"

mkdir -p priv/native

echo "Building lattice_nif v${VERSION} for aarch64-apple-darwin..."

# Set ERTS_INCLUDE_DIR for rustler_sys
export ERTS_INCLUDE_DIR=$(elixir -e 'IO.puts("#{:code.root_dir()}/erts-#{:erlang.system_info(:version)}/include")')

# Allow undefined symbols (provided by Erlang VM at runtime)
export RUSTFLAGS="-C link-arg=-undefined -C link-arg=dynamic_lookup"

cargo build --release --manifest-path="${CRATE_PATH}/Cargo.toml" --features sql

# Copy the built library
cp "../../target/release/liblattice_nif.dylib" "priv/native/${FILENAME}"

# Create tarball for GitHub release (in current directory, not priv/native)
tar -C priv/native -czvf "${FILENAME}.tar.gz" "${FILENAME}"

# Calculate checksums
SO_SHA=$(shasum -a 256 "priv/native/${FILENAME}" | cut -d' ' -f1)
TAR_SHA=$(shasum -a 256 "${FILENAME}.tar.gz" | cut -d' ' -f1)

cat > checksum-Elixir.Lattice.Native.exs << EOF
%{
  "${FILENAME}" => "sha256:${SO_SHA}",
  "${FILENAME}.tar.gz" => "sha256:${TAR_SHA}"
}
EOF

echo ""
echo "Build complete!"
echo ""
echo "Files for GitHub Release:"
echo "  ${FILENAME}.tar.gz"
echo ""
echo "Checksum file:"
cat checksum-Elixir.Lattice.Native.exs
echo ""
echo "To publish to GitHub:"
echo "  1. Create a new release with tag v${VERSION}"
echo "  2. Upload ${FILENAME}.tar.gz to the release"
echo "  3. Commit checksum-Elixir.Lattice.Native.exs to the package"
