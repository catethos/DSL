# DSL REPL - Build and Release Automation

# Configuration
binary_name := "dsl"
dist_dir := "dist"
vps_host := "72.61.149.67"
vps_user := "root"
ssh_key := "~/.ssh/id_hostinger"
remote_dir := "~/dsl-dist"
version_file := "VERSION"

# Default recipe (show available commands)
default:
    @just --list

# Show current version
version:
    @cat {{version_file}}

# Bump major version (X.0.0)
bump-major:
    #!/usr/bin/env bash
    set -e
    CURRENT=$(cat {{version_file}})
    MAJOR=$(echo $CURRENT | cut -d. -f1)
    NEW_VERSION="$((MAJOR + 1)).0.0"
    echo $NEW_VERSION > {{version_file}}
    echo "Version bumped: $CURRENT → $NEW_VERSION"
    just update-cargo-versions $NEW_VERSION

# Bump minor version (x.X.0)
bump-minor:
    #!/usr/bin/env bash
    set -e
    CURRENT=$(cat {{version_file}})
    MAJOR=$(echo $CURRENT | cut -d. -f1)
    MINOR=$(echo $CURRENT | cut -d. -f2)
    NEW_VERSION="$MAJOR.$((MINOR + 1)).0"
    echo $NEW_VERSION > {{version_file}}
    echo "Version bumped: $CURRENT → $NEW_VERSION"
    just update-cargo-versions $NEW_VERSION

# Bump patch version (x.x.X)
bump-patch:
    #!/usr/bin/env bash
    set -e
    CURRENT=$(cat {{version_file}})
    MAJOR=$(echo $CURRENT | cut -d. -f1)
    MINOR=$(echo $CURRENT | cut -d. -f2)
    PATCH=$(echo $CURRENT | cut -d. -f3)
    NEW_VERSION="$MAJOR.$MINOR.$((PATCH + 1))"
    echo $NEW_VERSION > {{version_file}}
    echo "Version bumped: $CURRENT → $NEW_VERSION"
    just update-cargo-versions $NEW_VERSION

# Internal: Update all Cargo.toml files with new version
update-cargo-versions VERSION:
    #!/usr/bin/env bash
    set -e
    echo "Updating Cargo.toml files to version {{VERSION}}..."
    sed -i.bak 's/^version = ".*"/version = "{{VERSION}}"/' crates/dsl-repl/Cargo.toml
    sed -i.bak 's/^version = ".*"/version = "{{VERSION}}"/' crates/dsl-core/Cargo.toml
    sed -i.bak 's/^version = ".*"/version = "{{VERSION}}"/' crates/dsl-tui/Cargo.toml
    sed -i.bak 's/^version = ".*"/version = "{{VERSION}}"/' crates/dsl-autocomplete/Cargo.toml
    rm -f crates/*/Cargo.toml.bak
    echo "✓ Cargo.toml files updated"

# Full release workflow: build, package, upload, and start server
release: build upload start-server
    @echo ""
    @echo "======================================"
    @echo "✓ Release complete!"
    @echo "======================================"
    @echo ""
    @echo "Server is running at: http://{{vps_host}}:8000/"
    @echo ""
    @echo "Users can install with:"
    @echo "  curl -fsSL http://{{vps_host}}:8000/install.sh | bash"
    @echo ""
    @echo "To view server logs:"
    @echo "  just logs"
    @echo ""
    @echo "To stop the server:"
    @echo "  just stop-server"
    @echo "======================================"

# Build and package for current architecture
build:
    @echo "======================================"
    @echo "Building and packaging {{binary_name}}..."
    @echo "======================================"
    @echo ""
    rm -rf {{dist_dir}}
    mkdir -p {{dist_dir}}
    @echo "Current architecture: $(uname -m)"
    @echo ""
    #!/usr/bin/env bash
    set -e
    VERSION=$(cat {{version_file}})
    echo "Version: $VERSION"
    echo ""
    echo "Building for current architecture..."
    cargo build --release --manifest-path crates/dsl-repl/Cargo.toml
    CURRENT_ARCH=$(uname -m)
    case $CURRENT_ARCH in
    x86_64)
    ARCH_NAME="x86_64"
    ;;
    arm64)
    ARCH_NAME="aarch64"
    ;;
    *)
    ARCH_NAME="$CURRENT_ARCH"
    ;;
    esac
    ARCHIVE="{{binary_name}}-v${VERSION}-${ARCH_NAME}-macos.tar.gz"
    echo "Packaging $ARCHIVE..."
    tar czf "{{dist_dir}}/$ARCHIVE" -C target/release "{{binary_name}}"
    echo ""
    echo "✓ Built and packaged: {{dist_dir}}/$ARCHIVE"
    cp install.sh {{dist_dir}}/
    cp {{version_file}} {{dist_dir}}/
    echo ""
    echo "Distribution files ready in: {{dist_dir}}/"

# Upload distribution files to VPS
upload:
    @echo ""
    @echo "======================================"
    @echo "Uploading to VPS..."
    @echo "======================================"
    @echo ""
    #!/usr/bin/env bash
    set -e
    if [ ! -d "{{dist_dir}}" ]; then
    echo "Error: {{dist_dir}} directory not found. Run 'just build' first."
    exit 1
    fi
    if [ -z "$(ls -A {{dist_dir}})" ]; then
    echo "Error: {{dist_dir}} is empty. Run 'just build' first."
    exit 1
    fi
    echo "Creating remote directory..."
    ssh -i {{ssh_key}} {{vps_user}}@{{vps_host}} "mkdir -p {{remote_dir}}"
    echo "Uploading files..."
    scp -i {{ssh_key}} {{dist_dir}}/* {{vps_user}}@{{vps_host}}:{{remote_dir}}/
    echo ""
    echo "✓ Files uploaded successfully!"

# Start HTTP server on VPS
start-server:
    @echo ""
    @echo "======================================"
    @echo "Starting server on VPS..."
    @echo "======================================"
    @echo ""
    @echo "Checking for existing server..."
    -ssh -i {{ssh_key}} {{vps_user}}@{{vps_host}} "pkill -f 'python3 -m http.server 8000' || true"
    @echo "Starting HTTP server..."
    ssh -i {{ssh_key}} {{vps_user}}@{{vps_host}} "cd {{remote_dir}} && nohup python3 -m http.server 8000 > server.log 2>&1 </dev/null & disown"
    @sleep 2
    @echo "Verifying server..."
    #!/usr/bin/env bash
    if curl -fsSL "http://{{vps_host}}:8000/" > /dev/null 2>&1; then
    echo "✓ Server is running!"
    else
    echo "Warning: Server may not be running properly."
    echo "Check logs with: just logs"
    fi

# Stop HTTP server on VPS
stop-server:
    @echo "Stopping server on VPS..."
    ssh -i {{ssh_key}} {{vps_user}}@{{vps_host}} "pkill -f 'python3 -m http.server 8000'"
    @echo "✓ Server stopped"

# View server logs
logs:
    ssh -i {{ssh_key}} {{vps_user}}@{{vps_host}} "tail -f {{remote_dir}}/server.log"

# Check server status
status:
    #!/usr/bin/env bash
    if curl -fsSL "http://{{vps_host}}:8000/" > /dev/null 2>&1; then
    echo "✓ Server is running at http://{{vps_host}}:8000/"
    else
    echo "✗ Server is not responding"
    exit 1
    fi

# Clean local build artifacts
clean:
    @echo "Cleaning build artifacts..."
    rm -rf {{dist_dir}}
    cargo clean --manifest-path crates/dsl-repl/Cargo.toml
    @echo "✓ Clean complete"

# SSH into VPS
ssh:
    ssh -i {{ssh_key}} {{vps_user}}@{{vps_host}}

# Update VPS system packages (Arch Linux)
update-vps:
    @echo "Updating VPS system packages..."
    ssh -i {{ssh_key}} {{vps_user}}@{{vps_host}} "pacman -Syu --noconfirm"
    @echo "✓ VPS updated successfully"

# Check VPS system info
vps-info:
    @echo "VPS System Information:"
    @echo "======================"
    ssh -i {{ssh_key}} {{vps_user}}@{{vps_host}} "uname -a && echo '' && cat /etc/os-release"

# Run tests
test:
    cargo test --manifest-path crates/dsl-repl/Cargo.toml

# Format code
fmt:
    cargo fmt --manifest-path crates/dsl-repl/Cargo.toml

# Run linter
lint:
    cargo clippy --manifest-path crates/dsl-repl/Cargo.toml

# Development build (not release)
dev-build:
    cargo build --manifest-path crates/dsl-repl/Cargo.toml
