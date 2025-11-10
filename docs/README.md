# Documentation Index

Welcome to the DSL TUI documentation! This directory contains comprehensive guides for understanding and using the Agentic LLM Workflow DSL.

## Documentation Structure

### Getting Started
1. **[01-Overview.md](01-Overview.md)** - Project introduction, architecture, and design principles
2. **[02-Getting-Started.md](02-Getting-Started.md)** - Installation, first steps, and tutorials
3. **[03-TUI-Interface.md](03-TUI-Interface.md)** - UI modes, navigation, and interface guide

### Core Concepts
4. **[04-Language-Features.md](04-Language-Features.md)** - Syntax, data types, operators, and expressions
5. **[05-Type-System.md](05-Type-System.md)** - Custom types, enums, and type validation
6. **[06-Builtin-Functions.md](06-Builtin-Functions.md)** - Complete reference for 50+ builtin functions
7. **[07-Workflow-Constructs.md](07-Workflow-Constructs.md)** - Sequential, parallel, and conditional workflows

### Integration Features
8. **[08-LLM-Integration.md](08-LLM-Integration.md)** - AI/LLM features with simplify_baml
9. **[09-SQL-DuckDB.md](09-SQL-DuckDB.md)** - Data processing with SQL and DuckDB
10. **[10-HTTP-Client.md](10-HTTP-Client.md)** - API integration and HTTP requests

### Advanced Topics
11. **[11-Advanced-Features.md](11-Advanced-Features.md)** - Advanced usage patterns and techniques
12. **[12-Implementation-Details.md](12-Implementation-Details.md)** - Technical architecture and internals

### GUI Development
13. **[08-egui-Migration-Guide.md](08-egui-Migration-Guide.md)** - Desktop GUI with egui framework
    - Rich display system (tables, markdown, images, charts)
    - Migration from TUI to GUI
    - Complete renderer documentation

## Quick Links

### For New Users
Start here:
1. Read [01-Overview.md](01-Overview.md) to understand what the DSL does
2. Follow [02-Getting-Started.md](02-Getting-Started.md) for installation and tutorials
3. Learn the [03-TUI-Interface.md](03-TUI-Interface.md) to navigate the application

### For Developers
Building workflows:
1. Master [04-Language-Features.md](04-Language-Features.md) for syntax
2. Define custom [05-Type-System.md](05-Type-System.md) types
3. Reference [06-Builtin-Functions.md](06-Builtin-Functions.md) for all builtin functions
4. Build [07-Workflow-Constructs.md](07-Workflow-Constructs.md) with operators

### For Integration
Connecting to external services:
1. Use [08-LLM-Integration.md](08-LLM-Integration.md) for AI features
2. Process data with [09-SQL-DuckDB.md](09-SQL-DuckDB.md)
3. Call APIs with [10-HTTP-Client.md](10-HTTP-Client.md)

### For Advanced Users
Deep dive:
1. Explore [11-Advanced-Features.md](11-Advanced-Features.md) patterns
2. Understand [12-Implementation-Details.md](12-Implementation-Details.md) internals

## Documentation Format

Each document includes:
- Clear section headings
- Code examples with expected output
- Best practices and tips
- Related documents links
- Next steps guidance

## How to Read This Documentation

### Linear Reading Path
Read documents in order (01 → 12) for comprehensive understanding.

**Recommended for:** Complete beginners, systematic learners

### Topic-Based Reading
Jump to specific topics as needed.

**Recommended for:** Experienced developers, quick reference

### Example-Driven Learning
Start with examples, refer to docs when needed.

**Recommended for:** Hands-on learners, quick prototyping

## Related Resources

### In Repository
- **[../README.md](../README.md)** - Project README and quick overview
- **[../DESIGN.md](../DESIGN.md)** - Complete design specification
- **[../PROGRESS.md](../PROGRESS.md)** - Development progress tracker
- **[../examples/](../examples/)** - Example workflow files
- **[../examples/README.md](../examples/README.md)** - Examples guide

### Specific Guides
- **[../EDITOR_USAGE.md](../EDITOR_USAGE.md)** - Editor-specific guide
- **[../HTTP_QUICK_START.md](../HTTP_QUICK_START.md)** - HTTP quick reference
- **[../HTTP_CLIENT_SUMMARY.md](../HTTP_CLIENT_SUMMARY.md)** - Complete HTTP docs
- **[../MULTILINE_INPUT.md](../MULTILINE_INPUT.md)** - Multi-line input guide
- **[../CSV_USAGE.md](../CSV_USAGE.md)** - CSV file handling

### Historical Documents
- **[../REPL_FIRST_PLAN.md](../REPL_FIRST_PLAN.md)** - Incremental development plan
- **[../PHASE*_*.md](../)** - Phase-specific documentation
- **[../SESSION_SUMMARY.md](../SESSION_SUMMARY.md)** - Development sessions
- **[../COMPLETION_SUMMARY.md](../COMPLETION_SUMMARY.md)** - Feature completions
- **[../IMPLEMENTATION_COMPLETE.md](../IMPLEMENTATION_COMPLETE.md)** - HTTP implementation

## Document Status

| Document | Status | Last Updated |
|----------|--------|--------------|
| 01-Overview.md | ✅ Complete | 2025-11-01 |
| 02-Getting-Started.md | ✅ Complete | 2025-11-01 |
| 03-TUI-Interface.md | ✅ Complete | 2025-11-01 |
| 04-Language-Features.md | ✅ Complete | 2025-11-01 |
| 05-Type-System.md | ✅ Complete | 2025-11-01 |
| 06-Functions.md | 🚧 In Progress | 2025-11-01 |
| 07-Workflow-Constructs.md | 🚧 In Progress | 2025-11-01 |
| 08-LLM-Integration.md | 🚧 In Progress | 2025-11-01 |
| 09-SQL-DuckDB.md | 🚧 In Progress | 2025-11-01 |
| 10-HTTP-Client.md | 🚧 In Progress | 2025-11-01 |
| 11-Advanced-Features.md | 🚧 In Progress | 2025-11-01 |
| 12-Implementation-Details.md | 🚧 In Progress | 2025-11-01 |

## Contributing to Documentation

### Style Guide
- Use clear, concise language
- Include code examples
- Show expected output
- Add "Next Steps" section
- Link related documents

### Structure
- Start with overview
- Progress from simple to complex
- Include practical examples
- End with best practices

### Code Examples
```javascript
// Good example: Shows input and output
flow> 42 as answer
✓ Bound 'answer' to 42 : Int

flow> answer * 2
✓ 84 : Int
```

## Getting Help

### Within the Application
- Press **F1** for REPL mode
- Type `:help` for quick reference
- Press **F3** for Type Explorer
- Use `:vars`, `:types`, `:funcs` commands

### In Documentation
- Check the relevant section in this index
- Follow "Next Steps" links in documents
- Search for keywords across files

### External Resources
- GitHub Issues: [Report problems](https://github.com/anthropics/DSL/issues)
- Examples: See `../examples/` directory
- Design Spec: Read `../DESIGN.md`

## Feedback

Found an error? Have a suggestion?
- Open an issue on GitHub
- Submit a pull request
- Contact the maintainers

## Version History

### v1.0 (Current)
- Complete documentation set
- All 12 core documents
- Examples and guides
- Implementation complete

### Future Versions
- Video tutorials
- Interactive examples
- API reference
- Cookbook

## License

See the main repository LICENSE file for licensing information.

---

**Happy Learning!** 🚀

Start your journey with [01-Overview.md](01-Overview.md) or jump straight to [02-Getting-Started.md](02-Getting-Started.md) to begin coding!
