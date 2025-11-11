([
    #{name: "Alice", age: 25, city: "NYC"},
    #{name: "Bob", age: 30, city: "LA"},
    #{name: "Charlie", age: 35, city: "NYC"}
] -> users) -> (SQL("SELECT * FROM $users WHERE age > 28") -> result) -> print(result)
