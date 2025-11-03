#!/usr/bin/env bash
set -e

BINARY_NAME="dsl"
DIST_DIR="dist"

echo "Building and packaging ${BINARY_NAME}..."

# Clean previous dist
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

# Detect current architecture
CURRENT_ARCH=$(uname -m)
echo "Current architecture: $CURRENT_ARCH"

# Build for current architecture
echo ""
echo "Building for current architecture..."
cargo build --release --manifest-path crates/dsl-repl/Cargo.toml

# Determine target architecture name
case $CURRENT_ARCH in
  x86_64)
    ARCH_NAME="x86_64"
    ;;
  arm64)
    ARCH_NAME="aarch64"
    ;;
  *)
    echo "Warning: Unknown architecture $CURRENT_ARCH, using as-is"
    ARCH_NAME="$CURRENT_ARCH"
    ;;
esac

# Package current architecture
ARCHIVE="${BINARY_NAME}-${ARCH_NAME}-macos.tar.gz"
echo "Packaging $ARCHIVE..."
tar czf "$DIST_DIR/$ARCHIVE" -C target/release "$BINARY_NAME"

echo ""
echo "✓ Built and packaged:"
echo "  - $DIST_DIR/$ARCHIVE"

# Optional: Build for other architecture (requires cross-compilation setup)
if [[ "$CURRENT_ARCH" == "arm64" ]]; then
  OTHER_TARGET="x86_64-apple-darwin"
  OTHER_ARCH="x86_64"
elif [[ "$CURRENT_ARCH" == "x86_64" ]]; then
  OTHER_TARGET="aarch64-apple-darwin"
  OTHER_ARCH="aarch64"
else
  OTHER_TARGET=""
fi

if [[ -n "$OTHER_TARGET" ]]; then
  echo ""
  echo "To build for the other macOS architecture ($OTHER_ARCH), run:"
  echo "  rustup target add $OTHER_TARGET"
  echo "  cargo build --release --target $OTHER_TARGET --manifest-path crates/dsl-repl/Cargo.toml"
  echo "  tar czf $DIST_DIR/${BINARY_NAME}-${OTHER_ARCH}-macos.tar.gz -C target/$OTHER_TARGET/release $BINARY_NAME"
fi

# Copy install script
cp install.sh "$DIST_DIR/"

echo ""
echo "==================================="
echo "Distribution files ready in: $DIST_DIR/"
echo ""
echo "To deploy to your VPS:"
echo ""
echo "Quick way:"
echo "  ./upload-to-vps.sh"
echo ""
echo "Manual way:"
echo "  1. Upload files:"
echo "     scp -i ~/.ssh/id_hostinger $DIST_DIR/* root@72.61.149.67:~/dsl-dist/"
echo ""
echo "  2. On your VPS, run:"
echo "     ssh -i ~/.ssh/id_hostinger root@72.61.149.67"
echo "     cd ~/dsl-dist"
echo "     python3 -m http.server 8000"
echo ""
echo "Then users can install with:"
echo "  curl -fsSL http://72.61.149.67:8000/install.sh | bash"
echo "==================================="
