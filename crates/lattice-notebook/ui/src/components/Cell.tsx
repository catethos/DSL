import { useState, useRef, useEffect, forwardRef, useImperativeHandle } from 'react'
import Editor, { OnMount } from '@monaco-editor/react'
import type { editor } from 'monaco-editor'
import ReactMarkdown from 'react-markdown'
import { PaginatedTable } from './PaginatedTable'
import { registerLatticeLanguage } from '../monaco'

export interface CellHandle {
  focus: () => void
}

// Matches the Rust CellOutput enum
export type CellOutput =
  | { type: 'Empty' }
  | { type: 'Text'; data: string }
  | { type: 'Table'; data: { headers: string[]; rows: string[][] } }

// Matches the Rust LlmDebugOutput struct
export interface LlmDebugOutput {
  function_name: string
  return_type: string
  prompt: string
  raw_response: string
}

// Matches the Rust EvalResponse struct
export interface EvalResponse {
  output: CellOutput
  llm_debug?: LlmDebugOutput
}

interface CellProps {
  id: string
  initialCode?: string
  initialDescription?: string
  output?: CellOutput | null
  llmDebug?: LlmDebugOutput | null
  error?: string | null
  isRunning?: boolean
  onRun: (id: string, code: string) => void
  onRunAndAddCell?: (id: string, code: string) => void
  onDelete?: (id: string) => void
  onCodeChange?: (id: string, code: string) => void
  onDescriptionChange?: (id: string, description: string) => void
  shouldFocus?: boolean
  onFocused?: () => void
}

const MIN_HEIGHT = 80
const DEFAULT_HEIGHT = 120

export const Cell = forwardRef<CellHandle, CellProps>(function Cell(
  {
    id,
    initialCode = '',
    initialDescription = '',
    output = null,
    llmDebug = null,
    error = null,
    isRunning = false,
    onRun,
    onRunAndAddCell,
    onDelete,
    onCodeChange,
    onDescriptionChange,
    shouldFocus = false,
    onFocused,
  },
  ref
) {
  const [code, setCode] = useState(initialCode)
  const [description, setDescription] = useState(initialDescription)
  const [isEditingDescription, setIsEditingDescription] = useState(false)
  const [editorHeight, setEditorHeight] = useState(DEFAULT_HEIGHT)
  const [debugExpanded, setDebugExpanded] = useState(false)
  const editorRef = useRef<editor.IStandaloneCodeEditor | null>(null)
  const descriptionTextareaRef = useRef<HTMLTextAreaElement | null>(null)
  const codeRef = useRef(initialCode)  // Initialize with initialCode, not code
  const onRunAndAddCellRef = useRef(onRunAndAddCell)
  const isResizing = useRef(false)
  const startY = useRef(0)
  const startHeight = useRef(0)

  const handleResizeStart = (e: React.MouseEvent) => {
    isResizing.current = true
    startY.current = e.clientY
    startHeight.current = editorHeight
    document.addEventListener('mousemove', handleResizeMove)
    document.addEventListener('mouseup', handleResizeEnd)
    e.preventDefault()
  }

  const handleResizeMove = (e: MouseEvent) => {
    if (!isResizing.current) return
    const delta = e.clientY - startY.current
    const newHeight = Math.max(MIN_HEIGHT, startHeight.current + delta)
    setEditorHeight(newHeight)
  }

  const handleResizeEnd = () => {
    isResizing.current = false
    document.removeEventListener('mousemove', handleResizeMove)
    document.removeEventListener('mouseup', handleResizeEnd)
  }

  // Keep refs in sync
  codeRef.current = code
  onRunAndAddCellRef.current = onRunAndAddCell

  // Expose focus method via ref
  useImperativeHandle(ref, () => ({
    focus: () => {
      editorRef.current?.focus()
    },
  }))

  const shouldFocusRef = useRef(shouldFocus)
  shouldFocusRef.current = shouldFocus

  // Handle focus when shouldFocus changes (for already mounted editors)
  useEffect(() => {
    if (shouldFocus && editorRef.current) {
      editorRef.current.focus()
      onFocused?.()
    }
  }, [shouldFocus, onFocused])

  const handleEditorMount: OnMount = (editor, monaco) => {
    registerLatticeLanguage(monaco)
    editorRef.current = editor

    // Focus immediately if this cell should be focused (for newly created cells)
    if (shouldFocusRef.current) {
      editor.focus()
      onFocused?.()
    }

    // Add keyboard shortcut: Shift+Enter to run and add new cell
    editor.onKeyDown((e) => {
      if (e.shiftKey && e.keyCode === monaco.KeyCode.Enter) {
        e.preventDefault()
        e.stopPropagation()

        const currentCode = editorRef.current?.getValue() ?? ''

        if (onRunAndAddCellRef.current) {
          onRunAndAddCellRef.current(id, currentCode)
        } else {
          onRun(id, currentCode)
        }
      }
    })
  }

  const handleCodeChange = (value: string | undefined) => {
    const newCode = value ?? ''
    setCode(newCode)
    onCodeChange?.(id, newCode)
  }

  const handleRunClick = () => {
    onRun(id, code)
  }

  const handleDescriptionChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const newDescription = e.target.value
    setDescription(newDescription)
    onDescriptionChange?.(id, newDescription)
  }

  const handleDescriptionBlur = () => {
    setIsEditingDescription(false)
  }

  const handleDescriptionKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Escape') {
      setIsEditingDescription(false)
    }
  }

  const handleBannerClick = () => {
    setIsEditingDescription(true)
    // Focus the textarea after state update
    setTimeout(() => {
      descriptionTextareaRef.current?.focus()
    }, 0)
  }

  return (
    <div className="cell">
      <div className="cell-banner" onClick={!isEditingDescription ? handleBannerClick : undefined}>
        {isEditingDescription ? (
          <textarea
            ref={descriptionTextareaRef}
            className="cell-banner-editor"
            value={description}
            onChange={handleDescriptionChange}
            onBlur={handleDescriptionBlur}
            onKeyDown={handleDescriptionKeyDown}
            placeholder="Add a description (supports markdown)..."
            rows={3}
          />
        ) : description ? (
          <div className="cell-banner-content">
            <ReactMarkdown>{description}</ReactMarkdown>
          </div>
        ) : (
          <div className="cell-banner-placeholder">
            Click to add description...
          </div>
        )}
      </div>
      <div className="cell-header">
        <div className="cell-actions">
          <button
            className="run-button"
            onClick={handleRunClick}
            disabled={isRunning}
            title="Run (Shift+Enter)"
          >
            {isRunning ? 'Running...' : 'Run'}
          </button>
          {onDelete && (
            <button
              className="delete-button"
              onClick={() => onDelete(id)}
              title="Delete cell"
            >
              Delete
            </button>
          )}
        </div>
      </div>
      <div className="cell-editor">
        <Editor
          height={`${editorHeight}px`}
          defaultLanguage="lattice"
          theme="vs-dark"
          value={code}
          onChange={handleCodeChange}
          onMount={handleEditorMount}
          options={{
            minimap: { enabled: false },
            fontSize: 14,
            lineNumbers: 'on',
            scrollBeyondLastLine: false,
            automaticLayout: true,
            tabSize: 4,
            wordWrap: 'on',
            folding: false,
            lineDecorationsWidth: 10,
            lineNumbersMinChars: 3,
            renderLineHighlight: 'line',
            scrollbar: {
              vertical: 'auto',
              horizontal: 'auto',
            },
          }}
        />
        <div className="resize-handle" onMouseDown={handleResizeStart} />
      </div>
      <div className={`cell-output ${error ? 'has-error' : ''}`}>
        {error ? (
          <div className="error">{error}</div>
        ) : output && output.type === 'Table' ? (
          <PaginatedTable
            headers={output.data.headers}
            rows={output.data.rows}
            pageSize={20}
          />
        ) : output && output.type === 'Text' ? (
          <pre>{output.data}</pre>
        ) : (
          <span className="placeholder">Output will appear here</span>
        )}
      </div>
      {llmDebug && (
        <div className="llm-debug">
          <button
            className="debug-toggle"
            onClick={() => setDebugExpanded(!debugExpanded)}
          >
            {debugExpanded ? '▼' : '▶'} LLM Debug ({llmDebug.function_name})
          </button>
          {debugExpanded && (
            <div className="debug-content">
              <div className="debug-section">
                <strong>Return Type:</strong>
                <code>{llmDebug.return_type}</code>
              </div>
              <div className="debug-section">
                <strong>Prompt:</strong>
                <pre className="debug-prompt">{llmDebug.prompt}</pre>
              </div>
              <div className="debug-section">
                <strong>Raw Response:</strong>
                <pre className="debug-response">{llmDebug.raw_response}</pre>
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  )
})
