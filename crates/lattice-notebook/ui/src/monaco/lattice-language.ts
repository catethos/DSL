import type { languages } from 'monaco-editor'

export const latticeLanguageConfig: languages.LanguageConfiguration = {
  comments: {
    lineComment: '//',
    blockComment: ['/*', '*/'],
  },
  brackets: [
    ['{', '}'],
    ['[', ']'],
    ['(', ')'],
  ],
  autoClosingPairs: [
    { open: '{', close: '}' },
    { open: '[', close: ']' },
    { open: '(', close: ')' },
    { open: '"', close: '"', notIn: ['string'] },
  ],
  surroundingPairs: [
    { open: '{', close: '}' },
    { open: '[', close: ']' },
    { open: '(', close: ')' },
    { open: '"', close: '"' },
  ],
  folding: {
    markers: {
      start: /^\s*\/\/\s*#?region\b/,
      end: /^\s*\/\/\s*#?endregion\b/,
    },
  },
}

export const latticeMonarchLanguage: languages.IMonarchLanguage = {
  defaultToken: '',
  tokenPostfix: '.lattice',

  keywords: [
    'type', 'enum', 'def', 'let', 'if', 'else', 'while', 'for', 'in',
    'return', 'match', 'true', 'false', 'null', 'parallel', 'parallel_map',
    'SQL', 'Ok', 'Err',
  ],

  typeKeywords: [
    'String', 'Int', 'Float', 'Bool', 'Null', 'Path', 'Map', 'Result',
  ],

  configKeys: [
    'base_url', 'model', 'api_key_env', 'temperature', 'max_tokens',
  ],

  operators: [
    '=', '>', '<', '!', '~', '?', ':', '==', '<=', '>=', '!=',
    '&&', '||', '+', '-', '*', '/', '%', '->', '=>', '::',
  ],

  symbols: /[=><!~?:&|+\-*\/\^%]+/,

  escapes: /\\(?:[btnfr\\"']|x[0-9A-Fa-f]{2}|u[0-9A-Fa-f]{4})/,

  tokenizer: {
    root: [
      { include: '@whitespace' },

      // Config key: prompt
      [/(prompt)(\s*)(:)/, [
        'keyword',
        'white',
        'delimiter',
      ]],

      // Other config keys
      [/(base_url|model|api_key_env|temperature|max_tokens)(\s*)(:)/, [
        'keyword',
        'white',
        'delimiter',
      ]],

      // Enum constructor Pattern::Variant
      [/[A-Z][a-zA-Z0-9_]*::/, 'type.identifier'],

      // Keywords and identifiers
      [/[a-zA-Z_][a-zA-Z0-9_]*/, {
        cases: {
          '@keywords': 'keyword',
          '@typeKeywords': 'type.identifier',
          '@default': 'identifier',
        },
      }],

      // Numbers
      [/\d*\.\d+([eE][\-+]?\d+)?/, 'number.float'],
      [/\d+/, 'number'],

      // Strings
      [/"""/, { token: 'string.quote', next: '@rawstring' }],
      [/f"/, { token: 'string.quote', next: '@fstring' }],
      [/"([^"\\]|\\.)*$/, 'string.invalid'],
      [/"/, { token: 'string.quote', next: '@string' }],

      // Delimiters and operators
      [/[{}()\[\]]/, '@brackets'],
      [/[<>](?!@symbols)/, '@brackets'],
      [/@symbols/, {
        cases: {
          '@operators': 'operator',
          '@default': '',
        },
      }],

      [/[;,.]/, 'delimiter'],

      // Field descriptions
      [/@"/, { token: 'annotation', next: '@annotation' }],
      [/@/, 'annotation'],
    ],

    whitespace: [
      [/[ \t\r\n]+/, 'white'],
      [/\/\*/, 'comment', '@comment'],
      [/\/\/.*$/, 'comment'],
    ],

    comment: [
      [/[^\/*]+/, 'comment'],
      [/\*\//, 'comment', '@pop'],
      [/[\/*]/, 'comment'],
    ],

    string: [
      [/[^\\"]+/, 'string'],
      [/@escapes/, 'string.escape'],
      [/\\./, 'string.escape.invalid'],
      [/"/, { token: 'string.quote', next: '@pop' }],
    ],

    rawstring: [
      [/[^"]+/, 'string'],
      [/"""/, { token: 'string.quote', next: '@pop' }],
      [/"/, 'string'],
    ],

    fstring: [
      [/\{/, { token: 'delimiter.bracket', next: '@fstringExpr' }],
      [/[^\\"{}]+/, 'string'],
      [/@escapes/, 'string.escape'],
      [/\\./, 'string.escape.invalid'],
      [/"/, { token: 'string.quote', next: '@pop' }],
    ],

    fstringExpr: [
      [/\}/, { token: 'delimiter.bracket', next: '@pop' }],
      { include: 'root' },
    ],

    annotation: [
      [/[^"]+/, 'annotation'],
      [/"/, { token: 'annotation', next: '@pop' }],
    ],
  },
}
