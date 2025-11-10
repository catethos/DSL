# DSL REPL - Distribution Setup

Quick reference for distributing your TUI app.

## For Users

Install with one command:
```bash
curl -fsSL http://72.61.149.67:8000/install.sh | bash
```

## For You (Maintainer)

### Quick Release (Recommended)
```bash
./release.sh
```
This does everything: builds, uploads, and starts the server!

### Step-by-Step
```bash
# 1. Build and package
./deploy.sh

# 2. Upload to VPS
./upload-to-vps.sh

# 3. Start server (if needed)
ssh -i ~/.ssh/id_hostinger root@72.61.149.67 'cd ~/dsl-dist && python3 -m http.server 8000'
```

## Scripts Overview

| Script | What it does |
|--------|-------------|
| `release.sh` | 🚀 Full workflow: build → upload → start server |
| `deploy.sh` | 📦 Build and package binary |
| `upload-to-vps.sh` | ⬆️ Upload to VPS |
| `install.sh` | 💾 User installation script |

## Common Tasks

### Update to new version
```bash
./release.sh
```

### Check if server is running
```bash
curl http://72.61.149.67:8000/
```

### View server logs
```bash
ssh -i ~/.ssh/id_hostinger root@72.61.149.67 'tail -f ~/dsl-dist/server.log'
```

### Stop the server
```bash
ssh -i ~/.ssh/id_hostinger root@72.61.149.67 'pkill -f "python3 -m http.server 8000"'
```

### Test installation locally
```bash
curl -fsSL http://72.61.149.67:8000/install.sh | bash
```

## VPS Details

- **Host:** 72.61.149.67
- **User:** root
- **SSH Key:** ~/.ssh/id_hostinger
- **Distribution Dir:** ~/dsl-dist
- **Server Port:** 8000

## Files Structure

```
DSL/
├── install.sh           # User installation script
├── deploy.sh           # Build & package
├── upload-to-vps.sh    # Upload to server
├── release.sh          # Full workflow
└── dist/               # Generated files
    ├── dsl-*.tar.gz    # Binary packages
    └── install.sh      # Copy for distribution
```

---

For detailed documentation, see [DISTRIBUTION.md](./DISTRIBUTION.md)
