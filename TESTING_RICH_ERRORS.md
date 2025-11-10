# Testing Rich Error Display - Quick Reference

## How to Test

1. **Start the GUI:**
   ```bash
   cargo run --bin dsl-gui
   ```

2. **Type errors directly in the REPL** (bottom input area)
3. **Or load the test file:** File > Open > `examples/test_rich_errors.dsl`

---

## Quick Test Cases (Copy & Paste These)

### ✅ Easy to Test (No Setup Required)

#### 1. Unknown Variable
```
unknown_var
```
**Expected:** 🔴 UNKNOWN VARIABLE with suggestions to define it

#### 2. Unknown Function
```
nonexistent_func()
```
**Expected:** 🔴 UNKNOWN FUNCTION with suggestions to check spelling

#### 3. Invalid Command
```
:badcommand
```
**Expected:** 🔴 ERROR with "Unknown command" message

#### 4. Wrong Argument Count
```
let greet = |name| "Hello " + name
greet()
```
**Expected:** 🔴 INVALID ARGUMENTS with span info

#### 5. Type Error (String Function on Number)
```
let x = 42
upper(x)
```
**Expected:** 🔴 RUNTIME ERROR (upper expects string)

#### 6. Type Error (Math on String)
```
let name = "Alice"
name * 2
```
**Expected:** 🔴 RUNTIME ERROR (can't multiply string)

#### 7. Parse Error
```
let x =
```
**Expected:** 🔴 PARSE ERROR with syntax issue

#### 8. Accessing Non-Existent Map Key
```
let person = {"name": "Alice"}
person.age
```
**Expected:** Error or null (depending on implementation)

#### 9. List Index Out of Bounds
```
let items = [1, 2, 3]
items[10]
```
**Expected:** 🔴 RUNTIME ERROR

#### 10. Division by Zero
```
100 / 0
```
**Expected:** 🔴 RUNTIME ERROR

---

## Error Display Features to Check

### For Each Error, Verify:

1. **Visual Design:**
   - [ ] Dark red background with border
   - [ ] Error type in red (e.g., "UNKNOWN VARIABLE")
   - [ ] 🔴 Red circle icon
   - [ ] Clear, readable text

2. **Error Information:**
   - [ ] Error type/category displayed
   - [ ] Clear error message
   - [ ] Source location (if available): `at file.dsl:line:column`
   - [ ] Function context (if applicable)

3. **Interactive Features:**
   - [ ] ▶ Arrow button appears for expandable sections
   - [ ] Click arrow to expand/collapse
   - [ ] Arrow changes to ▼ when expanded
   - [ ] Hover shows "Expand"/"Collapse" tooltip

4. **Suggestions Section (if present):**
   - [ ] 💡 Suggestions header with icon
   - [ ] Click to expand/collapse
   - [ ] Bullet points with helpful tips
   - [ ] Yellow/gold color for visibility

5. **Source Location Section:**
   - [ ] 📍 Source Location header
   - [ ] File name displayed
   - [ ] Line and column numbers
   - [ ] Expandable/collapsible

---

## Expected Error Types

| Input | Error Type | Has Suggestions | Has Location |
|-------|-----------|----------------|--------------|
| `unknown_var` | UNKNOWN VARIABLE | ✅ | ✅ |
| `unknown_func()` | UNKNOWN FUNCTION | ✅ | ✅ |
| `:badcmd` | ERROR | ❌ | ❌ |
| `upper(42)` | RUNTIME ERROR | ❌ | ✅ |
| `let x =` | PARSE ERROR | ❌ | ✅ |
| Wrong args | INVALID ARGUMENTS | ✅ | ✅ |
| `100 / 0` | RUNTIME ERROR | ❌ | ✅ |

---

## Advanced Test Cases (Require Setup)

### LLM Errors (Need API Key)

1. **Set up environment:**
   ```bash
   export OPENAI_API_KEY="your-key-here"
   ```

2. **Test LLM error:**
   ```
   function test_llm() -> String {
       prompt: "Extract name"
       model: "gpt-4"
   }
   test_llm()
   ```

**Expected Features:**
- 🔴 LLM ERROR
- Function context shown
- ▶ 📝 Prompt (expandable)
- ▶ 💬 Response (expandable)
- Suggestions about API credentials

### HTTP Errors (Need Network)

```
function test_http() -> String {
    http: GET "https://invalid-domain-xyz123.com"
}
test_http()
```

**Expected Features:**
- 🔴 HTTP ERROR
- Method: GET
- URL displayed
- Suggestions about connectivity

### SQL Errors (Need Database)

```
function test_sql() -> List {
    sql: "SELECT * FROM nonexistent_table"
}
test_sql()
```

**Expected Features:**
- 🔴 SQL ERROR
- ▶ 🗄️ Query (expandable)
- Suggestions about SQL syntax

---

## Testing Expandable Sections

### Test Expansion Behavior:

1. **Click once:** Section expands
   - [ ] Arrow changes from ▶ to ▼
   - [ ] Content appears below
   - [ ] Section has dark background

2. **Click again:** Section collapses
   - [ ] Arrow changes from ▼ to ▶
   - [ ] Content disappears
   - [ ] Only header remains visible

3. **Multiple sections:** Expand independently
   - [ ] Can have Suggestions expanded while Source collapsed
   - [ ] State persists for each section
   - [ ] No interference between sections

### Long Content Tests:

```
let very_long_string = "This is a very long string that goes on and on and on and should be truncated when displayed in the error message to prevent the UI from becoming unwieldy and taking up too much space in the output area because we want to keep things readable and manageable for the user experience and this string just keeps going and going and going..."

nonexistent_function(very_long_string)
```

**Check:**
- [ ] Long content is truncated at ~500 chars
- [ ] Shows "..." at the end
- [ ] Scroll bar appears if needed
- [ ] Doesn't break layout

---

## Color Verification

### Check These Colors Appear:

- **Error Type:** Bright red (`rgb(255, 100, 100)`)
- **Background:** Dark red (`rgb(40, 30, 30)`)
- **Border:** Red (`rgb(200, 60, 60)`)
- **Expected Type:** Green (`rgb(100, 200, 100)`)
- **Got Type:** Red (`rgb(255, 100, 100)`)
- **Suggestions:** Yellow (`rgb(255, 220, 100)`)
- **Code/URLs:** Light blue (`rgb(150, 200, 255)`)
- **Text:** Light gray (`rgb(220, 220, 220)`)

---

## Performance Testing

### Test with Multiple Errors:

1. **Generate 10+ errors quickly:**
   ```
   error1
   error2
   error3
   ...
   error10
   ```

2. **Check:**
   - [ ] UI remains responsive
   - [ ] Scrolling is smooth
   - [ ] Expansion still works
   - [ ] No lag or freezing

---

## Screenshot Locations

Take screenshots of these for documentation:

1. Unknown Variable error (expanded suggestions)
2. Type Error with expected vs got
3. Multiple errors in output
4. LLM error with expanded prompt/response
5. Side-by-side: old plain string vs new rich error

---

## Troubleshooting

### If errors don't show rich display:

1. **Check build:** `cargo build --package dsl-egui`
2. **Verify files exist:**
   - `crates/dsl-egui/src/renderers/error.rs`
   - `crates/dsl-egui/src/output_item.rs` (ErrorDetail added)
3. **Check imports:** `use crate::output_item::ErrorDetail;` in repl.rs
4. **Restart GUI:** Old version might be cached

### If expandable sections don't work:

1. **Check for borrow errors** in cargo build output
2. **Verify HashSet** is imported in output_item.rs
3. **Check button click handlers** in error.rs

### If colors look wrong:

1. **Check theme:** Dark theme recommended
2. **Verify Color32 values** match spec in error.rs
3. **Test in different lighting** conditions

---

## Success Criteria

Rich error display is working correctly if:

- ✅ Errors show in red boxes with borders
- ✅ Error types are clearly labeled
- ✅ Icons appear (🔴 💡 📍 etc.)
- ✅ Expandable sections work smoothly
- ✅ Suggestions are helpful and relevant
- ✅ Source locations are accurate
- ✅ Colors enhance readability
- ✅ Layout is clean and professional
- ✅ No crashes or freezes
- ✅ Performance is acceptable

---

## Quick Start Testing (30 seconds)

```bash
# Terminal 1: Start GUI
cargo run --bin dsl-gui

# Terminal 1: In REPL, type:
unknown_var

# Expected: See rich error with red box, suggestions, source location
# Click the ▶ arrow next to "💡 Suggestions" to expand
# Click the ▶ arrow next to "📍 Source Location" to expand
```

If you see a nicely formatted error with expandable sections, it's working! 🎉

---

**Last Updated:** November 9, 2025
**Version:** 2.0.0
**Status:** Ready for Testing ✅
