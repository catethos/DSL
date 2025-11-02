/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports = grammar({
  name: 'dsl',

  extras: $ => [
    /\s/,
    $.comment,
  ],

  word: $ => $.identifier,

  conflicts: $ => [
    [$.block, $.map_literal],
    [$.property, $.map_entry],
  ],

  rules: {
    // ============================================
    // Program Structure
    // ============================================

    program: $ => repeat($.declaration),

    declaration: $ => choice(
      $.type_decl,
      $.enum_decl,
      $.function_decl,
      $.workflow_decl,
    ),

    // ============================================
    // Type Definitions
    // ============================================

    type_decl: $ => seq(
      'type',
      field('name', $.identifier),
      '{',
      repeat(seq($.field, optional(','))),
      '}',
    ),

    field: $ => seq(
      field('name', $.identifier),
      optional('?'),
      ':',
      field('type', $.field_type),
      optional(field('description', $.string_literal)),
    ),

    field_type: $ => choice(
      $.list_type,
      $.primitive_type,
      $.custom_type,
    ),

    primitive_type: _ => choice(
      'String', 'string',
      'Int', 'int',
      'Float', 'float',
      'Bool', 'bool',
      'Table', 'table',
      'Any', 'any',
    ),

    list_type: $ => seq('[', $.field_type, ']'),

    custom_type: $ => $.identifier,

    enum_decl: $ => seq(
      'enum',
      field('name', $.identifier),
      '{',
      seq(
        $.enum_variant,
        repeat(seq(',', $.enum_variant)),
        optional(','),
      ),
      '}',
    ),

    enum_variant: $ => $.identifier,

    // ============================================
    // Function Definitions
    // ============================================

    function_decl: $ => seq(
      'def',
      field('name', $.identifier),
      '(',
      optional(field('params', $.def_parameters)),
      ')',
      optional(seq('->', field('return_type', $.field_type))),
      '{',
      optional(field('body', $.function_body)),
      '}',
    ),

    def_parameters: $ => seq(
      $.def_parameter,
      repeat(seq(',', $.def_parameter)),
    ),

    def_parameter: $ => seq(
      field('name', $.identifier),
      optional(seq(':', field('type', $.field_type))),
    ),

    function_body: $ => repeat1(choice(
      $.prompt_block,
      $.sql_block,
      $.http_block,
      $.property,
    )),

    property: $ => seq(
      field('key', $.identifier),
      ':',
      field('value', $.value),
    ),

    prompt_block: $ => seq(
      'prompt',
      ':',
      choice(
        $.template_string,
        $.triple_quoted_string,
        $.string_literal,
      ),
    ),

    sql_block: $ => seq(
      'sql',
      ':',
      choice(
        $.template_string,
        $.triple_quoted_string,
        $.string_literal,
      ),
    ),

    http_block: $ => seq(
      'http',
      ':',
      $.string_literal,
      optional(seq('url', ':', $.string_literal)),
      optional(seq('params', ':', $.map_literal)),
      optional(seq('body', ':', $.map_literal)),
      optional(seq('headers', ':', $.map_literal)),
    ),

    // Template strings with ${} interpolation
    template_string: $ => seq(
      '"',
      repeat(choice(
        $.template_expr,
        $.template_text,
        $.escape_sequence,
      )),
      '"',
    ),

    template_text: _ => token.immediate(prec(1, /[^"$\\]+/)),

    escape_sequence: _ => token.immediate(prec(2, /\\["\\/bfnrt$]/)),

    template_expr: $ => seq(
      '${',
      field('expression', /[^}]+/),
      '}',
    ),

    // ============================================
    // Workflow Definitions
    // ============================================

    workflow_decl: $ => seq(
      'workflow',
      field('name', $.identifier),
      '(',
      optional(field('params', $.def_parameters)),
      ')',
      optional(seq('->', field('return_type', $.field_type))),
      ':=',
      field('body', $.expr),
    ),

    // ============================================
    // Expressions
    // ============================================

    expr: $ => $.conditional,

    conditional: $ => prec.right(1, seq(
      $.sequential,
      optional(seq('?', $.sequential, ':', $.sequential)),
    )),

    sequential: $ => prec.left(2, seq(
      $.parallel,
      optional($.binding),
      repeat(seq('>>', $.parallel, optional($.binding))),
    )),

    parallel: $ => prec.left(3, seq(
      $.additive,
      repeat(seq('||', $.additive)),
      optional($.binding),
    )),

    binding: $ => seq(
      'as',
      choice(
        $.list_binding,
        $.identifier,
      ),
    ),

    list_binding: $ => seq(
      '[',
      $.identifier,
      repeat(seq(',', $.identifier)),
      ']',
    ),

    // Arithmetic expressions
    additive: $ => prec.left(4, choice(
      seq($.additive, choice('+', '-'), $.multiplicative),
      $.multiplicative,
    )),

    multiplicative: $ => prec.left(5, choice(
      seq($.multiplicative, choice('*', '/'), $.primary),
      $.primary,
    )),

    primary: $ => choice(
      $.paren_expr,
      $.function_call,
      $.access_chain,
      $.block,
      $.literal,
      $.identifier,
    ),

    paren_expr: $ => seq('(', $.expr, ')'),

    function_call: $ => seq(
      field('function', $.identifier),
      '(',
      optional(field('args', $.arguments)),
      ')',
    ),

    arguments: $ => seq(
      $.expr,
      repeat(seq(',', $.expr)),
    ),

    // Field/index access
    access_chain: $ => prec.left(6, seq(
      $.identifier,
      repeat1(choice(
        $.field_access,
        $.index_access,
      )),
    )),

    field_access: $ => seq('.', field('field', $.identifier)),

    index_access: $ => seq('[', field('index', $.expr), ']'),

    // Inline anonymous block
    block: $ => seq(
      '{',
      repeat($.property),
      optional(choice($.sql_block, $.prompt_block)),
      '}',
    ),

    // ============================================
    // Literals
    // ============================================

    literal: $ => choice(
      $.boolean,
      $.float,
      $.integer,
      $.template_string,
      $.string_literal,
      $.list_literal,
      $.map_literal,
    ),

    string_literal: _ => token(choice(
      seq('"', /([^"\\]|\\["\\/bfnrt]|\\u[0-9a-fA-F]{4})*/, '"'),
      seq("'", /([^'\\]|\\['\\/bfnrt])*/, "'"),
    )),

    triple_quoted_string: _ => token(seq('"""', /[^"]*(?:"[^"]+|""[^"])*/, '"""')),

    integer: _ => /-?\d+/,

    float: _ => /-?\d+\.\d+/,

    boolean: _ => choice('true', 'false'),

    list_literal: $ => seq(
      '[',
      optional(seq(
        $.expr,
        repeat(seq(',', $.expr)),
        optional(','),
      )),
      ']',
    ),

    map_literal: $ => seq(
      '{',
      optional(seq(
        $.map_entry,
        repeat(seq(',', $.map_entry)),
        optional(','),
      )),
      '}',
    ),

    map_entry: $ => seq(
      field('key', choice($.identifier, $.string_literal)),
      ':',
      field('value', $.value),
    ),

    value: $ => choice(
      $.template_string,
      $.string_literal,
      $.float,
      $.integer,
      $.boolean,
      $.identifier,
      $.map_literal,
      $.list_literal,
    ),

    // ============================================
    // Basic Tokens
    // ============================================

    identifier: _ => /[a-zA-Z_][a-zA-Z0-9_]*/,

    comment: _ => token(choice(
      seq('#', /[^\n]*/),
      seq('/*', /[^*]*\*+([^/*][^*]*\*+)*/, '/'),
    )),
  },
});
