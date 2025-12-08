import { useState, useRef, useEffect, useCallback } from 'react'
import Editor, { OnMount, BeforeMount } from '@monaco-editor/react'
import type { editor } from 'monaco-editor'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { registerLatticeLanguage } from '../monaco'
import {
  LLM_PROVIDERS,
  getModelsForBaseUrl,
  getDefaultEnvVar,
} from '../data/llm-providers'

// Module-level flag to ensure completion provider is only registered once
let yamlCompletionProviderRegistered = false

// Output types from Tauri backend
interface CellOutput {
  type: 'Empty' | 'Text' | 'Table'
  data?: string | { headers: string[]; rows: string[][] }
}

interface LlmDebugOutput {
  function_name: string
  return_type: string
  prompt: string
  raw_response: string
}

interface EvalResponse {
  output: CellOutput
  llm_debug?: LlmDebugOutput
  execution_time_ms: number
}

// Input field definition parsed from YAML
interface InputField {
  name: string
  type: string
}

interface MarkdownLlmEditorProps {
  frontmatter: string
  promptBody: string
  onFrontmatterChange: (value: string) => void
  onPromptBodyChange: (value: string) => void
  isDirty?: boolean
  sessionId?: string
  filePath?: string | null
  onTestRun?: (
    frontmatter: string,
    promptBody: string,
    testInputs: Record<string, string | number | boolean>,
  ) => Promise<EvalResponse>
}

const DEFAULT_FRONTMATTER_HEIGHT = 200
const DEFAULT_PROMPT_HEIGHT = 300
const MIN_HEIGHT = 100

// Parse input fields from YAML frontmatter
function parseInputFields(yaml: string): InputField[] {
  // Match the input: section and extract fields
  const inputMatch = yaml.match(/^input:\s*\n((?:  \w+:\s*\w+\n?)+)/m)
  if (!inputMatch) {
    // Try single-line format: input: "name: Type"
    const singleMatch = yaml.match(/^input:\s*["']?(\w+):\s*(\w+)["']?/m)
    if (singleMatch) {
      return [{ name: singleMatch[1], type: singleMatch[2] }]
    }
    return []
  }

  const fields: InputField[] = []
  const lines = inputMatch[1].split('\n')
  for (const line of lines) {
    const match = line.match(/^\s+(\w+):\s*(\w+)/)
    if (match) {
      fields.push({ name: match[1], type: match[2] })
    }
  }
  return fields
}

export function MarkdownLlmEditor({
  frontmatter,
  promptBody,
  onFrontmatterChange,
  onPromptBodyChange,
  isDirty = false,
  onTestRun,
}: MarkdownLlmEditorProps) {
  const [frontmatterHeight, setFrontmatterHeight] = useState(DEFAULT_FRONTMATTER_HEIGHT)
  const [promptHeight, setPromptHeight] = useState(DEFAULT_PROMPT_HEIGHT)
  const [isPreviewMode, setIsPreviewMode] = useState(false)
  const [isTestMode, setIsTestMode] = useState(false)
  const [frontmatterCollapsed, setFrontmatterCollapsed] = useState(false)
  const [promptCollapsed, setPromptCollapsed] = useState(false)

  // Test run state
  const [testInputs, setTestInputs] = useState<Record<string, string>>({})
  const [testOutput, setTestOutput] = useState<CellOutput | null>(null)
  const [testError, setTestError] = useState<string | null>(null)
  const [testLlmDebug, setTestLlmDebug] = useState<LlmDebugOutput | null>(null)
  const [isRunning, setIsRunning] = useState(false)
  const [executionTimeMs, setExecutionTimeMs] = useState<number | null>(null)
  const [debugExpanded, setDebugExpanded] = useState(false)

  // Test panel collapsible/resizable state
  const [testInputsCollapsed, setTestInputsCollapsed] = useState(false)
  const [testOutputCollapsed, setTestOutputCollapsed] = useState(false)
  const [testInputsHeight, setTestInputsHeight] = useState(200)
  const [testOutputHeight, setTestOutputHeight] = useState(250)

  const frontmatterEditorRef = useRef<editor.IStandaloneCodeEditor | null>(null)
  const promptEditorRef = useRef<editor.IStandaloneCodeEditor | null>(null)

  // Parse input fields from frontmatter
  const inputFields = parseInputFields(frontmatter)

  // Initialize test inputs when input fields change
  useEffect(() => {
    const newInputs: Record<string, string> = {}
    for (const field of inputFields) {
      newInputs[field.name] = testInputs[field.name] ?? ''
    }
    setTestInputs(newInputs)
  }, [frontmatter])

  // Handle test run
  const handleTestRun = useCallback(async () => {
    if (!onTestRun || isRunning) return

    setIsRunning(true)
    setTestError(null)
    setTestOutput(null)
    setTestLlmDebug(null)
    setExecutionTimeMs(null)

    try {
      // Convert string inputs to appropriate types based on field type
      const typedInputs: Record<string, string | number | boolean> = {}
      for (const field of inputFields) {
        const value = testInputs[field.name] ?? ''
        if (field.type === 'Int') {
          typedInputs[field.name] = parseInt(value, 10) || 0
        } else if (field.type === 'Float') {
          typedInputs[field.name] = parseFloat(value) || 0
        } else if (field.type === 'Bool') {
          typedInputs[field.name] = value.toLowerCase() === 'true'
        } else {
          typedInputs[field.name] = value
        }
      }

      const response = await onTestRun(frontmatter, promptBody, typedInputs)
      setTestOutput(response.output)
      setTestLlmDebug(response.llm_debug ?? null)
      setExecutionTimeMs(response.execution_time_ms)
    } catch (e) {
      setTestError(String(e))
    } finally {
      setIsRunning(false)
    }
  }, [onTestRun, frontmatter, promptBody, testInputs, inputFields, isRunning])

  // Resize handlers for frontmatter
  const isFrontmatterResizing = useRef(false)
  const frontmatterStartY = useRef(0)
  const frontmatterStartHeight = useRef(0)

  const handleFrontmatterResizeStart = (e: React.MouseEvent) => {
    isFrontmatterResizing.current = true
    frontmatterStartY.current = e.clientY
    frontmatterStartHeight.current = frontmatterHeight
    document.addEventListener('mousemove', handleFrontmatterResizeMove)
    document.addEventListener('mouseup', handleFrontmatterResizeEnd)
    e.preventDefault()
  }

  const handleFrontmatterResizeMove = (e: MouseEvent) => {
    if (!isFrontmatterResizing.current) return
    const delta = e.clientY - frontmatterStartY.current
    setFrontmatterHeight(Math.max(MIN_HEIGHT, frontmatterStartHeight.current + delta))
  }

  const handleFrontmatterResizeEnd = () => {
    isFrontmatterResizing.current = false
    document.removeEventListener('mousemove', handleFrontmatterResizeMove)
    document.removeEventListener('mouseup', handleFrontmatterResizeEnd)
  }

  // Resize handlers for prompt
  const isPromptResizing = useRef(false)
  const promptStartY = useRef(0)
  const promptStartHeight = useRef(0)

  const handlePromptResizeStart = (e: React.MouseEvent) => {
    isPromptResizing.current = true
    promptStartY.current = e.clientY
    promptStartHeight.current = promptHeight
    document.addEventListener('mousemove', handlePromptResizeMove)
    document.addEventListener('mouseup', handlePromptResizeEnd)
    e.preventDefault()
  }

  const handlePromptResizeMove = (e: MouseEvent) => {
    if (!isPromptResizing.current) return
    const delta = e.clientY - promptStartY.current
    setPromptHeight(Math.max(MIN_HEIGHT, promptStartHeight.current + delta))
  }

  const handlePromptResizeEnd = () => {
    isPromptResizing.current = false
    document.removeEventListener('mousemove', handlePromptResizeMove)
    document.removeEventListener('mouseup', handlePromptResizeEnd)
  }

  // Resize handlers for test inputs
  const isTestInputsResizing = useRef(false)
  const testInputsStartY = useRef(0)
  const testInputsStartHeight = useRef(0)

  const handleTestInputsResizeStart = (e: React.MouseEvent) => {
    isTestInputsResizing.current = true
    testInputsStartY.current = e.clientY
    testInputsStartHeight.current = testInputsHeight
    document.addEventListener('mousemove', handleTestInputsResizeMove)
    document.addEventListener('mouseup', handleTestInputsResizeEnd)
    e.preventDefault()
  }

  const handleTestInputsResizeMove = (e: MouseEvent) => {
    if (!isTestInputsResizing.current) return
    const delta = e.clientY - testInputsStartY.current
    setTestInputsHeight(Math.max(MIN_HEIGHT, testInputsStartHeight.current + delta))
  }

  const handleTestInputsResizeEnd = () => {
    isTestInputsResizing.current = false
    document.removeEventListener('mousemove', handleTestInputsResizeMove)
    document.removeEventListener('mouseup', handleTestInputsResizeEnd)
  }

  // Resize handlers for test output
  const isTestOutputResizing = useRef(false)
  const testOutputStartY = useRef(0)
  const testOutputStartHeight = useRef(0)

  const handleTestOutputResizeStart = (e: React.MouseEvent) => {
    isTestOutputResizing.current = true
    testOutputStartY.current = e.clientY
    testOutputStartHeight.current = testOutputHeight
    document.addEventListener('mousemove', handleTestOutputResizeMove)
    document.addEventListener('mouseup', handleTestOutputResizeEnd)
    e.preventDefault()
  }

  const handleTestOutputResizeMove = (e: MouseEvent) => {
    if (!isTestOutputResizing.current) return
    const delta = e.clientY - testOutputStartY.current
    setTestOutputHeight(Math.max(MIN_HEIGHT, testOutputStartHeight.current + delta))
  }

  const handleTestOutputResizeEnd = () => {
    isTestOutputResizing.current = false
    document.removeEventListener('mousemove', handleTestOutputResizeMove)
    document.removeEventListener('mouseup', handleTestOutputResizeEnd)
  }

  const handleBeforeMount: BeforeMount = (monaco) => {
    registerLatticeLanguage(monaco)

    // Only register completion provider once globally
    if (yamlCompletionProviderRegistered) {
      return
    }
    yamlCompletionProviderRegistered = true

    // Register YAML completion provider for LLM config fields
    monaco.languages.registerCompletionItemProvider('yaml', {
      triggerCharacters: [' ', ':', '"'],
      provideCompletionItems: (model: editor.ITextModel, position: { lineNumber: number; column: number }) => {
        const textUntilPosition = model.getValueInRange({
          startLineNumber: 1,
          startColumn: 1,
          endLineNumber: position.lineNumber,
          endColumn: position.column,
        })

        const currentLine = model.getLineContent(position.lineNumber)
        const lineUntilCursor = currentLine.substring(0, position.column - 1)

        const suggestions: {
          label: string
          kind: number
          insertText: string
          range: { startLineNumber: number; startColumn: number; endLineNumber: number; endColumn: number }
          sortText: string
          detail?: string
          documentation?: string
        }[] = []

        const range = {
          startLineNumber: position.lineNumber,
          startColumn: position.column,
          endLineNumber: position.lineNumber,
          endColumn: position.column,
        }

        // Check if we're on a model: line
        if (/^\s*model:\s*/.test(lineUntilCursor)) {
          // Extract base_url from the document to filter models
          const baseUrlMatch = textUntilPosition.match(/base_url:\s*["']?([^"'\n]+)["']?/)
          const baseUrl = baseUrlMatch?.[1]?.trim()
          const models = baseUrl ? getModelsForBaseUrl(baseUrl) : LLM_PROVIDERS.flatMap(p => p.models)

          models.forEach((modelName, index) => {
            suggestions.push({
              label: modelName,
              kind: monaco.languages.CompletionItemKind.Value,
              insertText: modelName,
              range,
              sortText: index.toString().padStart(4, '0'),
              detail: baseUrl ? `Model for ${baseUrl}` : 'LLM Model',
            })
          })
        }

        // Check if we're on a base_url: line
        if (/^\s*base_url:\s*["']?/.test(lineUntilCursor)) {
          LLM_PROVIDERS.forEach((provider, index) => {
            suggestions.push({
              label: provider.api,
              kind: monaco.languages.CompletionItemKind.Value,
              insertText: `"${provider.api}"`,
              range,
              sortText: index.toString().padStart(4, '0'),
              detail: provider.name,
              documentation: `API endpoint for ${provider.name}`,
            })
          })
        }

        // Check if we're on an api_key_env: line
        if (/^\s*api_key_env:\s*["']?/.test(lineUntilCursor)) {
          // Extract base_url to suggest the matching env var
          const baseUrlMatch = textUntilPosition.match(/base_url:\s*["']?([^"'\n]+)["']?/)
          const baseUrl = baseUrlMatch?.[1]?.trim()

          if (baseUrl) {
            const defaultEnv = getDefaultEnvVar(baseUrl)
            suggestions.push({
              label: defaultEnv,
              kind: monaco.languages.CompletionItemKind.Value,
              insertText: `"${defaultEnv}"`,
              range,
              sortText: '0000',
              detail: 'Recommended for selected provider',
            })
          }

          // Also show all env vars
          LLM_PROVIDERS.forEach((provider, index) => {
            provider.env.forEach((envVar) => {
              if (!suggestions.some(s => s.label === envVar)) {
                suggestions.push({
                  label: envVar,
                  kind: monaco.languages.CompletionItemKind.Value,
                  insertText: `"${envVar}"`,
                  range,
                  sortText: (index + 1).toString().padStart(4, '0'),
                  detail: provider.name,
                })
              }
            })
          })
        }

        return { suggestions }
      },
    })
  }

  const handleFrontmatterMount: OnMount = (editor) => {
    frontmatterEditorRef.current = editor
  }

  const handlePromptMount: OnMount = (editor) => {
    promptEditorRef.current = editor
  }

  // Parse frontmatter to extract function name for display
  const parseFunctionName = (yaml: string): string | null => {
    const match = yaml.match(/^name:\s*(.+)$/m)
    return match ? match[1].trim() : null
  }

  const functionName = parseFunctionName(frontmatter)

  // Render editor panels (shared between normal and test modes)
  const renderEditorPanels = () => (
    <>
      {/* Frontmatter section */}
      <div className="md-section">
        <button
          className="md-section-toggle"
          onClick={() => setFrontmatterCollapsed(!frontmatterCollapsed)}
        >
          {frontmatterCollapsed ? '▶' : '▼'} Configuration (YAML)
        </button>
        {!frontmatterCollapsed && (
          <div className="md-editor-container">
            <Editor
              height={`${frontmatterHeight}px`}
              defaultLanguage="yaml"
              theme="lattice-dark"
              value={frontmatter}
              onChange={(value) => onFrontmatterChange(value ?? '')}
              beforeMount={handleBeforeMount}
              onMount={handleFrontmatterMount}
              options={{
                minimap: { enabled: false },
                fontSize: 14,
                lineNumbers: 'on',
                scrollBeyondLastLine: false,
                automaticLayout: true,
                tabSize: 2,
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
                quickSuggestions: true,
                suggestOnTriggerCharacters: true,
              }}
            />
            <div className="resize-handle" onMouseDown={handleFrontmatterResizeStart} />
          </div>
        )}
      </div>

      {/* Prompt body section */}
      <div className="md-section">
        <button
          className="md-section-toggle"
          onClick={() => setPromptCollapsed(!promptCollapsed)}
        >
          {promptCollapsed ? '▶' : '▼'} Prompt Template (Markdown)
        </button>
        {!promptCollapsed && (
          <div className="md-editor-container">
            <Editor
              height={`${promptHeight}px`}
              defaultLanguage="markdown"
              theme="lattice-dark"
              value={promptBody}
              onChange={(value) => onPromptBodyChange(value ?? '')}
              beforeMount={handleBeforeMount}
              onMount={handlePromptMount}
              options={{
                minimap: { enabled: false },
                fontSize: 14,
                lineNumbers: 'on',
                scrollBeyondLastLine: false,
                automaticLayout: true,
                tabSize: 2,
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
            <div className="resize-handle" onMouseDown={handlePromptResizeStart} />
          </div>
        )}
      </div>
    </>
  )

  // Render test panel (inputs + output)
  const renderTestPanel = () => (
    <div className="md-test-panel-right">
      {/* Test Inputs Section */}
      <div className="md-section">
        <button
          className="md-section-toggle"
          onClick={() => setTestInputsCollapsed(!testInputsCollapsed)}
        >
          <span className="toggle-icon">{testInputsCollapsed ? '▶' : '▼'}</span>
          <span>Test Inputs</span>
          <span className="md-section-spacer" />
          <button
            className={`md-run-button ${isRunning ? 'running' : ''}`}
            onClick={(e) => {
              e.stopPropagation()
              handleTestRun()
            }}
            disabled={isRunning || inputFields.length === 0}
          >
            {isRunning ? 'Running...' : 'Run'}
          </button>
        </button>
        {!testInputsCollapsed && (
          <div className="md-test-section-content" style={{ height: `${testInputsHeight}px` }}>
            {inputFields.length === 0 ? (
              <div className="md-test-no-inputs">
                No input fields defined in the configuration.
              </div>
            ) : (
              <div className="md-test-fields">
                {inputFields.map((field) => (
                  <div key={field.name} className="md-test-field">
                    <label className="md-test-field-label">
                      {field.name}
                      <span className="md-test-field-type">{field.type}</span>
                    </label>
                    {field.type === 'String' ? (
                      <textarea
                        className="md-test-field-input md-test-field-textarea"
                        value={testInputs[field.name] ?? ''}
                        onChange={(e) =>
                          setTestInputs({ ...testInputs, [field.name]: e.target.value })
                        }
                        placeholder={`Enter ${field.name}...`}
                        rows={3}
                      />
                    ) : (
                      <input
                        className="md-test-field-input"
                        type={field.type === 'Int' || field.type === 'Float' ? 'number' : 'text'}
                        step={field.type === 'Float' ? '0.01' : undefined}
                        value={testInputs[field.name] ?? ''}
                        onChange={(e) =>
                          setTestInputs({ ...testInputs, [field.name]: e.target.value })
                        }
                        placeholder={`Enter ${field.name}...`}
                      />
                    )}
                  </div>
                ))}
              </div>
            )}
            <div className="resize-handle" onMouseDown={handleTestInputsResizeStart} />
          </div>
        )}
      </div>

      {/* Test Output Section */}
      <div className="md-section">
        <button
          className="md-section-toggle"
          onClick={() => setTestOutputCollapsed(!testOutputCollapsed)}
        >
          <span className="toggle-icon">{testOutputCollapsed ? '▶' : '▼'}</span>
          <span>Output</span>
          {executionTimeMs !== null && (
            <span className="execution-time">{executionTimeMs}ms</span>
          )}
        </button>
        {!testOutputCollapsed && (
          <div className="md-test-section-content" style={{ height: `${testOutputHeight}px` }}>
            <div className={`md-test-output-content ${testError ? 'has-error' : ''}`}>
              {testError ? (
                <pre className="error">{testError}</pre>
              ) : testOutput ? (
                testOutput.type === 'Empty' ? (
                  <span className="placeholder">No output</span>
                ) : testOutput.type === 'Text' ? (
                  <pre>{testOutput.data as string}</pre>
                ) : testOutput.type === 'Table' ? (
                  <div className="table-container">
                    <table className="output-table">
                      <thead>
                        <tr>
                          {(testOutput.data as { headers: string[]; rows: string[][] }).headers.map(
                            (h, i) => (
                              <th key={i}>{h}</th>
                            )
                          )}
                        </tr>
                      </thead>
                      <tbody>
                        {(testOutput.data as { headers: string[]; rows: string[][] }).rows.map(
                          (row, i) => (
                            <tr key={i}>
                              {row.map((cell, j) => (
                                <td key={j}>{cell}</td>
                              ))}
                            </tr>
                          )
                        )}
                      </tbody>
                    </table>
                  </div>
                ) : (
                  <span className="placeholder">Unknown output type</span>
                )
              ) : (
                <span className="placeholder">Run the function to see output</span>
              )}
            </div>

            {/* LLM Debug Info */}
            {testLlmDebug && (
              <div className="llm-debug">
                <button
                  className="debug-toggle"
                  onClick={() => setDebugExpanded(!debugExpanded)}
                >
                  {debugExpanded ? '▼' : '▶'} LLM Debug
                </button>
                {debugExpanded && (
                  <div className="debug-content">
                    <div className="debug-section">
                      <strong>Function</strong>
                      <code>
                        {testLlmDebug.function_name}() -&gt; {testLlmDebug.return_type}
                      </code>
                    </div>
                    <div className="debug-section">
                      <strong>Prompt Sent</strong>
                      <pre className="debug-prompt">{testLlmDebug.prompt}</pre>
                    </div>
                    <div className="debug-section">
                      <strong>Raw Response</strong>
                      <pre className="debug-response">{testLlmDebug.raw_response}</pre>
                    </div>
                  </div>
                )}
              </div>
            )}
            <div className="resize-handle" onMouseDown={handleTestOutputResizeStart} />
          </div>
        )}
      </div>
    </div>
  )

  return (
    <div className="markdown-llm-editor">
      {/* Header bar */}
      <div className="md-editor-header">
        <div className="md-editor-title">
          <span className="md-icon">fn</span>
          <span className="md-function-name">
            {functionName || 'Untitled Function'}
          </span>
          {isDirty && <span className="dirty-indicator">*</span>}
        </div>
        <div className="md-editor-actions">
          <button
            className={`md-preview-toggle ${isPreviewMode ? 'active' : ''}`}
            onClick={() => {
              setIsPreviewMode(!isPreviewMode)
              if (!isPreviewMode) setIsTestMode(false)
            }}
            title={isPreviewMode ? 'Edit mode' : 'Preview mode'}
          >
            {isPreviewMode ? 'Edit' : 'Preview'}
          </button>
          {onTestRun && (
            <button
              className={`md-test-toggle ${isTestMode ? 'active' : ''}`}
              onClick={() => {
                setIsTestMode(!isTestMode)
                if (!isTestMode) setIsPreviewMode(false)
              }}
              title={isTestMode ? 'Close test panel' : 'Test function'}
            >
              Test
            </button>
          )}
        </div>
      </div>

      {isPreviewMode ? (
        <div className="md-preview">
          <div className="md-preview-frontmatter">
            <div className="md-preview-label">Configuration (YAML)</div>
            <pre className="md-preview-yaml">{frontmatter}</pre>
          </div>
          <div className="md-preview-prompt">
            <div className="md-preview-label">Prompt Template</div>
            <div className="md-preview-markdown">
              <ReactMarkdown remarkPlugins={[remarkGfm]}>{promptBody}</ReactMarkdown>
            </div>
          </div>
        </div>
      ) : isTestMode ? (
        /* Side-by-side layout when in test mode */
        <div className="md-split-layout">
          <div className="md-editor-left">
            {renderEditorPanels()}
          </div>
          {renderTestPanel()}
        </div>
      ) : (
        /* Normal vertical layout */
        renderEditorPanels()
      )}

      {/* Help text at bottom */}
      <div className="md-help-text">
        <span>Use <code>{'{variable}'}</code> syntax in the prompt for template variables.</span>
        <span>Variables must match the <code>input</code> fields in the configuration.</span>
      </div>
    </div>
  )
}
