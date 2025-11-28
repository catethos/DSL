# Module System Example

This example demonstrates the DSL's module system, which allows you to organize code across multiple files.

## Files

- `math.dsl` - Mathematical utility functions
- `string_utils.dsl` - String manipulation functions  
- `main.dsl` - Main program that imports and uses the modules

## Running the Example

```bash
# Run the main program
dsl run examples/modules/main.dsl

# Check for errors
dsl check examples/modules/main.dsl
```

## Module System Features

### Import Syntax

```dsl
import "./math.dsl"              # Import from same directory
import "../lib/utils.dsl"        # Import from parent directory
import "./http.dsl" as http      # Import with namespace alias (coming soon)
```

### Key Features

1. **File-level imports** - Import all functions, types, and enums from a module
2. **Relative paths** - Paths are relative to the importing file
3. **Automatic caching** - Modules are loaded once and cached
4. **Circular dependency detection** - Clear error messages for cycles
5. **Everything is public** - No need for export declarations

### Design Philosophy

The module system follows these principles:

- **Simple** - Minimal syntax, easy to understand
- **Intuitive** - Similar to Python/JavaScript imports
- **Practical** - Solves the problem of organizing code across files

### Example: Multi-level Imports

You can create complex module hierarchies:

```
project/
  lib/
    math.dsl
    string.dsl
  utils/
    helpers.dsl       # imports ../lib/math.dsl
  main.dsl            # imports ./utils/helpers.dsl
```

The module loader will automatically resolve all dependencies and detect any circular imports.
