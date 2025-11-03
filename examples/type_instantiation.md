# Type Instantiation Example

The DSL now supports instantiating user-defined types with validation.

## Basic Usage

```dsl
# Define a type
type Person { 
  name: string, 
  age: int,
  email?: string
}

# Instantiate with required fields
Person { name: "Alice", age: 30 }

# Instantiate with optional field
Person { name: "Bob", age: 25, email: "bob@example.com" }
```

## Features

### 1. Required and Optional Fields
Fields marked with `?` are optional. All other fields are required.

```dsl
type User {
  username: string,
  password: string,
  bio?: string
}

# Valid - bio is optional
User { username: "alice", password: "secret" }

# Also valid - with optional field
User { username: "bob", password: "pass123", bio: "Developer" }
```

### 2. Type Validation
The evaluator validates:
- All required fields are present
- No unknown fields are provided
- Field values match the declared types

```dsl
type Point { x: int, y: int }

# Valid
Point { x: 10, y: 20 }

# Error: Missing required field 'y'
Point { x: 10 }

# Error: Unknown field 'z'
Point { x: 10, y: 20, z: 30 }

# Error: Field 'x' expects Int, got String
Point { x: "ten", y: 20 }
```

### 3. Expressions in Field Values
Field values can be expressions, not just literals.

```dsl
type Rectangle { width: int, height: int, area: int }

Rectangle { width: 10, height: 20, area: 10 * 20 }
```

### 4. Nested Types
Types can contain other types as fields.

```dsl
type Address { street: string, city: string }
type Person { name: string, address: Address }

Person {
  name: "Alice",
  address: Address {
    street: "123 Main St",
    city: "NYC"
  }
}
```

### 5. Lists of Types
You can create lists of instantiated types.

```dsl
type Task { title: string, done: bool }

[
  Task { title: "Buy milk", done: false },
  Task { title: "Walk dog", done: true },
  Task { title: "Write code", done: false }
]
```

## Implementation Details

- Type instantiation uses the syntax: `TypeName { field: value, ... }`
- Instantiated types are represented internally as `Value::Map`
- Validation occurs at evaluation time, not parse time
- All field expressions are evaluated before validation
