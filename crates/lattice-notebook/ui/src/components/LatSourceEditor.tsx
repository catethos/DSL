import Editor, { BeforeMount } from '@monaco-editor/react'
import { registerLatticeLanguage } from '../monaco'

interface LatSourceEditorProps {
  content: string
  onChange: (value: string) => void
  isDirty?: boolean
  fileName?: string
}

export function LatSourceEditor({
  content,
  onChange,
  isDirty = false,
  fileName = 'Untitled',
}: LatSourceEditorProps) {
  const handleBeforeMount: BeforeMount = (monaco) => {
    registerLatticeLanguage(monaco)
  }

  return (
    <div className="lat-source-editor">
      <div className="lat-editor-header">
        <div className="lat-editor-title">
          <span className="lat-icon">lat</span>
          <span className="lat-file-name">{fileName}</span>
          {isDirty && <span className="dirty-indicator">*</span>}
        </div>
      </div>
      <div className="lat-editor-container">
        <Editor
          height="100%"
          defaultLanguage="rust" // Use rust for syntax highlighting (similar to lattice)
          theme="lattice-dark"
          value={content}
          onChange={(value) => onChange(value ?? '')}
          beforeMount={handleBeforeMount}
          options={{
            minimap: { enabled: false },
            fontSize: 14,
            lineNumbers: 'on',
            scrollBeyondLastLine: false,
            automaticLayout: true,
            tabSize: 4,
            wordWrap: 'on',
            folding: true,
            lineDecorationsWidth: 10,
            lineNumbersMinChars: 3,
            renderLineHighlight: 'line',
            scrollbar: {
              vertical: 'auto',
              horizontal: 'auto',
              alwaysConsumeMouseWheel: false,
            },
          }}
        />
      </div>
    </div>
  )
}
