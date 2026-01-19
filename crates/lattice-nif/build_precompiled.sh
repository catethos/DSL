#!/bin/bash
# Build precompiled NIF binaries for multiple platforms
# Uses cargo-zigbuild for Linux cross-compilation from macOS
# Output format compatible with rustler_precompiled
#
# NIF Version Compatibility:
#   NIF 2.15 = OTP 24 (Elixir 1.12+)
#   NIF 2.16 = OTP 25 (Elixir 1.14+)
#   NIF 2.17 = OTP 26+ (Elixir 1.15+)
#
# Usage:
#   ./build_precompiled.sh              # Build for current OTP's NIF version
#   ./build_precompiled.sh 2.17         # Build for specific NIF version
#   ./build_precompiled.sh all          # Build for all NIF versions (requires Docker)

set -e

VERSION="0.2.2"
CRATE="lattice-nif"
LIB_NAME="liblattice_nif"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
OUT_DIR="$SCRIPT_DIR/precompiled"

# Detect current OTP's NIF version
detect_nif_version() {
  if command -v erl &> /dev/null; then
    erl -noshell -eval 'io:format("~s", [erlang:system_info(nif_version)]), halt().'
  else
    echo "2.17"  # Default fallback
  fi
}

# NIF version to build - from argument, environment, or auto-detect
if [ -n "$1" ] && [ "$1" != "all" ]; then
  NIF_VERSION="$1"
elif [ -n "$RUSTLER_NIF_VERSION" ]; then
  NIF_VERSION="$RUSTLER_NIF_VERSION"
else
  NIF_VERSION=$(detect_nif_version)
fi

# All supported NIF versions (for 'all' mode)
ALL_NIF_VERSIONS=("2.15" "2.16" "2.17")

echo "Workspace: $WORKSPACE_ROOT"
echo "Output: $OUT_DIR"
echo "NIF Version: $NIF_VERSION"
echo ""

mkdir -p "$OUT_DIR"

# Targets to build
TARGETS=(
  "aarch64-apple-darwin"
  "x86_64-unknown-linux-gnu"
)

cd "$WORKSPACE_ROOT"

# Function to build for a specific target and NIF version
build_target() {
  local target="$1"
  local nif_ver="$2"

  echo "========================================="
  echo "Building for $target (NIF $nif_ver)"
  echo "========================================="

  if [[ "$target" == *"linux"* ]]; then
    # Use zigbuild for Linux targets (cross-compilation)
    # Target glibc 2.17 for broad compatibility (CentOS 7+)
    # Allow undefined symbols - NIF symbols are resolved at runtime by Erlang VM
    RUSTFLAGS="-C link-arg=-Wl,--allow-shlib-undefined" \
    RUSTLER_NIF_VERSION="$nif_ver" \
      cargo zigbuild --package "$CRATE" --features sql --release --target "${target}.2.17"
  elif [[ "$target" == *"apple"* ]]; then
    # macOS: use -undefined dynamic_lookup to allow NIF symbols to be resolved at runtime
    RUSTFLAGS="-C link-arg=-undefined -C link-arg=dynamic_lookup" \
    RUSTLER_NIF_VERSION="$nif_ver" \
      cargo build --package "$CRATE" --features sql --release --target "$target"
  else
    RUSTLER_NIF_VERSION="$nif_ver" \
      cargo build --package "$CRATE" --features sql --release --target "$target"
  fi

  # Determine library extension
  if [[ "$target" == *"apple"* ]]; then
    SRC_EXT="dylib"
  else
    SRC_EXT="so"
  fi

  # rustler_precompiled naming format:
  # <lib_name>-v<version>-nif-<nif_version>-<target>.<ext>
  SRC="$WORKSPACE_ROOT/target/$target/release/${LIB_NAME}.$SRC_EXT"
  DEST="$OUT_DIR/${LIB_NAME}-v${VERSION}-nif-${nif_ver}-${target}.so"

  if [[ -f "$SRC" ]]; then
    cp "$SRC" "$DEST"
    echo "Created: $DEST"
    echo "Size: $(du -h "$DEST" | cut -f1)"
  else
    echo "ERROR: Expected file not found: $SRC"
    exit 1
  fi

  # Also create tar.gz for GitHub release (rustler_precompiled expects .so.tar.gz naming)
  TARBALL="$OUT_DIR/${LIB_NAME}-v${VERSION}-nif-${nif_ver}-${target}.so.tar.gz"
  tar -czf "$TARBALL" -C "$OUT_DIR" "${LIB_NAME}-v${VERSION}-nif-${nif_ver}-${target}.so"
  echo "Created: $TARBALL"

  echo ""
}

# Build for all targets
if [ "$1" == "all" ]; then
  echo "Building for ALL NIF versions: ${ALL_NIF_VERSIONS[*]}"
  echo "WARNING: This requires the binaries to be compatible with all NIF versions."
  echo "         For true multi-NIF support, use GitHub Actions with OTP matrix."
  echo ""
  for nif_ver in "${ALL_NIF_VERSIONS[@]}"; do
    for target in "${TARGETS[@]}"; do
      build_target "$target" "$nif_ver"
    done
  done
else
  for target in "${TARGETS[@]}"; do
    build_target "$target" "$NIF_VERSION"
  done
fi

# Generate checksums for rustler_precompiled
echo "========================================="
echo "Generating checksums"
echo "========================================="

cd "$OUT_DIR"

# Create checksum file in rustler_precompiled format
CHECKSUM_FILE="$SCRIPT_DIR/elixir_package/checksum-Elixir.Lattice.Native.exs"

cat > "$CHECKSUM_FILE" << 'HEADER'
%{
HEADER

for tarball in ${LIB_NAME}-v${VERSION}-nif-*.so.tar.gz; do
  hash=$(shasum -a 256 "$tarball" | cut -d' ' -f1)
  echo "  \"$tarball\" => \"sha256:$hash\"," >> "$CHECKSUM_FILE"
done

echo "}" >> "$CHECKSUM_FILE"

echo "Checksums written to: $CHECKSUM_FILE"
cat "$CHECKSUM_FILE"

# Extract NIF versions from built artifacts and update native.ex
echo ""
echo "========================================="
echo "Updating native.ex nif_versions"
echo "========================================="

NATIVE_EX="$SCRIPT_DIR/elixir_package/lib/lattice/native.ex"
BUILT_NIF_VERSIONS=$(ls ${LIB_NAME}-v${VERSION}-nif-*.so.tar.gz 2>/dev/null | \
  sed -E 's/.*-nif-([0-9.]+)-.*/\1/' | sort -u | tr '\n' ' ')

echo "Built NIF versions: $BUILT_NIF_VERSIONS"

# Format as Elixir list
NIF_LIST=$(echo "$BUILT_NIF_VERSIONS" | xargs -n1 | sort -u | sed 's/.*/"&"/' | tr '\n' ',' | sed 's/,$//' | sed 's/,/, /g')
echo "Updating native.ex with: nif_versions: [$NIF_LIST]"

# Update the nif_versions line in native.ex
if grep -q "nif_versions:" "$NATIVE_EX"; then
  sed -i '' "s/nif_versions: \[.*\]/nif_versions: [$NIF_LIST]/" "$NATIVE_EX"
else
  echo "WARNING: nif_versions not found in native.ex. Please add manually."
fi

echo ""
echo "========================================="
echo "Build complete!"
echo "========================================="
echo ""
echo "Files to upload to GitHub release v${VERSION}:"
ls -lh "$OUT_DIR"/*.tar.gz
echo ""
echo "NIF versions supported: $BUILT_NIF_VERSIONS"
echo ""
echo "Next steps:"
echo "  1. Review changes: git diff"
echo "  2. Create GitHub release tag: v${VERSION}"
echo "  3. Upload the .tar.gz files to the release"
echo "  4. cd elixir_package && mix deps.get && mix hex.publish"
