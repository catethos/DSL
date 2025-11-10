# egui Ecosystem Upgrade Plan

**Current Version:** egui 0.31
**Target Version:** egui 0.34+ (latest stable)
**Last Updated:** 2025-11-10

---

## Executive Summary

This document provides a comprehensive plan for upgrading the egui ecosystem from version 0.31 to the latest stable version. It documents compatibility issues discovered during development, version dependencies, API breaking changes, and step-by-step upgrade instructions.

**Key Takeaway:** The egui ecosystem requires **strict version alignment** across all crates. Mixing versions causes compilation errors due to type mismatches.

---

## Current State (egui 0.31)

### Dependencies

```toml
# crates/dsl-egui/Cargo.toml
eframe = "0.31"
egui = "0.31"
egui_extras = { version = "0.31", features = ["syntect", "image"] }
egui_plot = "0.31"
egui_commonmark = "0.20"
```

### Known Limitations

1. **Heading Size Hierarchy** - All markdown headings (H1-H6) render at the same font size
   - `egui_commonmark` 0.20 uses a single `TextStyle::Heading` for all levels
   - No visual hierarchy between heading levels

2. **Deprecated APIs in Use**
   - Using `egui::menu::bar()` which works but is marked deprecated
   - Using `Frame::corner_radius()` (correct for 0.31)

3. **API Compatibility**
   - `Line::new(points)` - takes 1 argument (the data points)
   - `Points::new(points)` - takes 1 argument (the data points)

---

## Version Compatibility Matrix

### Discovered Through Testing

| egui Version | egui_plot | egui_commonmark | Status | Notes |
|--------------|-----------|-----------------|--------|-------|
| **0.29** | 0.29 | 0.18 | ✅ Working | Initial implementation |
| **0.31** | 0.31 | 0.20 | ✅ **Current** | Stable, no heading hierarchy |
| **0.32** | 0.32 | 0.21 | ❌ **BROKEN** | egui_plot 0.32.0 has dependency bug |
| **0.33** | 0.33 | 0.22 | ✅ Working | Requires API changes |
| **0.34** | 0.34 | ? | ❓ Unknown | Latest - needs testing |

### Critical Bug: egui_plot 0.32.0

**Problem:**
Despite being labeled version 0.32, `egui_plot = "0.32.0"` incorrectly depends on `egui ^0.31`.

**Symptoms:**
```
error[E0308]: mismatched types
  --> crates/dsl-egui/src/renderers/chart.rs:101:15
   |
   | expected `egui::ui::Ui`, found `Ui`
   |
note: two different versions of crate `egui` are being used
```

**Root Cause:**
Cargo tree shows:
```
├── egui v0.32.3         # Your direct dependency
│   ├── egui_plot v0.32.0
│   │   ├── egui v0.31.1  # ← BUG: Should be 0.32!
```

**Solution:**
**Skip version 0.32 entirely.** Either:
- Stay on 0.31 (current approach)
- Jump directly to 0.33+

**Why This Matters:**
This creates `Color32` type conflicts because:
- Your code uses `egui 0.32` → `ecolor 0.32.3` → `Color32` v0.32
- `egui_plot` pulls `egui 0.31` → `ecolor 0.31.1` → `Color32` v0.31
- Rust sees these as **different types** even though they have the same name

**Verification Command:**
```bash
cargo tree -p dsl-egui | grep -E "egui |ecolor"
```

---

## API Breaking Changes

### 0.31 → 0.33+ Changes

#### 1. egui_plot API - Name Parameter Required

**Change:** `Line::new()` and `Points::new()` now require a name parameter.

**0.31 (Current):**
```rust
// animations.rs, chart.rs
egui_plot::Line::new(wave_points)
    .color(Color32::from_rgb(100, 150, 255))
    .width(2.0)

egui_plot::Points::new(PlotPoints::from(points))
    .color(Color32::from_rgb(200, 100, 100))
    .radius(5.0)
```

**0.33+ (New):**
```rust
egui_plot::Line::new("wave", wave_points)
    .color(Color32::from_rgb(100, 150, 255))
    .width(2.0)

egui_plot::Points::new("points", PlotPoints::from(points))
    .color(Color32::from_rgb(200, 100, 100))
    .radius(5.0)
```

**Files to Update:**
- `crates/dsl-egui/src/animations.rs` - ~11 instances
- `crates/dsl-egui/src/renderers/chart.rs` - 2 instances

**Why:** The name parameter enables better accessibility and debugging in plots.

#### 2. Frame API - Rounding Renamed

**Change:** `Frame::rounding()` renamed to `Frame::corner_radius()`

**0.31 (Current):**
```rust
Frame::default()
    .fill(egui::Color32::from_rgb(40, 30, 30))
    .rounding(4.0)  // ← Old name
```

**0.33+ (New):**
```rust
Frame::default()
    .fill(egui::Color32::from_rgb(40, 30, 30))
    .corner_radius(4.0)  // ← New name
```

**Files to Update:**
- `crates/dsl-egui/src/renderers/error.rs` - 6 instances

**Status:** Already updated in 0.31 (we use `corner_radius()`)

#### 3. Menu Bar API

**Change:** `egui::menu::bar()` deprecated in favor of `egui::MenuBar::new().ui()`

**0.31 (Current):**
```rust
egui::menu::bar(ui, |ui| {
    ui.menu_button("File", |ui| {
        // menu items
    });
});
```

**0.33+ (New):**
```rust
egui::MenuBar::new().ui(ui, |ui| {
    ui.menu_button("File", |ui| {
        // menu items
    });
});
```

**Files to Update:**
- `crates/dsl-egui/src/app.rs` - 1 instance

**Status:** Currently using deprecated API (works but shows warning in 0.31)

#### 4. Color32 Import Changes

**Issue:** When using egui_plot, you must import `Color32` from `ecolor` directly, not from `egui`.

**0.31 (Current - Fixed):**
```rust
use egui;
use egui::ecolor::Color32;  // ← Correct

// NOT: use egui::Color32;  ← Wrong, causes type conflicts
```

**Files Already Fixed:**
- `crates/dsl-egui/src/animations.rs`
- `crates/dsl-egui/src/renderers/chart.rs`

**Why:** egui_plot expects `ecolor::Color32` directly. Using `egui::Color32` can cause type mismatches across version boundaries.

---

## Benefits of Upgrading to 0.33+

### 1. Better Markdown Heading Hierarchy

**Current (0.31 + egui_commonmark 0.20):**
- All headings (H1-H6) render at the same font size
- Only styling differences (bold, color)

**After Upgrade (0.33 + egui_commonmark 0.22):**
- Proper font size hierarchy (H1 largest → H6 smallest)
- Better visual document structure

### 2. Latest Features

- Performance improvements
- Bug fixes
- New widgets and features
- Better accessibility

### 3. Active Maintenance

- Security updates
- Community support
- Up-to-date documentation

---

## Step-by-Step Upgrade Plan

### Phase 1: Pre-Upgrade Checklist

- [ ] Create a new git branch: `git checkout -b egui-upgrade-0.34`
- [ ] Document current version: `cargo tree -p dsl-egui > before-upgrade.txt`
- [ ] Run full test suite: `cargo test`
- [ ] Test GUI application manually
- [ ] Take screenshots of current UI for comparison

### Phase 2: Update Dependencies

**File:** `crates/dsl-egui/Cargo.toml`

```diff
 [dependencies]
 # egui framework
-eframe = "0.31"
-egui = "0.31"
-egui_extras = { version = "0.31", features = ["syntect", "image"] }
-egui_plot = "0.31"
-egui_commonmark = "0.20"
+eframe = "0.34"        # or latest stable
+egui = "0.34"
+egui_extras = { version = "0.34", features = ["syntect", "image"] }
+egui_plot = "0.34"
+egui_commonmark = "0.24"  # Check docs.rs for latest compatible version
```

**Action:**
```bash
# Update dependencies
cargo update -p eframe -p egui -p egui_extras -p egui_plot -p egui_commonmark

# Verify no version conflicts
cargo tree -p dsl-egui | grep -E "egui |ecolor"
```

**Expected Output:** All egui crates should show the same minor version (e.g., all 0.34.x)

### Phase 3: Fix API Breaking Changes

#### 3.1 Update egui_plot API Calls

**Files to modify:**
- `crates/dsl-egui/src/animations.rs`
- `crates/dsl-egui/src/renderers/chart.rs`

**Find and replace:**
```bash
# Find all Line::new calls
rg "Line::new\(" crates/dsl-egui/src/

# Find all Points::new calls
rg "Points::new\(" crates/dsl-egui/src/
```

**Update pattern:**
```rust
// animations.rs - line 123 (example)
- egui_plot::Line::new(wave_points)
+ egui_plot::Line::new("wave", wave_points)

// Give each line a unique, descriptive name:
- egui_plot::Line::new(points)
+ egui_plot::Line::new("lissajous", points)  // for Lissajous animation
+ egui_plot::Line::new("helix1", helix1)     // for first helix
+ egui_plot::Line::new("helix2", helix2)     // for second helix
// etc.
```

**Testing:**
```bash
cargo check -p dsl-egui 2>&1 | grep "Line::new\|Points::new"
```

Should show no errors after fixing all instances.

#### 3.2 Update Menu Bar API

**File:** `crates/dsl-egui/src/app.rs` (line ~87)

```diff
 egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
-    egui::menu::bar(ui, |ui| {
+    egui::MenuBar::new().ui(ui, |ui| {
         ui.menu_button("File", |ui| {
             // ... menu items
         });
     });
 });
```

#### 3.3 Verify Color32 Imports

Check these files still use correct imports:

```bash
rg "use.*Color32" crates/dsl-egui/src/
```

Should see:
```rust
use egui::ecolor::Color32;  // ✓ Correct
```

NOT:
```rust
use egui::Color32;  // ✗ Wrong - will cause issues
```

### Phase 4: Update Markdown Renderer

**File:** `crates/dsl-egui/src/renderers/markdown.rs`

The current workaround for heading sizes can be **removed** if egui_commonmark 0.22+ supports proper heading hierarchy.

**Check egui_commonmark 0.22+ documentation:**
```bash
# View docs for new version
cargo doc --package egui_commonmark --open
```

**Potential update:**
```rust
pub fn render_markdown(ui: &mut egui::Ui, cache: &mut CommonMarkCache, markdown: &str) {
    // If 0.22+ has native heading support, remove style customization
    CommonMarkViewer::new().show(ui, cache, markdown);
}
```

**Test:**
```rust
RenderMarkdown("# H1\n## H2\n### H3")
```

Verify headings have different sizes.

### Phase 5: Compile and Fix Remaining Issues

```bash
# Clean build
cargo clean

# Build with verbose output
cargo build -p dsl-egui 2>&1 | tee build-log.txt

# Check for errors
cargo check -p dsl-egui
```

**Common issues to watch for:**

1. **Type mismatches** - Check `cargo tree` for version conflicts
2. **Deprecated APIs** - Read compiler warnings
3. **Missing features** - Check if new versions need feature flags

### Phase 6: Testing

#### Unit Tests
```bash
cargo test -p dsl-egui
```

#### Manual Testing Checklist

- [ ] Launch GUI: `cargo run -p dsl-egui`
- [ ] Test REPL evaluation
- [ ] Test markdown rendering with different heading levels
- [ ] Test chart rendering (line, bar, scatter)
- [ ] Test animations (if any)
- [ ] Test file operations (Open, Save, Save As)
- [ ] Test menu bar
- [ ] Test keyboard shortcuts
- [ ] Test error display
- [ ] Test table rendering
- [ ] Test image rendering

#### Visual Regression Testing

Compare screenshots:
- Before upgrade (from Phase 1)
- After upgrade (current)

Check for:
- Heading size differences (H1 should be larger than H2, etc.)
- Layout changes
- Color differences
- Font rendering

### Phase 7: Update Documentation

**Files to update:**

1. **`docs/08-egui-Migration-Guide.md`**
   - Update "Current Version" section
   - Update version compatibility table
   - Add "0.34 upgrade notes" if needed

2. **`docs/Markdown-Rendering-Guide.md`**
   - Remove heading size limitation note (if fixed)
   - Update version numbers
   - Update code examples if API changed

3. **`crates/dsl-egui/Cargo.toml`**
   - Already updated in Phase 2

4. **`README.md`** (if exists)
   - Update version numbers

### Phase 8: Commit and Review

```bash
# Review all changes
git diff

# Stage changes
git add -A

# Commit with descriptive message
git commit -m "Upgrade egui ecosystem from 0.31 to 0.34

- Update all egui crates to 0.34
- Fix Line::new() and Points::new() API (add name parameter)
- Update menu::bar() to MenuBar::new().ui()
- Fix heading size hierarchy in markdown rendering
- Update documentation

Fixes #<issue-number> (if applicable)"

# Push branch
git push origin egui-upgrade-0.34
```

**Code Review Checklist:**
- [ ] All compiler warnings resolved
- [ ] No version conflicts in `cargo tree`
- [ ] Manual testing passed
- [ ] Documentation updated
- [ ] No regression in existing features
- [ ] Markdown headings show size hierarchy

---

## Troubleshooting Guide

### Issue 1: Version Conflicts

**Error:**
```
error[E0308]: mismatched types
note: two different versions of crate `egui` are being used
```

**Diagnosis:**
```bash
cargo tree -p dsl-egui | grep -E "egui v"
```

**Solution:**
- Ensure ALL egui crates use the same minor version
- Check for indirect dependencies pulling old versions
- Use `cargo update` to sync versions
- If needed, use `cargo clean && cargo build`

### Issue 2: Missing Methods

**Error:**
```
error[E0599]: no method named `rounding` found for struct `Frame`
```

**Solution:**
- Check egui changelog for renamed methods
- Use `corner_radius` instead of `rounding`
- Search docs: `cargo doc --package egui --open`

### Issue 3: Wrong Number of Arguments

**Error:**
```
error[E0061]: this function takes 2 arguments but 1 argument was supplied
```

**Location:** `Line::new()` or `Points::new()`

**Solution:**
- Add name parameter as first argument
- Example: `Line::new("my_line", points)`

### Issue 4: Color32 Type Mismatch

**Error:**
```
error[E0277]: the trait bound `ecolor::color32::Color32: From<Color32>` is not satisfied
```

**Solution:**
```rust
// Change this:
use egui::Color32;

// To this:
use egui::ecolor::Color32;
```

### Issue 5: Cache Errors

**Error:** Issues with `cargo` cache

**Solution:**
```bash
# Nuclear option - clean everything
cargo clean
rm -rf ~/.cargo/registry/index/*
rm -rf ~/.cargo/registry/cache/*
cargo build -p dsl-egui
```

---

## Rollback Plan

If upgrade fails or causes critical issues:

```bash
# Revert all changes
git checkout main

# Or reset branch
git reset --hard origin/main

# Or revert specific commit
git revert <commit-hash>

# Verify working state
cargo build -p dsl-egui
cargo run -p dsl-egui
```

**Cargo.toml rollback:**
```toml
eframe = "0.31"
egui = "0.31"
egui_extras = { version = "0.31", features = ["syntect", "image"] }
egui_plot = "0.31"
egui_commonmark = "0.20"
```

---

## Success Criteria

Upgrade is successful when:

- ✅ `cargo build -p dsl-egui` completes with 0 errors
- ✅ `cargo test -p dsl-egui` passes
- ✅ `cargo tree -p dsl-egui | grep egui` shows consistent versions
- ✅ GUI launches without crashes
- ✅ All manual tests pass
- ✅ Markdown headings show proper size hierarchy (H1 > H2 > H3)
- ✅ Charts render correctly
- ✅ No visual regressions
- ✅ Documentation updated

---

## Lessons Learned

### Version Management

1. **Always check `cargo tree`** - Visual inspection of dependency tree prevents version conflicts
2. **Test incrementally** - Upgrade one minor version at a time when possible
3. **Read changelogs** - Check egui GitHub releases for breaking changes
4. **Skip broken versions** - egui_plot 0.32.0 taught us to verify intermediate versions

### API Stability

1. **Immediate mode = API churn** - Expect breaking changes in minor versions
2. **Type system is strict** - Rust won't let you mix types across versions
3. **Import paths matter** - `egui::Color32` vs `ecolor::Color32` is significant

### Testing Strategy

1. **Compiler is your friend** - Most issues caught at compile time
2. **Visual testing required** - Some issues only visible in UI (heading sizes)
3. **Keep screenshots** - Visual regression testing is valuable

---

## References

- [egui Changelog](https://github.com/emilk/egui/blob/master/CHANGELOG.md)
- [egui_plot Changelog](https://github.com/emilk/egui_plot/releases)
- [egui_commonmark Changelog](https://github.com/lampsitter/egui_commonmark/releases)
- [egui Migration Guide](./08-egui-Migration-Guide.md)
- [Markdown Rendering Guide](./Markdown-Rendering-Guide.md)

---

## Appendix: Version History

### Why We're on 0.31

**Decision Date:** 2025-11-10

**Rationale:**
1. egui 0.29 → 0.31 upgrade needed for egui_commonmark compatibility
2. Discovered egui_plot 0.32.0 dependency bug
3. Chose stability over latest features
4. 0.31 is known-good configuration

**Trade-offs:**
- ❌ All headings same size (limitation of egui_commonmark 0.20)
- ❌ Using deprecated menu::bar() API
- ✅ Stable, no type conflicts
- ✅ All features working
- ✅ Well-tested configuration

### Future Upgrade Triggers

Consider upgrading when:
- Critical security issue in 0.31
- Need features only available in newer versions
- egui_commonmark 0.22+ required for heading hierarchy
- egui LTS version released
- Team bandwidth available for testing

---

**Document Version:** 1.0
**Last Updated:** 2025-11-10
**Author:** Development Team
**Next Review:** Before next major feature addition
