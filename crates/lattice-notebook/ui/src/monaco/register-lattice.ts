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

  isRegistered = true
}
