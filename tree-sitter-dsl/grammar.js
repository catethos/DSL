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
    [$.def_parameter, $.pattern_variable],
    [$.primary, $.type_instantiation],
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
      optional(field('body', choice($.function_body, $.expr))),
      '}',
    ),

    def_parameters: $ => seq(
      $.def_parameter,
      repeat(seq(',', $.def_parameter)),
    ),

    def_parameter: $ => choice(
      seq(field('name', $.identifier), ':', field('type', $.field_type)),
      $.pattern,
      $.identifier,
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
    // Pattern Matching
    // ============================================

    pattern: $ => choice(
      $.pattern_literal,
      $.pattern_wildcard,
      $.pattern_binding,
      $.pattern_list,
      $.pattern_map,
      $.pattern_tuple,
      $.pattern_type,
      $.pattern_variable,
    ),

    pattern_literal: $ => choice(
      $.boolean,
      $.float,
      $.integer,
      $.string_literal,
    ),

    pattern_wildcard: _ => '_',

    pattern_binding: $ => seq(
      field('name', $.identifier),
      '@',
      field('pattern', $.pattern),
    ),

    pattern_list: $ => choice(
      seq('[', ']'),
      seq('[', $.pattern, repeat(seq(',', $.pattern)), optional(seq(',', '...', $.identifier)), optional(','), ']'),
      seq('[', '...', $.identifier, ']'),
    ),

    pattern_map: $ => choice(
      seq('{', '}'),
      seq('{', $.pattern_map_field, repeat(seq(',', $.pattern_map_field)), optional(','), '}'),
    ),

    pattern_map_field: $ => seq(
      field('name', $.identifier),
      optional(seq(':', $.pattern)),
    ),

    pattern_tuple: $ => seq(
      '(',
      $.pattern,
      repeat1(seq(',', $.pattern)),
      optional(','),
      ')',
    ),

    pattern_type: $ => seq(
      field('type', $.identifier),
      '(',
      field('inner', $.pattern),
      ')',
    ),

    pattern_variable: $ => $.identifier,

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

    expr: $ => choice(
      $.let_binding,
      $.conditional,
    ),

    let_binding: $ => seq(
      'let',
      choice(
        seq('[', $.identifier, repeat(seq(',', $.identifier)), ']'),
        $.identifier
      ),
      '=',
      $.conditional,
    ),

    conditional: $ => prec.right(1, seq(
      $.sequential,
      optional(seq('?', $.sequential, ':', $.sequential)),
    )),

    sequential: $ => prec.left(2, seq(
      $.logical_or,
      optional($.binding),
      repeat(seq('|>', $.logical_or, optional($.binding))),
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

    logical_or: $ => prec.left(3, seq(
      $.logical_and,
      repeat(seq('||', $.logical_and)),
    )),

    logical_and: $ => prec.left(4, seq(
      $.comparison,
      repeat(seq('&&', $.comparison)),
    )),

    comparison: $ => prec.left(5, seq(
      $.additive,
      optional(seq(
        field('op', choice('==', '!=', '<=', '>=', '<', '>')),
        $.additive,
      )),
    )),

    // Arithmetic expressions
    additive: $ => prec.left(6, choice(
      seq($.additive, choice('+', '-'), $.multiplicative),
      $.multiplicative,
    )),

    multiplicative: $ => prec.left(7, choice(
      seq($.multiplicative, choice('*', '/'), $.unary),
      $.unary,
    )),

    unary: $ => prec.right(8, choice(
      seq(choice('!', '-'), $.unary),
      $.primary,
    )),

    primary: $ => choice(
      $.match_expr,
      $.paren_expr,
      $.type_instantiation,
      $.function_call,
      $.access_chain,
      $.block,
      $.literal,
      $.identifier,
    ),

    match_expr: $ => seq(
      'match',
      field('scrutinee', $.expr),
      '{',
      $.match_case,
      repeat(seq(',', $.match_case)),
      optional(','),
      '}',
    ),

    match_case: $ => seq(
      field('pattern', $.pattern),
      optional(seq('if', field('guard', $.expr))),
      '=>',
      field('body', $.expr),
    ),

    type_instantiation: $ => seq(
      field('type', $.identifier),
      '{',
      optional(seq(
        $.type_field_assignment,
        repeat(seq(',', $.type_field_assignment)),
        optional(','),
      )),
      '}',
    ),

    type_field_assignment: $ => seq(
      field('field', $.identifier),
      ':',
      field('value', $.expr),
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
