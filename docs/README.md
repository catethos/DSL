# Documentation Index

**Last Updated:** 2025-11-10

Welcome to the DSL documentation! This guide helps you find the right documentation for your needs.

---

## 🚀 Quick Start

### New Users

1. **[Getting Started](user-guide/02-Getting-Started.md)** - Installation and first steps
2. **[Language Features](user-guide/04-Language-Features.md)** - Learn the DSL syntax
3. **[TUI Interface](gui/tui-interface.md)** or **[Desktop GUI](gui/egui-desktop-gui.md)** - Choose your interface

### Developers

1. **[Architecture](developer/architecture.md)** - Understand the system design
2. **[Building](developer/building.md)** - Build from source
3. **[egui Implementation](developer/egui-implementation.md)** - GUI development

---

## 📚 Documentation Structure

### User Guide (`user-guide/`)

Complete guides for using the DSL:

| Document | Description |
|----------|-------------|
| [01-Overview.md](user-guide/01-Overview.md) | Project overview and architecture |
| [02-Getting-Started.md](user-guide/02-Getting-Started.md) | Installation and first steps |
| [04-Language-Features.md](user-guide/04-Language-Features.md) | Complete language reference |
| [05-Type-System.md](user-guide/05-Type-System.md) | Custom types and enums |
| [06-Builtin-Functions.md](user-guide/06-Builtin-Functions.md) | 50+ builtin functions |
| [07-Workflow-Constructs.md](user-guide/07-Workflow-Constructs.md) | Sequential & parallel workflows |
| [08-LLM-Integration.md](user-guide/08-LLM-Integration.md) | AI/LLM features |
| [09-SQL-DuckDB.md](user-guide/09-SQL-DuckDB.md) | Data processing with SQL |
| [10-HTTP-Client.md](user-guide/10-HTTP-Client.md) | API integration |

---

### GUI Documentation (`gui/`)

Interface guides:

| Document | Description |
|----------|-------------|
| [egui-desktop-gui.md](gui/egui-desktop-gui.md) | Modern desktop GUI with rich rendering |
| [tui-interface.md](gui/tui-interface.md) | Terminal-based interface |

**Both interfaces are actively maintained!** Choose based on your preference.

---

### Advanced Topics (`advanced/`)

In-depth features:

| Document | Description |
|----------|-------------|
| [11-Advanced-Features.md](advanced/11-Advanced-Features.md) | Advanced patterns and techniques |
| [13-Chart-Generation.md](advanced/13-Chart-Generation.md) | Data visualization |
| [tracing-and-debugging.md](advanced/tracing-and-debugging.md) | Execution tracing and debugging |

---

### Developer Documentation (`developer/`)

For contributors and maintainers:

| Document | Description |
|----------|-------------|
| [architecture.md](developer/architecture.md) | IR-based system architecture (11 crates) |
| [error-handling.md](developer/error-handling.md) | Error system internals |
| [known-issues.md](developer/known-issues.md) | Current limitations and fixes |
| [egui-implementation.md](developer/egui-implementation.md) | GUI development guide |
| [building.md](developer/building.md) | Build and compilation guide |
| [EGUI-UPGRADE-PLAN.md](EGUI-UPGRADE-PLAN.md) | ⚠️ **Critical:** egui version upgrade guide |

**👉 Before upgrading egui versions, read [EGUI-UPGRADE-PLAN.md](EGUI-UPGRADE-PLAN.md)!**

---

### Reference Materials (`references/`)

Design and history:

| Document | Description |
|----------|-------------|
| [dspy-comparison.md](references/dspy-comparison.md) | DSPy vs DSL design analysis |
| [operator-changes.md](references/operator-changes.md) | Historical syntax changes |

---

### Migration Guides (`migration-guides/`)

Upgrading and transitions:

| Document | Description |
|----------|-------------|
| [autocomplete.md](migration-guides/autocomplete.md) | Autocomplete system guide |

---

## 🎯 By Use Case

### I want to...

#### **Learn the Language**
→ [Getting Started](user-guide/02-Getting-Started.md) → [Language Features](user-guide/04-Language-Features.md)

#### **Use LLM/AI Features**
→ [LLM Integration](user-guide/08-LLM-Integration.md)

#### **Process Data**
→ [SQL/DuckDB](user-guide/09-SQL-DuckDB.md) for SQL queries
→ [HTTP Client](user-guide/10-HTTP-Client.md) for API calls

#### **Build Workflows**
→ [Workflow Constructs](user-guide/07-Workflow-Constructs.md)

#### **Create Custom Types**
→ [Type System](user-guide/05-Type-System.md)

#### **Use the GUI**
→ [Desktop GUI Guide](gui/egui-desktop-gui.md) or [TUI Guide](gui/tui-interface.md)

#### **Contribute Code**
→ [Architecture](developer/architecture.md) → [Building](developer/building.md)

#### **Debug Issues**
→ [Known Issues](developer/known-issues.md) → [Tracing](advanced/tracing-and-debugging.md)

---

## 📖 Learning Paths

### Path 1: Complete Beginner

1. [Overview](user-guide/01-Overview.md) - What is the DSL?
2. [Getting Started](user-guide/02-Getting-Started.md) - Install and run
3. [TUI Interface](gui/tui-interface.md) - Navigate the interface
4. [Language Features](user-guide/04-Language-Features.md) - Learn syntax
5. [Builtin Functions](user-guide/06-Builtin-Functions.md) - Use functions
6. [Type System](user-guide/05-Type-System.md) - Define types
7. [Workflows](user-guide/07-Workflow-Constructs.md) - Build workflows

**Time:** 2-3 hours

---

### Path 2: Experienced Programmer

1. [Getting Started](user-guide/02-Getting-Started.md) - Quick setup
2. [Language Features](user-guide/04-Language-Features.md) - Syntax overview
3. [LLM Integration](user-guide/08-LLM-Integration.md) or [SQL/DuckDB](user-guide/09-SQL-DuckDB.md)
4. [Advanced Features](advanced/11-Advanced-Features.md) - Power features

**Time:** 30-60 minutes

---

### Path 3: Contributor

1. [Architecture](developer/architecture.md) - System design
2. [Building](developer/building.md) - Build from source
3. [Known Issues](developer/known-issues.md) - Current work
4. [Error Handling](developer/error-handling.md) or [egui Implementation](developer/egui-implementation.md)

**Time:** 1-2 hours

---

## 🔍 Finding Information

### By Topic

**Language:**
- Syntax → [Language Features](user-guide/04-Language-Features.md)
- Types → [Type System](user-guide/05-Type-System.md)
- Functions → [Builtin Functions](user-guide/06-Builtin-Functions.md)

**Integration:**
- LLM/AI → [LLM Integration](user-guide/08-LLM-Integration.md)
- Database → [SQL/DuckDB](user-guide/09-SQL-DuckDB.md)
- APIs → [HTTP Client](user-guide/10-HTTP-Client.md)

**Interface:**
- Desktop → [egui Desktop GUI](gui/egui-desktop-gui.md)
- Terminal → [TUI Interface](gui/tui-interface.md)

**Development:**
- System Design → [Architecture](developer/architecture.md)
- Building → [Building Guide](developer/building.md)
- Errors → [Error Handling](developer/error-handling.md)
- GUI Code → [egui Implementation](developer/egui-implementation.md)

---

## 📋 Document Status

### User Documentation ✅

| Category | Status | Files |
|----------|--------|-------|
| User Guide | ✅ Complete | 9 files |
| GUI Documentation | ✅ Complete | 2 files |
| Advanced Topics | ✅ Complete | 3 files |

### Developer Documentation ✅

| Category | Status | Files |
|----------|--------|-------|
| Architecture | ✅ Complete | 5 files |
| References | ✅ Complete | 2 files |
| Migration Guides | ✅ Complete | 1 file |

---

## 🔗 Related Resources

### In Repository

- **[Main README](../README.md)** - Project overview
- **[Examples](../examples/)** - Example programs
- **[Historical Docs](../historical/README.md)** - Development history

### LLM Integration

- **[simplify_baml](../crates/simplify_baml/README.md)** - LLM framework documentation

### External Links

- **GitHub Issues** - Report bugs or request features
- **Architecture Diagrams** - See [Architecture](developer/architecture.md)

---

## 📊 Documentation Statistics

### Content

- **Total Documents:** 20+ organized files
- **User Guides:** 9 comprehensive guides
- **Developer Docs:** 5 technical documents
- **Total Lines:** 15,000+ lines of documentation

### Organization

- ✅ Clear hierarchy (user-guide/, developer/, gui/, etc.)
- ✅ Cross-references between documents
- ✅ Examples in every guide
- ✅ Search-friendly structure

---

## 🎓 Documentation Conventions

### Code Examples

All code examples follow this format:

```javascript
// Input
let x = 5
let y = 10
x + y

// Output
15
```

### Document Structure

Each document includes:
- **Table of contents** - Quick navigation
- **Clear sections** - Organized topics
- **Code examples** - Practical demonstrations
- **Best practices** - Tips and recommendations
- **Related documents** - Cross-references

---

## 🆘 Getting Help

### In the Application

- Type `:help` in REPL for quick reference
- Press `F1` for interface help (TUI)
- Use `:vars`, `:types`, `:funcs` commands

### In Documentation

1. Check this index for topic
2. Read relevant user guide
3. Try examples
4. Check troubleshooting in [Known Issues](developer/known-issues.md)

### Still Stuck?

- Search documentation for keywords
- Check [examples/](../examples/) directory
- Review [architecture](developer/architecture.md) for technical details
- See [historical docs](../historical/README.md) for context

---

## 🤝 Contributing

### Documentation

- Follow existing structure
- Include examples
- Test all code snippets
- Add cross-references

### Style Guide

- Use clear, concise language
- Start simple, progress to complex
- Show input and output
- Include "Next Steps" or "See Also"

---

## 📝 Version History

### Current (November 2025)

- ✅ Complete documentation reorganization
- ✅ User guide (9 files)
- ✅ Developer documentation (5 files)
- ✅ GUI guides (2 files)
- ✅ Advanced topics (3 files)
- ✅ References and migration guides

### Previous

- Original flat documentation structure
- Phase-by-phase development docs
- Scattered reference materials

---

## 🚀 Next Steps

### New Users
→ Start with [Getting Started](user-guide/02-Getting-Started.md)

### Returning Users
→ Check [What's New](#) for recent updates

### Contributors
→ Read [Architecture](developer/architecture.md) and [Building](developer/building.md)

---

**Happy Learning!** 🎉

**Quick Links:**
[Getting Started](user-guide/02-Getting-Started.md) |
[Language Features](user-guide/04-Language-Features.md) |
[Architecture](developer/architecture.md) |
[Examples](../examples/)
