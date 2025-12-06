import type { Monaco } from '@monaco-editor/react'
import { latticeLanguageConfig, latticeMonarchLanguage } from './lattice-language'

let isRegistered = false

export function registerLatticeLanguage(monaco: Monaco): void {
  if (isRegistered) {
    return
  }

  monaco.languages.register({
    id: 'lattice',
    extensions: ['.lattice', '.lat'],
    aliases: ['Lattice', 'lattice'],
    mimetypes: ['text/x-lattice'],
  })

  monaco.languages.setLanguageConfiguration('lattice', latticeLanguageConfig)
  monaco.languages.setMonarchTokensProvider('lattice', latticeMonarchLanguage)

  // Define custom theme - "Midnight Terminal" aesthetic
  // Inspired by Tokyo Night / Catppuccin - refined, cohesive, easy on the eyes
  monaco.editor.defineTheme('lattice-dark', {
    base: 'vs-dark',
    inherit: true,
    rules: [
      // === Lattice Language Tokens ===
      // Keywords: soft purple-pink (def, let, if, else, return, match)
      { token: 'keyword.lattice', foreground: 'BB9AF7' },
      // Types: muted teal (String, Int, Bool, Result)
      { token: 'type.identifier.lattice', foreground: '7DCFFF' },
      // Strings: warm peach
      { token: 'string.lattice', foreground: 'E0AF68' },
      { token: 'string.quote.lattice', foreground: 'E0AF68' },
      // Numbers: soft orange
      { token: 'number.lattice', foreground: 'FF9E64' },
      { token: 'number.float.lattice', foreground: 'FF9E64' },
      // Comments: muted slate
      { token: 'comment.lattice', foreground: '565F89', fontStyle: 'italic' },
      // Operators: soft gray
      { token: 'operator.lattice', foreground: '89DDFF' },
      // Annotations: golden yellow
      { token: 'annotation.lattice', foreground: 'E0AF68', fontStyle: 'italic' },
      // Identifiers: soft white
      { token: 'identifier.lattice', foreground: 'C0CAF5' },
      // Brackets: muted blue-gray
      { token: 'delimiter.bracket.lattice', foreground: '7AA2F7' },

      // === SQL Embedded Tokens ===
      // SQL Keywords: bright cyan-blue, bold for emphasis
      { token: 'keyword.sql', foreground: '7AA2F7', fontStyle: 'bold' },
      // SQL Operators: cyan
      { token: 'operator.sql', foreground: '89DDFF' },
      // SQL Strings: green (distinct from Lattice strings)
      { token: 'string.sql', foreground: '9ECE6A' },
      { token: 'string.double.sql', foreground: '9ECE6A' },
      // SQL Numbers: soft orange (consistent)
      { token: 'number.sql', foreground: 'FF9E64' },
      // SQL Identifiers (columns, tables): soft lavender
      { token: 'identifier.sql', foreground: 'C0CAF5' },
      // SQL Functions (ROUND, COUNT, etc.): teal
      { token: 'predefined.sql', foreground: '7DCFFF' },
      // SQL Comments: muted slate
      { token: 'comment.sql', foreground: '565F89', fontStyle: 'italic' },
    ],
    colors: {
      // Rich dark background with subtle blue undertone
      'editor.background': '#1A1B26',
      // Slightly lighter for current line
      'editor.lineHighlightBackground': '#24283B',
      // Selection with transparency
      'editor.selectionBackground': '#364A82',
      // Gutter/line numbers
      'editorLineNumber.foreground': '#3B4261',
      'editorLineNumber.activeForeground': '#737AA2',
      // Cursor
      'editorCursor.foreground': '#C0CAF5',
      // Matching brackets
      'editorBracketMatch.background': '#3B4261',
      'editorBracketMatch.border': '#7AA2F7',
      // Indent guides
      'editorIndentGuide.background': '#292E42',
      'editorIndentGuide.activeBackground': '#3B4261',
      // Scrollbar
      'scrollbarSlider.background': '#292E4280',
      'scrollbarSlider.hoverBackground': '#3B426180',
    },
  })

  isRegistered = true
}
