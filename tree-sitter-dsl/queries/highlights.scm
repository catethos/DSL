; Keywords
[
  "type"
  "enum"
  "def"
  "workflow"
  "as"
  "prompt"
  "sql"
  "http"
  "url"
  "params"
  "body"
  "headers"
] @keyword

; Primitive types
[
  "String" "string"
  "Int" "int"
  "Float" "float"
  "Bool" "bool"
  "Table" "table"
  "Any" "any"
] @type.builtin

; Custom types
(custom_type) @type

; Function definitions
(function_decl
  name: (identifier) @function)

(workflow_decl
  name: (identifier) @function)

; Function calls
(function_call
  function: (identifier) @function.call)

; Type declarations
(type_decl
  name: (identifier) @type.definition)

(enum_decl
  name: (identifier) @type.definition)

; Enum variants
(enum_variant) @constant

; Field names
(field
  name: (identifier) @property)

; Parameters
(def_parameter
  name: (identifier) @variable.parameter)

; Properties
(property
  key: (identifier) @property)

; Map entries
(map_entry
  key: (identifier) @property)

; Operators
[
  ">>"
  "||"
  "?"
  ":"
  "->"
  ":="
  "+"
  "-"
  "*"
  "/"
] @operator

; Delimiters
[
  "{"
  "}"
  "["
  "]"
  "("
  ")"
] @punctuation.bracket

[
  ","
] @punctuation.delimiter

; Literals
(string_literal) @string
(triple_quoted_string) @string
(template_string) @string
(template_text) @string
(template_expr) @embedded

(integer) @number
(float) @number
(boolean) @constant.builtin

; Special blocks
(prompt_block
  "prompt" @keyword.special)

(sql_block
  "sql" @keyword.special)

(http_block
  "http" @keyword.special)

; Comments
(comment) @comment

; Identifiers (catch-all)
(identifier) @variable
