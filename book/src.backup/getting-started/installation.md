# Installation

This guide will walk you through installing the DSL on your system.

## Quick Install (Recommended)

The easiest way to install DSL is using our one-line installer:

```bash
curl -fsSL https://catethos.cloud:8000/install.sh | bash
```

**That's it!** The installer will:
- ✓ Detect your system architecture (x86_64 or aarch64)
- ✓ Download the pre-compiled binaries
- ✓ Install both `dsl` (terminal REPL) and `dsl-gui` (GUI interface)
- ✓ Configure your PATH automatically

After installation, you can use either interface:

**Terminal REPL:**
```bash
dsl
```

**GUI Interface:**
```bash
dsl-gui
```

Verify installation:
```bash
dsl --version
```

```admonish success title="No Rust Required!"
Pre-built binaries mean you don't need to install Rust or compile anything. Just run the installer and start using DSL immediately.
```

```admonish tip title="Two Interfaces Available"
- **dsl** - Terminal-based REPL with a polished TUI (Text User Interface)
- **dsl-gui** - Native GUI built with egui for a graphical experience

Both interfaces provide the same DSL functionality. Choose whichever you prefer!
```

### Optional: OpenAI API Key

For LLM-powered features, set your OpenAI API key:

```bash
export OPENAI_API_KEY="sk-..."
```

```admonish tip
Add the API key to your shell profile (`~/.bashrc`, `~/.zshrc`, etc.) to persist it across sessions.
```

---

## Alternative Installation Methods

### Build from Source

If you prefer to build from source or need the latest development version:

**Prerequisites:**
- **Rust** (latest stable version)

  Install Rust via rustup:
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

  After installation, ensure Rust is in your PATH:
  ```bash
  rustc --version
  cargo --version
  ```

**Build steps:**

1. **Clone the Repository**
   ```bash
   git clone https://github.com/catethos/DSL
   cd DSL
   ```

2. **Build the Project**

   For development (faster compilation):
   ```bash
   cargo build
   ```

   For production use (optimized performance):
   ```bash
   cargo build --release
   ```

   ```admonish note
   The release build takes longer to compile but runs significantly faster.
   ```

3. **Run the DSL**

   Debug build:
   ```bash
   cargo run --bin dsl
   ```

   Release build:
   ```bash
   cargo run --release --bin dsl
   ```

   Or install locally:
   ```bash
   cargo install --path crates/dsl-repl
   ```

## Verify Installation

After running the DSL, you should see the welcome banner:

```
╔═══════════════════════════════════════════════════════════╗
║   ██████╗ █████╗ ██████╗ ██╗   ██╗                    ║
║  ██╔════╝██╔══██╗██╔══██╗╚██╗ ██╔╝                    ║
║  ██║     ███████║██████╔╝ ╚████╔╝                     ║
║  ██║     ██╔══██║██╔═══╝   ╚██╔╝                      ║
║  ╚██████╗██║  ██║██║        ██║                       ║
║   ╚═════╝╚═╝  ╚═╝╚═╝        ╚═╝                       ║
║                 ███████╗██╗      ██████╗ ██╗    ██╗      ║
║                 ██╔════╝██║     ██╔═══██╗██║    ██║      ║
║                 █████╗  ██║     ██║   ██║██║ █╗ ██║      ║
║                 ██╔══╝  ██║     ██║   ██║██║███╗██║      ║
║                 ██║     ███████╗╚██████╔╝╚███╔███╔╝      ║
║                 ╚═╝     ╚══════╝ ╚═════╝  ╚══╝╚══╝       ║
║              Agentic LLM Workflow DSL                ║
╚═══════════════════════════════════════════════════════════╝

Version:  0.3.0 (Phase 2 Complete)
Phase:    3/9 phases complete (33%)
Features: Literals, Arithmetic, Variables, Binding
Commands: :vars, :help, :clear, :q
```

Try a simple expression to confirm everything works:
```
flow> 42
✓ 42 : Int
```

## Platform-Specific Notes

### macOS
No special configuration needed. Ensure you have Xcode Command Line Tools installed:
```bash
xcode-select --install
```

### Linux
You may need to install additional dependencies:

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install build-essential pkg-config libssl-dev
```

**Fedora/RHEL:**
```bash
sudo dnf install gcc pkg-config openssl-devel
```

### Windows
Install the [Visual C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) before installing Rust.

## Troubleshooting

### Issue: "rustc: command not found"
**Solution:** Ensure Rust is in your PATH. Restart your terminal or run:
```bash
source $HOME/.cargo/env
```

### Issue: Build fails with SSL errors
**Solution:** Install OpenSSL development libraries:
```bash
# macOS
brew install openssl

# Linux (Ubuntu/Debian)
sudo apt-get install libssl-dev
```

### Issue: Permission denied when running
**Solution:** Ensure the binary has execute permissions:
```bash
chmod +x target/release/dsl
```

## Next Steps

Now that you have DSL installed, continue to:
- [Quick Start](quick-start.md) - Get up and running in 5 minutes
- [First Program](first-program.md) - Write your first DSL program
- [REPL Tour](repl-tour.md) - Interactive tour of the REPL
