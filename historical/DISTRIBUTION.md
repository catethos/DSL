# Distribution Guide

This document explains how to build and distribute the DSL REPL TUI application.

## Quick Start

### For Users (Installation)

Install with a single command:

```bash
curl -fsSL http://72.61.149.67:8000/install.sh | bash
```

Or for safety-conscious users who want to review the script first:

```bash
curl -fsSL http://72.61.149.67:8000/install.sh -o install.sh
less install.sh  # Review the script
bash install.sh
```

### For Maintainers (Building & Deploying)

**Super Quick (One Command):**
```bash
./release.sh
```
This builds, uploads, and starts the server automatically!

**Or Step by Step:**

1. **Build and package:**
   ```bash
   ./deploy.sh
   ```

2. **Upload to your VPS:**
   ```bash
   ./upload-to-vps.sh
   ```

3. **Start the server (optional - release.sh does this):**
   ```bash
   ssh -i ~/.ssh/id_hostinger root@72.61.149.67
   cd ~/dsl-dist
   python3 -m http.server 8000
   ```

That's it! Users can now install with the curl command above.

---

## Detailed Information

### Files

- **`install.sh`** - User-facing installation script (users run via curl)
- **`deploy.sh`** - Build and package the binary for distribution
- **`upload-to-vps.sh`** - Upload packaged files to your VPS
- **`release.sh`** - Complete workflow (build + upload + start server)
- **`dist/`** - Generated directory containing packaged binaries (created by deploy.sh)

### Supported Platforms

Currently supports:
- macOS (Intel and Apple Silicon)

### Architecture Detection

The install script automatically detects:
- `x86_64` (Intel Macs) → downloads `dsl-x86_64-macos.tar.gz`
- `arm64/aarch64` (M1/M2/M3 Macs) → downloads `dsl-aarch64-macos.tar.gz`

### Installation Directory

By default, the binary is installed to `/usr/local/bin/dsl`. You can customize this:

```bash
INSTALL_DIR=~/bin curl -fsSL http://72.61.149.67:8000/install.sh | bash
```

### Building for Both Architectures

The `deploy.sh` script builds for your current architecture. To build for the other macOS architecture:

1. Add the target:
   ```bash
   # For Intel on Apple Silicon Mac:
   rustup target add x86_64-apple-darwin

   # For Apple Silicon on Intel Mac:
   rustup target add aarch64-apple-darwin
   ```

2. Build:
   ```bash
   # For Intel target:
   cargo build --release --target x86_64-apple-darwin --manifest-path crates/dsl-repl/Cargo.toml
   tar czf dist/dsl-x86_64-macos.tar.gz -C target/x86_64-apple-darwin/release dsl

   # For Apple Silicon target:
   cargo build --release --target aarch64-apple-darwin --manifest-path crates/dsl-repl/Cargo.toml
   tar czf dist/dsl-aarch64-macos.tar.gz -C target/aarch64-apple-darwin/release dsl
   ```

### VPS Setup

#### Option 1: Quick & Simple (Python)

```bash
# On your VPS
ssh -i ~/.ssh/id_hostinger root@72.61.149.67
mkdir -p ~/dsl-dist
cd ~/dsl-dist
python3 -m http.server 8000
```

**Pros:** Zero setup, works immediately
**Cons:** Not persistent, no HTTPS, requires keeping terminal open

To keep it running in the background:
```bash
nohup python3 -m http.server 8000 > server.log 2>&1 </dev/null & disown
```

Or remotely via SSH:
```bash
ssh -i ~/.ssh/id_hostinger root@72.61.149.67 "cd ~/dsl-dist && nohup python3 -m http.server 8000 > server.log 2>&1 </dev/null & disown"
```

#### Option 2: Persistent Service (Systemd)

Create `/etc/systemd/system/dsl-server.service`:

```ini
[Unit]
Description=DSL Distribution Server
After=network.target

[Service]
Type=simple
User=youruser
WorkingDirectory=/home/youruser/dsl-dist
ExecStart=/usr/bin/python3 -m http.server 8000
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl enable dsl-server
sudo systemctl start dsl-server
sudo systemctl status dsl-server
```

#### Option 3: Nginx (Production)

If you already have nginx:

```nginx
server {
    listen 80;
    server_name your-domain.com;

    location /dsl/ {
        alias /home/youruser/dsl-dist/;
        autoindex on;
    }
}
```

Then users install with:
```bash
curl -fsSL https://your-domain.com/dsl/install.sh | bash
```

### Updating

To release a new version:

1. Update version in `Cargo.toml`
2. Run `./deploy.sh` to build new packages
3. Upload to VPS (overwrites old files)
4. Done! Users will automatically get the new version

### Uninstalling

Users can uninstall with:

```bash
sudo rm /usr/local/bin/dsl
```

### Security Notes

- **HTTP vs HTTPS:** The simple Python server uses HTTP. For production, consider using nginx with HTTPS.
- **Script Review:** Encourage users to review the install script before running it.
- **Checksums:** Consider adding SHA256 checksums for verification (not currently implemented).

### Troubleshooting

**"Failed to download":**
- Verify VPS server is running: `curl http://72.61.149.67:8000/`
- Check firewall allows port 8000
- Verify files exist in `~/dsl-dist/` on the VPS
- SSH to VPS and check: `ssh -i ~/.ssh/id_hostinger root@72.61.149.67 'ls -la ~/dsl-dist/'`

**"Unsupported architecture":**
- Only macOS (Intel and Apple Silicon) is supported
- Linux and Windows users need to compile from source

**Permission denied during install:**
- The script will prompt for sudo password when installing to `/usr/local/bin`
- Or use custom directory: `INSTALL_DIR=~/bin curl ... | bash`

---

## Alternative Distribution Methods

If you want to expand distribution in the future:

1. **GitHub Releases + Actions** - Automated binary building for multiple platforms
2. **crates.io** - `cargo install dsl-repl` for Rust users
3. **Homebrew** - `brew install yourusername/tap/dsl` for Mac users
4. **Docker** - Containerized distribution

See [DISTRIBUTION-ADVANCED.md](./DISTRIBUTION-ADVANCED.md) for details (not created yet).
