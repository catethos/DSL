#!/usr/bin/env bash
set -e

# Configuration
BINARY_NAME="dsl"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
BASE_URL="${BASE_URL:-http://72.61.149.67:8000}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "Installing ${BINARY_NAME}..."

# Check if macOS
if [[ "$(uname -s)" != "Darwin" ]]; then
  echo -e "${RED}Error: This installer only supports macOS${NC}"
  echo "Detected OS: $(uname -s)"
  exit 1
fi

# Fetch the latest version
echo "Checking latest version..."
VERSION=$(curl -fsSL "${BASE_URL}/VERSION" || echo "unknown")
if [ "$VERSION" = "unknown" ]; then
  echo -e "${YELLOW}Warning: Could not fetch version info, will try to install anyway${NC}"
fi

# Check if already installed
if command -v ${BINARY_NAME} &> /dev/null; then
  CURRENT_VERSION=$(${BINARY_NAME} --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || echo "unknown")
  if [ "$CURRENT_VERSION" != "unknown" ] && [ "$VERSION" != "unknown" ]; then
    echo -e "${YELLOW}Current version: ${CURRENT_VERSION}${NC}"
    echo -e "${YELLOW}Latest version: ${VERSION}${NC}"
    if [ "$CURRENT_VERSION" = "$VERSION" ]; then
      echo -e "${GREEN}Already up to date!${NC}"
      exit 0
    fi
    echo "Updating..."
  else
    echo "Reinstalling..."
  fi
fi

# Detect architecture
ARCH=$(uname -m)
case $ARCH in
  x86_64)
    ARCH_NAME="x86_64"
    ;;
  arm64|aarch64)
    ARCH_NAME="aarch64"
    ;;
  *)
    echo -e "${RED}Unsupported architecture: $ARCH${NC}"
    exit 1
    ;;
esac

# Determine archive name (try versioned first, fallback to unversioned)
if [ "$VERSION" != "unknown" ]; then
  ARCHIVE="${BINARY_NAME}-v${VERSION}-${ARCH_NAME}-macos.tar.gz"
else
  ARCHIVE="${BINARY_NAME}-${ARCH_NAME}-macos.tar.gz"
fi

URL="${BASE_URL}/${ARCHIVE}"
echo -e "${YELLOW}Downloading from: ${URL}${NC}"

# Create temporary directory
TMP_DIR=$(mktemp -d)
cd "$TMP_DIR"

# Download
if ! curl -fSL "$URL" -o "$ARCHIVE"; then
  echo -e "${RED}Failed to download ${ARCHIVE}${NC}"
  echo "Make sure the server is running and the file exists at:"
  echo "  $URL"
  rm -rf "$TMP_DIR"
  exit 1
fi

# Extract
echo "Extracting..."
tar xzf "$ARCHIVE"

# Ensure install directory exists
if [ ! -d "$INSTALL_DIR" ]; then
  echo "Creating $INSTALL_DIR..."
  if sudo mkdir -p "$INSTALL_DIR" 2>/dev/null; then
    echo -e "${GREEN}✓ Created $INSTALL_DIR${NC}"
  else
    echo -e "${RED}Failed to create $INSTALL_DIR${NC}"
    rm -rf "$TMP_DIR"
    exit 1
  fi
fi

# Install
if [ -w "$INSTALL_DIR" ]; then
  mv "$BINARY_NAME" "$INSTALL_DIR/"
  echo -e "${GREEN}✓ Installed to $INSTALL_DIR/${BINARY_NAME}${NC}"
else
  echo "Installing to $INSTALL_DIR (requires sudo)..."
  sudo mv "$BINARY_NAME" "$INSTALL_DIR/"
  echo -e "${GREEN}✓ Installed to $INSTALL_DIR/${BINARY_NAME}${NC}"
fi

# Cleanup
cd - > /dev/null
rm -rf "$TMP_DIR"

echo ""
echo -e "${GREEN}Installation complete!${NC}"
echo "Run '${BINARY_NAME}' to get started"
