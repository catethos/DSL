#!/usr/bin/env bash
set -e

# VPS Configuration
VPS_HOST="72.61.149.67"
VPS_USER="root"
SSH_KEY="~/.ssh/id_hostinger"
REMOTE_DIR="~/dsl-dist"
DIST_DIR="dist"

echo "Uploading distribution files to VPS..."

# Check if dist directory exists
if [ ! -d "$DIST_DIR" ]; then
  echo "Error: $DIST_DIR directory not found. Run ./deploy.sh first."
  exit 1
fi

# Check if there are files to upload
if [ -z "$(ls -A $DIST_DIR)" ]; then
  echo "Error: $DIST_DIR is empty. Run ./deploy.sh first."
  exit 1
fi

# Create remote directory if it doesn't exist
echo "Creating remote directory..."
ssh -i "$SSH_KEY" "$VPS_USER@$VPS_HOST" "mkdir -p $REMOTE_DIR"

# Upload files
echo "Uploading files..."
scp -i "$SSH_KEY" "$DIST_DIR"/* "$VPS_USER@$VPS_HOST:$REMOTE_DIR/"

echo ""
echo "✓ Files uploaded successfully!"
echo ""
echo "To start the server on your VPS, run:"
echo "  ssh -i $SSH_KEY $VPS_USER@$VPS_HOST"
echo "  cd $REMOTE_DIR"
echo "  python3 -m http.server 8000"
echo ""
echo "Or run it in the background:"
echo "  ssh -i $SSH_KEY $VPS_USER@$VPS_HOST 'cd $REMOTE_DIR && nohup python3 -m http.server 8000 > server.log 2>&1 &'"
echo ""
echo "Then users can install with:"
echo "  curl -fsSL http://$VPS_HOST:8000/install.sh | bash"
