# Publishing Guide: simplify_baml to crates.io and simplify_baml_ex to Hex.pm

This guide walks through publishing the Rust crate to crates.io and then the Elixir NIF wrapper to Hex.pm.

## Prerequisites

1. **Crates.io account**: Sign up at https://crates.io/
2. **Cargo login**: Run `cargo login` and paste your API token
3. **Hex.pm account**: Sign up at https://hex.pm/
4. **Hex authentication**: Run `mix hex.user auth`
5. **Update metadata**: Replace placeholder values in Cargo.toml files with your actual information

## Metadata to Update

Before publishing, update these files:

### `/Users/catethos/workspace/simplify_baml/Cargo.toml`
```toml
authors = ["Your Name <your.email@example.com>"]  # Update this
repository = "https://github.com/your-username/simplify_baml"  # Update this
```

### `/Users/catethos/workspace/simplify_baml/simplify_baml_macros/Cargo.toml`
```toml
authors = ["Your Name <your.email@example.com>"]  # Update this
repository = "https://github.com/your-username/simplify_baml"  # Update this
```

### `/Users/catethos/workspace/simplify_baml_ex/mix.exs`
```elixir
@source_url "https://github.com/your-org/simplify_baml_ex"  # Update this
```

## Step-by-Step Publishing Process

### Phase 1: Publish Rust Crates to crates.io

#### 1.1. Publish `simplify_baml_macros` (dependency first)

```bash
cd /Users/catethos/workspace/simplify_baml/simplify_baml_macros

# Verify package contents
cargo package --list

# Dry run to check for issues
cargo publish --dry-run

# If successful, publish for real
cargo publish
```

**Wait 1-2 minutes** for the crate to be available on crates.io.

#### 1.2. Update main crate to use published macros

Edit `/Users/catethos/workspace/simplify_baml/Cargo.toml`:

```toml
# Change from:
simplify_baml_macros = { path = "simplify_baml_macros" }

# To:
simplify_baml_macros = "0.1.0"
```

#### 1.3. Test that it works

```bash
cd /Users/catethos/workspace/simplify_baml
cargo clean
cargo build --release
cargo test
```

#### 1.4. Publish `simplify_baml`

```bash
cd /Users/catethos/workspace/simplify_baml

# Verify package contents
cargo package --list

# Dry run
cargo publish --dry-run

# Publish
cargo publish
```

**Wait 1-2 minutes** for the crate to be available.

### Phase 2: Update Elixir NIF to Use Published Crate

#### 2.1. Update NIF Cargo.toml

Edit `/Users/catethos/workspace/simplify_baml_ex/native/simplify_baml_nif/Cargo.toml`:

```toml
[dependencies]
# Change from:
simplify_baml = { path = "../../../simplify_baml" }

# To:
simplify_baml = "0.1.0"
```

#### 2.2. Test Elixir build

```bash
cd /Users/catethos/workspace/simplify_baml_ex

# Clean previous builds
mix clean
rm -rf _build deps

# Fetch dependencies
mix deps.get

# Compile (this will download from crates.io)
mix compile

# Run tests
mix test
```

### Phase 3: Publish to Hex.pm

#### 3.1. Verify package contents

```bash
cd /Users/catethos/workspace/simplify_baml_ex

# Check what will be included
mix hex.build
```

#### 3.2. Publish to Hex

```bash
# Publish (this will prompt for confirmation)
mix hex.publish

# Or if you want to see what would be published without actually publishing:
mix hex.publish --dry-run
```

## Verification Checklist

After publishing, verify:

- [ ] `simplify_baml_macros` appears on https://crates.io/crates/simplify_baml_macros
- [ ] `simplify_baml` appears on https://crates.io/crates/simplify_baml
- [ ] `simplify_baml` appears on https://hex.pm/packages/simplify_baml
- [ ] Test installing in a new project:
  ```bash
  # Test Elixir package
  mix new test_project
  cd test_project
  # Add to mix.exs: {:simplify_baml, "~> 0.1.0"}
  mix deps.get
  mix compile
  ```

## Common Issues and Solutions

### Issue: "no matching package found"

**Cause**: Trying to use a crate version that hasn't been published yet.

**Solution**: Make sure you've published dependencies first and waited 1-2 minutes for them to become available.

### Issue: "failed to verify package tarball"

**Cause**: Missing files or incorrect file paths in package configuration.

**Solution**: 
- For Rust: Check that all files referenced exist
- For Elixir: Check the `files` list in `mix.exs` package configuration

### Issue: Rustler compilation fails

**Cause**: The NIF can't find or compile the Rust crate.

**Solution**:
```bash
cd /Users/catethos/workspace/simplify_baml_ex
mix clean
rm -rf _build
mix deps.clean rustler --build
mix compile
```

## Future Version Updates

When publishing new versions:

1. Update version in all `Cargo.toml` files
2. Update version in `mix.exs`
3. Publish in order: `simplify_baml_macros` → `simplify_baml` → `simplify_baml_ex`
4. Use semantic versioning (MAJOR.MINOR.PATCH)

### Version Compatibility

Keep versions in sync:
```toml
# simplify_baml/Cargo.toml
[package]
version = "0.2.0"
[dependencies]
simplify_baml_macros = "0.2.0"  # Match version

# simplify_baml_ex/mix.exs
@version "0.2.0"

# simplify_baml_ex/native/simplify_baml_nif/Cargo.toml
simplify_baml = "0.2.0"  # Match version
```

## Development Workflow

For local development, you can keep using path dependencies:

```toml
# Development (fast iteration)
simplify_baml = { path = "../../../simplify_baml" }

# Production (for publishing)
simplify_baml = "0.1.0"
```

Consider using a feature flag or environment variable to switch between them:

```toml
[dependencies]
simplify_baml = { version = "0.1.0", optional = true }

[target.'cfg(not(feature = "local_dev"))'.dependencies]
simplify_baml = "0.1.0"

[target.'cfg(feature = "local_dev")'.dependencies]
simplify_baml = { path = "../../../simplify_baml" }
```

Then develop with:
```bash
# Local development
mix compile --force

# Test published version
mix clean && mix compile
```

## Resources

- **Cargo Publishing**: https://doc.rust-lang.org/cargo/reference/publishing.html
- **Hex Publishing**: https://hex.pm/docs/publish
- **Rustler Guide**: https://hexdocs.pm/rustler/basics.html
- **Semantic Versioning**: https://semver.org/
