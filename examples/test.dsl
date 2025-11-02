// Test DSL file with syntax highlighting

type User {
    id: Int "User ID"
    name: String "User name"
    email?: String "Optional email"
}

enum Status {
    Active,
    Inactive,
    Pending
}

def get_user(user_id: Int) -> User {
    prompt: "Get user details for ${user_id}"
    sql: """
        SELECT id, name, email
        FROM users
        WHERE id = ${user_id}
    """
}

def process_users() {
    http: "GET"
    url: "https://api.example.com/users"
    headers: {
        "Authorization": "Bearer token123"
    }
}

workflow analyze_data(table: Table) -> Int :=
    get_user(123) >> process_users() || calculate_stats(table)
