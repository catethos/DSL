#!/usr/bin/env bash
set -e

# Full release workflow: build, package, upload, and start server

echo "======================================"
echo "DSL REPL - Full Release Workflow"
echo "======================================"
echo ""

# Step 1: Build and package
echo "Step 1/3: Building and packaging..."
./deploy.sh

# Step 2: Upload to VPS
echo ""
echo "Step 2/3: Uploading to VPS..."
./upload-to-vps.sh

# Step 3: Start server on VPS
echo ""
echo "Step 3/3: Starting server on VPS..."

VPS_HOST="72.61.149.67"
VPS_USER="root"
SSH_KEY="~/.ssh/id_hostinger"
REMOTE_DIR="~/dsl-dist"

# Kill existing server if running
echo "Checking for existing server..."
ssh -i "$SSH_KEY" "$VPS_USER@$VPS_HOST" "pkill -f 'python3 -m http.server 8000' || true"

# Start new server in background (properly detached)
echo "Starting HTTP server..."
ssh -i "$SSH_KEY" "$VPS_USER@$VPS_HOST" "cd $REMOTE_DIR && nohup python3 -m http.server 8000 > server.log 2>&1 </dev/null & disown"

# Wait a moment for server to start
sleep 2

# Verify server is running
echo "Verifying server..."
if curl -fsSL "http://$VPS_HOST:8000/" > /dev/null 2>&1; then
  echo ""
  echo "======================================"
  echo "✓ Release complete!"
  echo "======================================"
  echo ""
  echo "Server is running at: http://$VPS_HOST:8000/"
  echo ""
  echo "Users can install with:"
  echo "  curl -fsSL http://$VPS_HOST:8000/install.sh | bash"
  echo ""
  echo "To view server logs:"
  echo "  ssh -i $SSH_KEY $VPS_USER@$VPS_HOST 'tail -f $REMOTE_DIR/server.log'"
  echo ""
  echo "To stop the server:"
  echo "  ssh -i $SSH_KEY $VPS_USER@$VPS_HOST 'pkill -f \"python3 -m http.server 8000\"'"
  echo "======================================"
else
  echo ""
  echo "Warning: Server may not be running properly."
  echo "Check logs: ssh -i $SSH_KEY $VPS_USER@$VPS_HOST 'cat $REMOTE_DIR/server.log'"
fi
