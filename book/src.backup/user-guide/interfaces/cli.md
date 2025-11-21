# Command Line Interface

The DSL Command Line Interface provides powerful tools for batch processing, automation, CI/CD integration, and script execution. Use it for headless execution, syntax checking, compilation, and performance profiling.

## Overview

The `dsl` CLI offers several modes:

- **Interactive TUI**: Full-featured terminal interface (default)
- **Non-Interactive Mode**: Batch execution from stdin/scripts
- **Check Mode**: Syntax validation without execution
- **IR Compilation**: Compile to Intermediate Representation
- **Run Mode**: Execute scripts with optional tracing
- **Update Mode**: Self-update to latest version

## Installation

```bash
# Install from source
cargo install dsl

# Or use pre-built binaries
curl -sSL https://dsl-lang.org/install.sh | sh

# Verify installation
dsl --version
```

## Command Syntax

```bash
dsl [OPTIONS] [COMMAND]
```

### Global Options

| Option | Description |
|--------|-------------|
| `--version` | Show version information |
| `--help` | Show help message |
| `-c, --stdin` | Read from stdin (non-interactive) |

## Commands

### Interactive Mode (Default)

Launch the TUI for interactive development:

```bash
# Start interactive TUI
dsl

# Equivalent
dsl tui
```

**Use When:**
- Developing interactively
- Experimenting with code
- Debugging workflows
- Learning the language

**See:** [Terminal UI (TUI)](./tui.md) for full documentation

### Non-Interactive Mode

Execute DSL code from stdin for batch processing:

```bash
# Execute from stdin
echo "5 + 3" | dsl -c
# Output: 8

# Execute multi-line program
cat <<EOF | dsl -c
let x = 10
let y = 20
x + y
EOF
# Output: 30

# From file
dsl -c < script.dsl

# With pipes
cat data.csv | dsl -c < process.dsl
```

**Features:**
- No interactive prompts
- Reads continuously from stdin
- Outputs results to stdout
- Errors to stderr
- Suitable for pipes and scripts

**Example Pipeline:**

```bash
#!/bin/bash
# process_data.sh

# Generate data | Process with DSL | Save output
generate_data | dsl -c <<'DSL' > output.json
  SQL("SELECT * FROM stdin")
    |> Filter(_, def (row) := row.value > 100)
    |> Map(_, def (row) := {id: row.id, value: row.value})
DSL
```

### `dsl check` - Syntax Validation

Check DSL files for syntax and type errors without executing:

```bash
# Check a single file
dsl check script.dsl

# Output
Checking script.dsl...
✓ No errors found

# With errors
dsl check invalid.dsl
Checking invalid.dsl...
✗ Compilation failed:
Error: Undefined variable: unknownVar
  at line 5, column 10
```

**Use Cases:**
- Pre-commit hooks
- CI/CD validation
- Editor integrations
- Quick syntax verification

**Exit Codes:**
- `0`: No errors
- `1`: Compilation errors found

**CI/CD Integration:**

```yaml
# .github/workflows/test.yml
name: Validate DSL Scripts

on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Install DSL
        run: curl -sSL https://dsl-lang.org/install.sh | sh
      - name: Check all scripts
        run: |
          for file in *.dsl; do
            dsl check "$file" || exit 1
          done
```

### `dsl ir` - IR Compilation

Compile DSL to Intermediate Representation (IR) for inspection or optimization:

```bash
# Compile to MessagePack (default)
dsl ir input.dsl -o output.ir

# Compile to JSON (human-readable)
dsl ir input.dsl -o output.json --json

# Example output
Parsing input.dsl...
Compiling to IR...
Saving IR to output.json...
✓ IR saved successfully
```

**IR Formats:**

**MessagePack (default):**
- Binary format
- Compact size
- Fast parsing
- Production use

**JSON (`--json`):**
- Human-readable
- Easy inspection
- Debugging
- Tooling integration

**IR Structure Example:**

```json
{
  "version": "1.0",
  "nodes": [
    {
      "type": "FunctionDef",
      "name": "processData",
      "params": ["data"],
      "body": {
        "type": "Pipeline",
        "steps": [...]
      }
    }
  ],
  "types": [...],
  "metadata": {
    "source_file": "input.dsl",
    "compiled_at": "2025-11-13T..."
  }
}
```

**Use Cases:**
- Inspect compilation output
- Debug optimization issues
- Build tooling (formatters, linters)
- Cross-platform distribution (compile once, run anywhere)

### `dsl run` - Execute Scripts

Run DSL scripts directly from files:

```bash
# Simple execution
dsl run script.dsl

# Output
Running script.dsl...
Compiling to IR...
Executing...
Result: 42
```

**Example Script:**

```dsl
// script.dsl
type User {
  id: Int
  name: String
}

def getUsers() :=
  SQL("SELECT * FROM 'users.csv'")
    |> Filter(_, def (u) := u.active)
    |> Map(_, def (u) := {id: u.id, name: u.name})

let users = getUsers()
let count = Length(users)

"Found " + ToString(count) + " active users"
```

**Execution:**

```bash
$ dsl run script.dsl
Running script.dsl...
Compiling to IR...
Executing...
Found 42 active users
```

**With Tracing:**

```bash
# Enable execution tracing
dsl run script.dsl --trace

# Save trace to file
dsl run script.dsl --trace --trace-output trace.json

# Verbose tracing
dsl run script.dsl --trace --trace-verbose

# Filter by node types
dsl run script.dsl --trace --trace-filter FunctionCall,Pipeline

# Only show slow operations (> 1000 microseconds)
dsl run script.dsl --trace --trace-min-duration 1000
```

**Tracing Options:**

| Flag | Description |
|------|-------------|
| `--trace` | Enable execution tracing |
| `--trace-output <file>` | Save trace to JSON file |
| `--trace-verbose` | Show detailed trace events |
| `--trace-filter <types>` | Filter by node types (comma-separated) |
| `--trace-min-duration <μs>` | Only show events slower than N microseconds |

**Trace Output Example:**

```json
{
  "total_duration_us": 1523,
  "events": [
    {
      "node_type": "FunctionCall",
      "name": "SQL",
      "duration_us": 1245,
      "start_time": "2025-11-13T...",
      "success": true
    },
    {
      "node_type": "Pipeline",
      "steps": 3,
      "duration_us": 245,
      "success": true
    }
  ]
}
```

**Use Cases:**
- Production execution
- Batch processing
- Scheduled jobs (cron)
- Performance profiling
- Debugging slow operations

### `dsl update` - Self-Update

Update DSL to the latest version:

```bash
# Check for updates
dsl update --check

# Output
New version available: 0.1.0 -> 0.2.0
Run 'dsl update' to install the update.

# Install update
dsl update

# Output
Downloading version 0.2.0...
Installing...
✓ Successfully updated to 0.2.0
```

**Automatic Update Checks:**

DSL automatically checks for updates periodically (in interactive mode only):

```
🔔 New version available: 0.1.0 -> 0.2.0
   Run 'dsl update' to install the update.
```

**Configuration:**

Update checks stored in `~/.dsl_config.json`:

```json
{
  "last_update_check": "2025-11-13T12:00:00Z",
  "update_check_interval_days": 7
}
```

## Scripting and Automation

### Shebang Support

Make DSL scripts executable:

```dsl
#!/usr/bin/env dsl run

type Task {
  name: String
  status: String
}

def main() :=
  SQL("SELECT * FROM 'tasks.csv'")
    |> Filter(_, def (t) := t.status == "pending")
    |> Map(_, def (t) := "- " + t.name)
    |> Join(_, "\n")

main()
```

**Make executable:**

```bash
chmod +x script.dsl
./script.dsl
```

### Environment Variables

Access environment variables in scripts:

```dsl
// Read API key from environment
let apiKey = GetEnv("API_KEY")

// With default
let apiKey = GetEnv("API_KEY") ?? "default-key"

// Check if set
let hasKey = GetEnv("API_KEY") != null
```

**Usage:**

```bash
# Set environment variable
export API_KEY="secret-key"

# Run script
dsl run workflow.dsl
```

### Exit Codes

DSL scripts return meaningful exit codes:

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | Compilation error |
| `2` | Runtime error |
| `3` | Type error |

**Example:**

```bash
#!/bin/bash

dsl run workflow.dsl
EXIT_CODE=$?

if [ $EXIT_CODE -eq 0 ]; then
  echo "Workflow completed successfully"
else
  echo "Workflow failed with code $EXIT_CODE"
  exit $EXIT_CODE
fi
```

### Standard Streams

DSL respects standard Unix streams:

**stdin**: Input data
**stdout**: Normal output
**stderr**: Errors and logging

```bash
# Input from file
dsl -c < input.dsl > output.txt 2> errors.log

# Pipe through DSL
cat data.csv | dsl -c <<'DSL' | process_output
  SQL("SELECT * FROM stdin")
    |> Filter(_, isValid)
DSL

# Suppress errors
dsl run script.dsl 2>/dev/null
```

## Batch Processing

### Processing Multiple Files

```bash
#!/bin/bash
# process_all.sh

for file in data/*.csv; do
  echo "Processing $file..."
  dsl -c <<DSL
    SQL("SELECT * FROM '$file'")
      |> Filter(_, def (row) := row.valid)
      |> Map(_, transform)
      |> SaveAs("output/$(basename $file)")
DSL
done
```

### Parallel Processing

```bash
#!/bin/bash
# parallel_process.sh

# GNU parallel
find data/*.csv | parallel dsl run process.dsl {}

# xargs
find data/*.csv | xargs -P 4 -I {} dsl run process.dsl {}
```

### Cron Jobs

```bash
# crontab -e

# Run daily at 2 AM
0 2 * * * /usr/local/bin/dsl run /path/to/daily_job.dsl >> /var/log/dsl.log 2>&1

# Run every hour
0 * * * * /usr/local/bin/dsl run /path/to/hourly_sync.dsl
```

## CI/CD Integration

### GitHub Actions

```yaml
name: DSL Workflows

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2

      - name: Install DSL
        run: |
          curl -sSL https://dsl-lang.org/install.sh | sh
          echo "$HOME/.dsl/bin" >> $GITHUB_PATH

      - name: Check syntax
        run: |
          for file in workflows/*.dsl; do
            dsl check "$file"
          done

      - name: Run tests
        run: dsl run tests/test_suite.dsl

      - name: Execute workflow
        run: dsl run workflows/main.dsl

  deploy:
    needs: validate
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v2

      - name: Deploy workflow
        env:
          API_KEY: ${{ secrets.API_KEY }}
        run: dsl run workflows/deploy.dsl
```

### GitLab CI

```yaml
# .gitlab-ci.yml
stages:
  - validate
  - test
  - deploy

variables:
  DSL_VERSION: "0.2.0"

before_script:
  - curl -sSL https://dsl-lang.org/install.sh | sh
  - export PATH="$HOME/.dsl/bin:$PATH"

validate:
  stage: validate
  script:
    - dsl check workflows/*.dsl

test:
  stage: test
  script:
    - dsl run tests/test_suite.dsl

deploy:
  stage: deploy
  only:
    - main
  script:
    - dsl run workflows/deploy.dsl
```

### Jenkins

```groovy
// Jenkinsfile
pipeline {
    agent any

    stages {
        stage('Install DSL') {
            steps {
                sh 'curl -sSL https://dsl-lang.org/install.sh | sh'
            }
        }

        stage('Validate') {
            steps {
                sh 'dsl check workflows/*.dsl'
            }
        }

        stage('Test') {
            steps {
                sh 'dsl run tests/test_suite.dsl'
            }
        }

        stage('Deploy') {
            when {
                branch 'main'
            }
            steps {
                withCredentials([string(credentialsId: 'api-key', variable: 'API_KEY')]) {
                    sh 'dsl run workflows/deploy.dsl'
                }
            }
        }
    }

    post {
        failure {
            echo 'Workflow failed'
        }
        success {
            echo 'Workflow succeeded'
        }
    }
}
```

## Performance Profiling

### Trace Analysis

Use tracing to identify bottlenecks:

```bash
# Run with trace
dsl run workflow.dsl --trace --trace-output trace.json

# Analyze trace
cat trace.json | jq '.events | sort_by(.duration_us) | reverse | .[0:10]'
```

**Example Analysis:**

```json
[
  {
    "node_type": "FunctionCall",
    "name": "SQL",
    "duration_us": 15234,
    "percentage": 85.3
  },
  {
    "node_type": "Pipeline",
    "steps": 5,
    "duration_us": 2145,
    "percentage": 12.0
  }
]
```

### Filtering Slow Operations

```bash
# Only show operations > 5ms
dsl run workflow.dsl --trace --trace-min-duration 5000

# Output
[TRACE] FunctionCall(SQL): 15234μs
[TRACE] FunctionCall(ProcessLargeData): 8567μs
```

### Node Type Filtering

```bash
# Only trace function calls
dsl run workflow.dsl --trace --trace-filter FunctionCall

# Multiple types
dsl run workflow.dsl --trace --trace-filter FunctionCall,Pipeline,BinaryOp
```

## Examples

### Example 1: Data Processing Script

```dsl
#!/usr/bin/env dsl run
// process_sales.dsl

type Sale {
  id: Int
  amount: Float
  date: String
  category: String
}

def processSales(filename) :=
  SQL("SELECT * FROM '" + filename + "'")
    |> Filter(_, def (s) := s.amount > 0)
    |> Map(_, def (s) := {
         category: s.category,
         amount: s.amount,
         month: Split(s.date, "-")[1]
       })
    |> GroupBy(_, "category")
    |> Map(_, def (group) := {
         category: group.key,
         total: Sum(Map(group.values, def (s) := s.amount)),
         count: Length(group.values)
       })

let results = processSales("sales.csv")

"Processed " + ToString(Length(results)) + " categories"
```

**Run:**

```bash
$ dsl run process_sales.dsl
Running process_sales.dsl...
Compiling to IR...
Executing...
Processed 5 categories
```

### Example 2: Automated Testing

```dsl
#!/usr/bin/env dsl run
// test_suite.dsl

def assert(condition, message) :=
  condition
    ? "✓ " + message
    : Error("✗ " + message)

def testStringFunctions() :=
  [
    assert(Upper("hello") == "HELLO", "Upper works"),
    assert(Lower("WORLD") == "world", "Lower works"),
    assert(Length("test") == 4, "Length works")
  ]

def testListFunctions() :=
  [
    assert(Length([1, 2, 3]) == 3, "List length"),
    assert(First([1, 2, 3]) == 1, "First element"),
    assert(Last([1, 2, 3]) == 3, "Last element")
  ]

def runTests() :=
  [
    ...testStringFunctions(),
    ...testListFunctions()
  ] |> Join(_, "\n")

runTests()
```

**Run:**

```bash
$ dsl run test_suite.dsl
✓ Upper works
✓ Lower works
✓ Length works
✓ List length
✓ First element
✓ Last element
```

### Example 3: CI/CD Deployment

```dsl
#!/usr/bin/env dsl run
// deploy.dsl

type DeployConfig {
  environment: String
  version: String
  apiEndpoint: String
}

def deploy(config) :=
  config
    |> validateConfig(_) as validated
    |> buildArtifact(validated) as artifact
    |> uploadToServer(artifact, config.apiEndpoint) as result
    |> notifySlack("Deployed " + config.version + " to " + config.environment)
    |> result

def validateConfig(config) :=
  All([
    Length(config.environment) > 0,
    Length(config.version) > 0,
    Contains(config.apiEndpoint, "https://")
  ])
  ? config
  : Error("Invalid configuration")

let config = {
  environment: GetEnv("DEPLOY_ENV") ?? "staging",
  version: GetEnv("VERSION") ?? "latest",
  apiEndpoint: GetEnv("API_ENDPOINT") ?? "https://api.example.com"
}

deploy(config)
```

**Run:**

```bash
$ export DEPLOY_ENV="production"
$ export VERSION="1.2.3"
$ export API_ENDPOINT="https://api.prod.example.com"
$ dsl run deploy.dsl
```

## Troubleshooting

### Command Not Found

**Problem:** `dsl: command not found`

**Solutions:**
- Ensure DSL is installed: `which dsl`
- Add to PATH: `export PATH="$HOME/.dsl/bin:$PATH"`
- Reinstall: `curl -sSL https://dsl-lang.org/install.sh | sh`

### Permission Denied

**Problem:** `Permission denied` when running scripts

**Solutions:**
- Make executable: `chmod +x script.dsl`
- Check shebang: `#!/usr/bin/env dsl run`
- Run with dsl: `dsl run script.dsl`

### Stdin Not Working

**Problem:** `dsl -c` hangs or doesn't read input

**Solutions:**
- Verify piping: `echo "5 + 3" | dsl -c`
- Check for interactive prompts (not supported in `-c` mode)
- Use heredoc for multi-line: `dsl -c <<EOF ... EOF`

### Trace File Too Large

**Problem:** Trace output is huge

**Solutions:**
- Filter by type: `--trace-filter FunctionCall`
- Set minimum duration: `--trace-min-duration 1000`
- Disable verbose: remove `--trace-verbose`

## Next Steps

- [Terminal UI (TUI)](./tui.md) - Interactive terminal interface
- [Desktop GUI (egui)](./egui.md) - Visual development environment
- [Workflows](../workflows/sequential.md) - Building complex workflows
- [Builtin Functions](../builtins/README.md) - Available functions
