import { useState, useRef, useEffect, forwardRef, useImperativeHandle } from 'react'
import Editor, { OnMount, BeforeMount } from '@monaco-editor/react'
import type { editor } from 'monaco-editor'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { PaginatedTable } from './PaginatedTable'
import { registerLatticeLanguage } from '../monaco'

export interface CellHandle {
  focus: () => void
  getViewState: () => editor.ICodeEditorViewState | null
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
  execution_time_ms: number
}

// Monaco editor view state type (for folding persistence)
type EditorViewState = editor.ICodeEditorViewState | null

interface CellProps {
  id: string
  initialCode?: string
  initialDescription?: string
  output?: CellOutput | null
  llmDebug?: LlmDebugOutput | null
  error?: string | null
  isRunning?: boolean
  executionTimeMs?: number | null
  onRun: (id: string, code: string) => void
  onRunAndAddCell?: (id: string, code: string) => void
  onDelete?: (id: string) => void
  onCodeChange?: (id: string, code: string) => void
  onDescriptionChange?: (id: string, description: string) => void
  shouldFocus?: boolean
  onFocused?: () => void
  // UI state props (for persistence)
  initialEditorHeight?: number
  initialBannerHeight?: number
  initialEditorCollapsed?: boolean
  initialOutputCollapsed?: boolean
  initialEditorViewState?: EditorViewState
  onUiStateChange?: (id: string, state: { editorHeight?: number; bannerHeight?: number; editorCollapsed?: boolean; outputCollapsed?: boolean; editorViewState?: EditorViewState }) => void
  // Cell reordering
  onAddCellAbove?: (id: string) => void
  onAddCellBelow?: (id: string) => void
  onAddChild?: (id: string) => void
  onMoveUp?: (id: string) => void
  onMoveDown?: (id: string) => void
  // Hierarchy props
  level?: number
  collapsed?: boolean
  hasChildren?: boolean
  descendantCount?: number
  onIndent?: (id: string) => void
  onDedent?: (id: string) => void
  onToggleCollapse?: (id: string) => void
  onHoist?: (id: string) => void
  canIndent?: boolean
  canDedent?: boolean
}

const MIN_HEIGHT = 80
const DEFAULT_HEIGHT = 120
const MIN_BANNER_HEIGHT = 70
const DEFAULT_BANNER_HEIGHT = 100

// Format execution time for display
function formatExecutionTime(ms: number): string {
  if (ms < 1000) {
    return `${ms}ms`
  } else {
    return `${(ms / 1000).toFixed(2)}s`
  }
}

export const Cell = forwardRef<CellHandle, CellProps>(function Cell(
  {
    id,
    initialCode = '',
    initialDescription = '',
    output = null,
    llmDebug = null,
    error = null,
    isRunning = false,
    executionTimeMs = null,
    onRun,
    onRunAndAddCell,
    onDelete,
    onCodeChange,
    onDescriptionChange,
    shouldFocus = false,
    onFocused,
    initialEditorHeight,
    initialBannerHeight,
    initialEditorCollapsed,
    initialOutputCollapsed,
    initialEditorViewState,
    onUiStateChange,
    onAddCellAbove,
    onAddCellBelow,
    onAddChild,
    onMoveUp,
    onMoveDown,
    // Hierarchy props
    level = 0,
    collapsed = false,
    hasChildren = false,
    descendantCount = 0,
    onIndent,
    onDedent,
    onToggleCollapse,
    onHoist,
    canIndent = false,
    canDedent = false,
  },
  ref
) {
  const [code, setCode] = useState(initialCode)
  const [description, setDescription] = useState(initialDescription)
  const [isEditingTitle, setIsEditingTitle] = useState(false)
  const [isDescriptionExpanded, setIsDescriptionExpanded] = useState(false)
  const [isEditingDescription, setIsEditingDescription] = useState(false)
  const [editorHeight, setEditorHeight] = useState(initialEditorHeight ?? DEFAULT_HEIGHT)
  const [bannerHeight, setBannerHeight] = useState(initialBannerHeight ?? DEFAULT_BANNER_HEIGHT)
  const [debugExpanded, setDebugExpanded] = useState(false)
  const [editorCollapsed, setEditorCollapsed] = useState(initialEditorCollapsed ?? false)
  const [outputCollapsed, setOutputCollapsed] = useState(initialOutputCollapsed ?? false)
  const editorRef = useRef<editor.IStandaloneCodeEditor | null>(null)
  const pendingViewState = useRef<EditorViewState>(initialEditorViewState ?? null)
  const editorContainerRef = useRef<HTMLDivElement | null>(null)
  const descriptionTextareaRef = useRef<HTMLTextAreaElement | null>(null)
  const codeRef = useRef(initialCode)  // Initialize with initialCode, not code
  const onRunAndAddCellRef = useRef(onRunAndAddCell)
  const onMoveUpRef = useRef(onMoveUp)
  const onMoveDownRef = useRef(onMoveDown)
  const onIndentRef = useRef(onIndent)
  const onDedentRef = useRef(onDedent)
  const isResizing = useRef(false)
  const isResizingBanner = useRef(false)
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

  const currentEditorHeight = useRef(editorHeight)
  currentEditorHeight.current = editorHeight

  const currentBannerHeight = useRef(bannerHeight)
  currentBannerHeight.current = bannerHeight

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
    // Notify parent of height change (use ref to get current value)
    onUiStateChange?.(id, { editorHeight: currentEditorHeight.current })
  }

  const handleBannerResizeStart = (e: React.MouseEvent) => {
    isResizingBanner.current = true
    startY.current = e.clientY
    startHeight.current = bannerHeight
    document.addEventListener('mousemove', handleBannerResizeMove)
    document.addEventListener('mouseup', handleBannerResizeEnd)
    e.preventDefault()
    e.stopPropagation()
  }

  const handleBannerResizeMove = (e: MouseEvent) => {
    if (!isResizingBanner.current) return
    const delta = e.clientY - startY.current
    const newHeight = Math.max(MIN_BANNER_HEIGHT, startHeight.current + delta)
    setBannerHeight(newHeight)
  }

  const handleBannerResizeEnd = () => {
    isResizingBanner.current = false
    document.removeEventListener('mousemove', handleBannerResizeMove)
    document.removeEventListener('mouseup', handleBannerResizeEnd)
    // Notify parent of height change (use ref to get current value)
    onUiStateChange?.(id, { bannerHeight: currentBannerHeight.current })
  }

  // Keep refs in sync
  codeRef.current = code
  onRunAndAddCellRef.current = onRunAndAddCell
  onMoveUpRef.current = onMoveUp
  onMoveDownRef.current = onMoveDown
  onIndentRef.current = onIndent
  onDedentRef.current = onDedent

  // Expose methods via ref
  useImperativeHandle(ref, () => ({
    focus: () => {
      editorRef.current?.focus()
    },
    getViewState: () => {
      return editorRef.current?.saveViewState() ?? null
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

  // Handle wheel events to allow page scrolling when editor can't scroll
  useEffect(() => {
    const container = editorContainerRef.current
    if (!container) return

    const handleWheel = (e: WheelEvent) => {
      const editor = editorRef.current
      if (!editor) return

      // Get the scroll info from Monaco
      const scrollTop = editor.getScrollTop()
      const scrollHeight = editor.getScrollHeight()
      const clientHeight = editor.getLayoutInfo().height

      const canScrollUp = scrollTop > 0
      const canScrollDown = scrollTop + clientHeight < scrollHeight

      const scrollingUp = e.deltaY < 0
      const scrollingDown = e.deltaY > 0

      // If we can't scroll in the direction user wants, let the page scroll
      if ((scrollingUp && !canScrollUp) || (scrollingDown && !canScrollDown)) {
        // Find the notebook container and scroll it
        const notebook = document.querySelector('.notebook')
        if (notebook) {
          notebook.scrollTop += e.deltaY
        }
      }
    }

    container.addEventListener('wheel', handleWheel, { passive: true })
    return () => container.removeEventListener('wheel', handleWheel)
  }, [])

  const handleEditorBeforeMount: BeforeMount = (monaco) => {
    registerLatticeLanguage(monaco)
  }

  const handleEditorMount: OnMount = (editor, monaco) => {
    editorRef.current = editor

    // Restore view state (including folding) if we have one
    if (pendingViewState.current) {
      editor.restoreViewState(pendingViewState.current)
      pendingViewState.current = null
    }

    // Focus immediately if this cell should be focused (for newly created cells)
    if (shouldFocusRef.current) {
      editor.focus()
      onFocused?.()
    }

    // Save view state when editor loses focus (captures folding state)
    editor.onDidBlurEditorWidget(() => {
      const viewState = editor.saveViewState()
      onUiStateChange?.(id, { editorViewState: viewState })
    })

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

      // Cmd+Shift+Up to move cell up (Ctrl on Windows/Linux)
      if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.keyCode === monaco.KeyCode.UpArrow) {
        e.preventDefault()
        e.stopPropagation()
        onMoveUpRef.current?.(id)
      }

      // Cmd+Shift+Down to move cell down (Ctrl on Windows/Linux)
      if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.keyCode === monaco.KeyCode.DownArrow) {
        e.preventDefault()
        e.stopPropagation()
        onMoveDownRef.current?.(id)
      }

      // Ctrl+] to indent (increase hierarchy level)
      if ((e.metaKey || e.ctrlKey) && e.keyCode === monaco.KeyCode.BracketRight) {
        e.preventDefault()
        e.stopPropagation()
        onIndentRef.current?.(id)
      }

      // Ctrl+[ to dedent (decrease hierarchy level)
      if ((e.metaKey || e.ctrlKey) && e.keyCode === monaco.KeyCode.BracketLeft) {
        e.preventDefault()
        e.stopPropagation()
        onDedentRef.current?.(id)
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

  const handleDescriptionBlur = () => {
    setIsEditingDescription(false)
  }

  const handleDescriptionKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Escape') {
      setIsEditingDescription(false)
    }
  }

  // Parse title (first line) and body (rest) from description
  const parseDescription = (desc: string): { title: string; body: string } => {
    const lines = desc.split('\n')
    const title = lines[0]?.replace(/^#*\s*/, '') || '' // Remove leading # if present
    const body = lines.slice(1).join('\n').trim()
    return { title, body }
  }

  const { title: parsedTitle, body: descriptionBody } = parseDescription(description)
  const hasDescriptionBody = descriptionBody.length > 0

  const handleTitleChange = (newTitle: string) => {
    // Reconstruct description: new title + existing body
    const newDescription = descriptionBody ? `${newTitle}\n${descriptionBody}` : newTitle
    setDescription(newDescription)
    onDescriptionChange?.(id, newDescription)
  }

  const handleBodyChange = (newBody: string) => {
    // Reconstruct description: existing title + new body
    const newDescription = newBody ? `${parsedTitle}\n${newBody}` : parsedTitle
    setDescription(newDescription)
    onDescriptionChange?.(id, newDescription)
  }

  // Compute title weight class based on level
  const titleWeightClass = level === 0 ? 'title-weight-bold' : level === 1 ? 'title-weight-semibold' : 'title-weight-normal'

  // Level indicator bullets: ● (L0), ○ (L1), ◦ (L2+)
  const levelBullet = level === 0 ? '●' : level === 1 ? '○' : '◦'
  const levelBulletClass = level === 0 ? 'level-bullet-primary' : level === 1 ? 'level-bullet-secondary' : 'level-bullet-tertiary'

  return (
    <div className="cell" data-level={level}>
      {/* Cell insertion buttons */}
      <div className="cell-insert-buttons">
        {onAddCellAbove && (
          <button
            className="insert-cell-btn insert-above"
            onClick={() => onAddCellAbove(id)}
            title="Add cell above"
          >
            +
          </button>
        )}
        {onAddCellBelow && (
          <button
            className="insert-cell-btn insert-below"
            onClick={() => onAddCellBelow(id)}
            title="Add cell below"
          >
            +
          </button>
        )}
      </div>

      {/* Cell title bar with hierarchy controls */}
      <div className={`cell-title-bar ${titleWeightClass}`}>
        {/* Level indicator bullet */}
        <span className={`level-bullet ${levelBulletClass}`}>{levelBullet}</span>

        {/* Collapse toggle - only show if has children */}
        {hasChildren ? (
          <button
            className="collapse-toggle"
            onClick={() => onToggleCollapse?.(id)}
            title={collapsed ? `Expand (${descendantCount} items)` : 'Collapse'}
          >
            {collapsed ? '▶' : '▼'}
          </button>
        ) : (
          <span className="collapse-placeholder" />
        )}

        {/* Editable title (first line of description) */}
        {isEditingTitle ? (
          <input
            type="text"
            className="cell-title-input"
            value={parsedTitle}
            onChange={(e) => handleTitleChange(e.target.value)}
            onBlur={() => setIsEditingTitle(false)}
            onKeyDown={(e) => {
              if (e.key === 'Enter' || e.key === 'Escape') {
                setIsEditingTitle(false)
              }
            }}
            autoFocus
            placeholder="Cell title..."
          />
        ) : (
          <span
            className="cell-title"
            onClick={() => setIsEditingTitle(true)}
            title="Click to edit title"
          >
            {parsedTitle || <span className="cell-title-placeholder">···</span>}
          </span>
        )}

        {/* Description indicator - shows if there's body text */}
        {hasDescriptionBody && !isDescriptionExpanded && (
          <span className="description-indicator" title="Has description">
            ¶
          </span>
        )}

        {/* Child count badge when collapsed */}
        {collapsed && descendantCount > 0 && (
          <span className="child-count-badge">{descendantCount}</span>
        )}

        {/* Spacer */}
        <span className="cell-title-spacer" />

        {/* Cell actions (show on hover) */}
        <div className="cell-title-actions">
          {/* Toggle description button */}
          <button
            className={`hierarchy-btn description-toggle-btn ${isDescriptionExpanded ? 'active' : ''}`}
            onClick={() => setIsDescriptionExpanded(!isDescriptionExpanded)}
            title={isDescriptionExpanded ? 'Hide description' : 'Show/add description'}
          >
            ¶
          </button>
          {/* Indent/Dedent buttons */}
          {canDedent && (
            <button
              className="hierarchy-btn"
              onClick={() => onDedent?.(id)}
              title="Dedent (Cmd+[)"
            >
              ←
            </button>
          )}
          {canIndent && (
            <button
              className="hierarchy-btn"
              onClick={() => onIndent?.(id)}
              title="Indent (Cmd+])"
            >
              →
            </button>
          )}
          {/* Add child button */}
          {onAddChild && (
            <button
              className="hierarchy-btn"
              onClick={() => onAddChild(id)}
              title="Add child cell"
            >
              +↳
            </button>
          )}
          {/* Hoist button - zoom into this cell's subtree */}
          {hasChildren && onHoist && (
            <button
              className="hierarchy-btn"
              onClick={() => onHoist(id)}
              title="Focus on this section"
            >
              ⤢
            </button>
          )}
          {/* Run button */}
          <button
            className="run-button"
            onClick={handleRunClick}
            disabled={isRunning}
            title={isRunning ? 'Running...' : 'Run (Shift+Enter)'}
            aria-label={isRunning ? 'Running' : 'Run cell'}
          />
          {/* Delete button */}
          {onDelete && (
            <button
              className="delete-button"
              onClick={() => onDelete(id)}
              title="Delete cell"
              aria-label="Delete cell"
            />
          )}
        </div>
      </div>

      {/* Expandable description section (markdown) */}
      {isDescriptionExpanded && (
        <div className="cell-description-section">
          {isEditingDescription ? (
            <div className="cell-description-editor-container">
              <textarea
                ref={descriptionTextareaRef}
                className="cell-description-editor"
                style={{ height: `${bannerHeight}px` }}
                defaultValue={descriptionBody}
                onInput={(e) => handleBodyChange((e.target as HTMLTextAreaElement).value)}
                onBlur={(e) => {
                  handleBodyChange(e.target.value)
                  handleDescriptionBlur()
                }}
                onKeyDown={handleDescriptionKeyDown}
                placeholder="Add a description (supports markdown)..."
                autoFocus
              />
              <div className="banner-resize-handle" onMouseDown={handleBannerResizeStart} />
            </div>
          ) : (
            <div
              className="cell-description-content"
              onClick={() => {
                setIsEditingDescription(true)
                setTimeout(() => descriptionTextareaRef.current?.focus(), 0)
              }}
            >
              {descriptionBody ? (
                <ReactMarkdown remarkPlugins={[remarkGfm]}>{descriptionBody}</ReactMarkdown>
              ) : (
                <span className="cell-description-placeholder">
                  Click to add a description...
                </span>
              )}
            </div>
          )}
        </div>
      )}

      <div className="cell-section">
        <button
          className="section-toggle"
          onClick={() => {
            const newValue = !editorCollapsed
            setEditorCollapsed(newValue)
            onUiStateChange?.(id, { editorCollapsed: newValue })
          }}
        >
          {editorCollapsed ? '▶' : '▼'} Code
        </button>
        {!editorCollapsed && (
          <div className="cell-editor" ref={editorContainerRef}>
            <Editor
              height={`${editorHeight}px`}
              defaultLanguage="lattice"
              theme="lattice-dark"
              value={code}
              onChange={handleCodeChange}
              beforeMount={handleEditorBeforeMount}
              onMount={handleEditorMount}
              options={{
                minimap: { enabled: false },
                fontSize: 14,
                lineNumbers: 'on',
                scrollBeyondLastLine: false,
                automaticLayout: true,
                tabSize: 4,
                wordWrap: 'on',
                folding: true,
                foldingStrategy: 'indentation',
                showFoldingControls: 'always',
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
            <div className="resize-handle" onMouseDown={handleResizeStart} />
          </div>
        )}
      </div>
      {/* Only show output section when there's actual output or an error */}
      {(error || (output && output.type !== 'Empty')) && (
        <div className="cell-section">
          <button
            className="section-toggle"
            onClick={() => {
              const newValue = !outputCollapsed
              setOutputCollapsed(newValue)
              onUiStateChange?.(id, { outputCollapsed: newValue })
            }}
          >
            {outputCollapsed ? '▶' : '▼'} Output
            {executionTimeMs !== null && (
              <span className="execution-time">{formatExecutionTime(executionTimeMs)}</span>
            )}
          </button>
          {!outputCollapsed && (
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
              ) : null}
            </div>
          )}
        </div>
      )}
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
