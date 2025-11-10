# HTTP Client

## Overview

The DSL provides a full-featured HTTP client for integrating with REST APIs and web services.

## Quick Start

```javascript
def getTodo() {
  http: "GET"
  url: "https://jsonplaceholder.typicode.com/todos/1"
}

getTodo()
// { userId: 1, id: 1, title: "delectus aut autem", completed: false }
```

## HTTP Methods

### GET Requests

Fetch data from APIs.

```javascript
def getUser(userId: Int) {
  http: "GET"
  url: "https://api.example.com/users/${userId}"
}

getUser(123) as user
user.name
user.email
```

### POST Requests

Send data to APIs.

```javascript
def createPost(title: String, body: String) {
  http: "POST"
  url: "https://api.example.com/posts"
  headers: {
    "Content-Type": "application/json"
  }
  body: {
    "title": "${title}",
    "body": "${body}"
  }
}

createPost("My Title", "Content here") as result
```

### PUT Requests

Update existing resources.

```javascript
def updateUser(id: Int, name: String, email: String) {
  http: "PUT"
  url: "https://api.example.com/users/${id}"
  body: {
    "name": "${name}",
    "email": "${email}"
  }
}
```

### DELETE Requests

Remove resources.

```javascript
def deletePost(id: Int) {
  http: "DELETE"
  url: "https://api.example.com/posts/${id}"
}
```

### PATCH Requests

Partial updates.

```javascript
def updateEmail(userId: Int, newEmail: String) {
  http: "PATCH"
  url: "https://api.example.com/users/${userId}"
  body: {
    "email": "${newEmail}"
  }
}
```

## Function Properties

### URL with Interpolation

```javascript
def getResource(category: String, id: Int) {
  http: "GET"
  url: "https://api.example.com/${category}/${id}"
}

getResource("posts", 42)
// GET https://api.example.com/posts/42
```

### Headers

```javascript
def authenticatedRequest(token: String) {
  http: "GET"
  url: "https://api.example.com/protected"
  headers: {
    "Authorization": "Bearer ${token}",
    "Content-Type": "application/json"
  }
}
```

### Query Parameters

```javascript
def searchPosts(query: String, limit: Int) {
  http: "GET"
  url: "https://api.example.com/posts"
  params: {
    "q": "${query}",
    "limit": "${limit}"
  }
}

// GET https://api.example.com/posts?q=ai&limit=10
```

### Request Body

```javascript
def createUser(name: String, email: String, age: Int) {
  http: "POST"
  url: "https://api.example.com/users"
  body: {
    "name": "${name}",
    "email": "${email}",
    "age": ${age}
  }
}
```

## Response Handling

### Automatic JSON Parsing

```javascript
getUser(1)
// { "id": 1, "name": "John", "email": "john@example.com" }

getUser(1) as user
user.name           // "John"
user.email          // "john@example.com"
```

### Nested Field Access

```javascript
getUser(1) as user
user.address.city       // Access nested fields
user.address.zipCode
```

### Lists

```javascript
def getUsers() {
  http: "GET"
  url: "https://api.example.com/users"
}

getUsers() as users
users[0].name           // First user's name
Length(users)           // Number of users
```

## Integration Patterns

### HTTP + LLM

Fetch data, then analyze with AI:

```javascript
type Analysis { summary: String, insights: [String] }

def analyzeUserData(userId: Int) -> Analysis {
  http: "GET"
  url: "https://api.example.com/users/${userId}"

  model: "gpt-4o-mini"
  prompt: "Analyze this user profile and provide insights"
}

analyzeUserData(123)
// Fetches user data, then sends to LLM
```

### Sequential HTTP Calls

```javascript
getUser(1) as user
  >> getUserPosts(user.id) as posts
  >> getPostComments(posts[0].id)
```

### Parallel HTTP Calls

```javascript
(getUser(1) || getUser(2) || getUser(3)) as users
// Fetches 3 users concurrently

(
  getTodo(1) ||
  getTodo(2) ||
  getTodo(3)
) as [todo1, todo2, todo3]
```

### HTTP + SQL

```javascript
def getUserAnalytics() {
  http: "GET"
  url: "https://api.example.com/analytics/users"

  sql: """
    SELECT
      region,
      COUNT(*) as user_count,
      AVG(activity_score) as avg_activity
    FROM _
    GROUP BY region
  """
}
```

## Error Handling

### Network Errors

```javascript
getUser(999)
// ✗ Error: HTTP request failed: Connection refused
```

### HTTP Status Errors

```javascript
getUser(99999)
// ✗ Error: HTTP 404 Not Found
```

### Timeout

```javascript
// Request takes >30 seconds
slowEndpoint()
// ✗ Error: Request timeout
```

### Invalid JSON

```javascript
// Server returns invalid JSON
badEndpoint()
// Falls back to string response
```

## Performance Tips

### 1. Use Parallel Requests

```javascript
// Slow (sequential)
getUser(1) >> getUser(2) >> getUser(3)

// Fast (parallel)
(getUser(1) || getUser(2) || getUser(3))
```

### 2. Request Only Needed Data

```javascript
// Good: Specific fields
def getBasicUser(id: Int) {
  http: "GET"
  url: "https://api.example.com/users/${id}?fields=name,email"
}

// Bad: Request everything
def getUser(id: Int) {
  http: "GET"
  url: "https://api.example.com/users/${id}"
}
```

### 3. Batch When Possible

```javascript
// Good: Single batch request
def getUsers(ids: [Int]) {
  http: "POST"
  url: "https://api.example.com/users/batch"
  body: { "ids": ${ids} }
}

// Bad: Multiple requests
(getUser(1) || getUser(2) || getUser(3))
```

## Best Practices

### 1. Type Return Values

```javascript
type User { id: Int, name: String, email: String }

def getUser(id: Int) -> User {
  http: "GET"
  url: "https://api.example.com/users/${id}"
}
```

### 2. Handle Authentication

```javascript
// Environment variable
def authenticatedGet(path: String) {
  http: "GET"
  url: "https://api.example.com/${path}"
  headers: {
    "Authorization": "Bearer ${API_TOKEN}"
  }
}
```

### 3. Use Meaningful Function Names

```javascript
// Good
def fetchUserProfile(userId: Int)
def createBlogPost(title: String, content: String)
def updateUserSettings(userId: Int, settings: Map)

// Bad
def get(id: Int)
def post(data: Map)
def update(x: Int, y: Map)
```

## Common API Patterns

### REST API

```javascript
// CRUD operations
def getResource(id: Int) {
  http: "GET"
  url: "https://api.example.com/resources/${id}"
}

def createResource(data: Map) {
  http: "POST"
  url: "https://api.example.com/resources"
  body: ${data}
}

def updateResource(id: Int, data: Map) {
  http: "PUT"
  url: "https://api.example.com/resources/${id}"
  body: ${data}
}

def deleteResource(id: Int) {
  http: "DELETE"
  url: "https://api.example.com/resources/${id}"
}
```

### Pagination

```javascript
def getPage(pageNum: Int, pageSize: Int) {
  http: "GET"
  url: "https://api.example.com/items"
  params: {
    "page": "${pageNum}",
    "size": "${pageSize}"
  }
}

// Fetch multiple pages in parallel
(getPage(1, 10) || getPage(2, 10) || getPage(3, 10))
```

### Search API

```javascript
def search(query: String, filters: Map) {
  http: "GET"
  url: "https://api.example.com/search"
  params: {
    "q": "${query}",
    "filter": "${filters}"
  }
}
```

## Testing APIs

### Public Test APIs

```javascript
// JSONPlaceholder
def getTodo(id: Int) {
  http: "GET"
  url: "https://jsonplaceholder.typicode.com/todos/${id}"
}

// httpbin
def testPost(data: String) {
  http: "POST"
  url: "https://httpbin.org/post"
  body: { "data": "${data}" }
}
```

## Limitations

### Current Limitations

1. **30-second timeout** - Cannot be changed
2. **No streaming** - Waits for complete response
3. **No file upload** - Cannot send multipart/form-data
4. **No cookie management** - No persistent session
5. **No redirect control** - Follows redirects automatically

## Next Steps

- **[06-Functions.md](06-Functions.md)** - Define HTTP functions
- **[07-Workflow-Constructs.md](07-Workflow-Constructs.md)** - HTTP workflows
- **[examples/](../examples/)** - HTTP examples

## Related Documents

- **[HTTP_QUICK_START.md](../HTTP_QUICK_START.md)** - Quick reference
- **[HTTP_CLIENT_SUMMARY.md](../HTTP_CLIENT_SUMMARY.md)** - Complete reference
- **[IMPLEMENTATION_COMPLETE.md](../IMPLEMENTATION_COMPLETE.md)** - Implementation details
- **[REFACTORING_SUMMARY.md](../REFACTORING_SUMMARY.md)** - Architecture details
