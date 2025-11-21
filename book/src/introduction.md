# Introduction

Welcome to the DSL documentation! DSL is a powerful Domain-Specific Language designed for building agentic LLM workflows, data processing pipelines, and integration systems.

## What is DSL?

DSL is a modern, expressive language that combines:

- **Simple, readable syntax** inspired by functional programming
- **Native LLM integration** with multi-provider support (OpenAI, Anthropic, OpenRouter)
- **Powerful data processing** with DuckDB integration and SQL support
- **Type-safe workflows** with custom types and BAML integration
- **Rich ecosystem** with multiple interfaces (CLI, TUI, Desktop GUI)
- **50+ builtin functions** for common operations

## Quick Example

```dsl
[1, 2, 3, 4, 5] as numbers

numbers |> sum(_)

"hello world" |> upper(_) |> split(_, " ")

type Person {
    name: String
    age: Int
    email: String
}

(1 || 2 || 3) as results
```

## Who is this documentation for?

This documentation serves multiple audiences:

- **New Users**: Start with [Getting Started](./getting-started/installation.md) to install DSL and write your first program
- **Integration Developers**: Jump to [Integration](./integration/llm/overview.md) to learn about LLM, database, and HTTP integration
- **Library Users**: Check out the [API Documentation](./api/embedding/simple-evaluation.md) to embed DSL in your Rust applications
- **Contributors**: Read the [Developer Guide](./developer-guide/contributing.md) to contribute to the project

## Learning Paths

We recommend three paths through this documentation:

### 1. Beginner Path
If you're new to DSL:
1. [Getting Started](./getting-started/installation.md)
2. [User Guide](./user-guide/language/syntax.md)
3. [Cookbook](./cookbook/data-pipelines.md)

### 2. Integration Path
If you want to build LLM workflows:
1. [LLM Integration](./integration/llm/overview.md)
2. [Cookbook](./cookbook/llm-workflows.md)
3. [Advanced Topics](./advanced/pattern-matching.md)

### 3. Contributor Path
If you want to contribute to DSL:
1. [Developer Guide](./developer-guide/contributing.md)
2. [Architecture](./architecture/overview.md)
3. [API Documentation](./api/crates/dsl-core.md)

## Features at a Glance

- **🚀 Fast**: Compiles to intermediate representation for efficient execution
- **🔧 Flexible**: Embed in Rust applications or use standalone
- **🤖 AI-Native**: Built-in LLM integration with streaming support
- **📊 Data-Rich**: DuckDB, CSV, and SQL support out of the box
- **🎨 Interactive**: TUI and Desktop GUI for visual development
- **🔒 Type-Safe**: Strong type system with runtime validation
- **📦 Extensible**: Add custom builtins and extend functionality

## Getting Help

- **Issues**: Found a bug? [Report it on GitHub](https://github.com/catethos/DSL/issues)
- **Questions**: Check the [FAQ](./troubleshooting/faq.md) or [Troubleshooting](./troubleshooting/common-errors.md)
- **Contributing**: See our [Contributing Guide](./developer-guide/contributing.md)

## Documentation Status

This documentation is actively being developed. Some sections are marked as:

- ✅ **Complete**: Fully documented with examples
- 🚧 **In Progress**: Content being added
- 📋 **Planned**: On the roadmap

Let's get started! Head to [Installation](./getting-started/installation.md) to begin your journey with DSL.
