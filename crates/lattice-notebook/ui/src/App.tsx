import { useState, useCallback, useRef, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { save, open } from '@tauri-apps/plugin-dialog'
import { Cell, CellHandle, CellOutput, EvalResponse, LlmDebugOutput } from './components/Cell'
import { DropdownMenu, MenuItem, MenuDivider, SubMenu } from './components/DropdownMenu'
import { TabBar, Tab } from './components/TabBar'
import { MarkdownLlmEditor } from './components/MarkdownLlmEditor'
import { LatSourceEditor } from './components/LatSourceEditor'
import { FileExplorer } from './components/FileExplorer'
import { WelcomeScreen } from './components/WelcomeScreen'
import { SplitContainer } from './components/SplitContainer'
import './App.css'

import type { editor } from 'monaco-editor'

// Markdown LLM file structure from Rust backend
interface MarkdownLlmFile {
  frontmatter: string
  prompt_body: string
}

// Monaco editor view state type
type EditorViewState = editor.ICodeEditorViewState | null

// Notebook file format types
interface CellUiState {
  editor_height?: number
  banner_height?: number
  editor_collapsed?: boolean
  output_collapsed?: boolean
  editor_view_state?: string // JSON-serialized Monaco view state
  level?: number // Hierarchy level (0 = root)
  collapsed?: boolean // Are children hidden?
}

interface NotebookCell {
  id: string
  code: string
  description: string
  output: CellOutput | null
  llm_debug: LlmDebugOutput | null
  ui_state?: CellUiState | null
}

interface Notebook {
  version: string
  created_at: string
  modified_at: string
  cells: NotebookCell[]
}

interface CellData {
  id: string
  code: string
  description: string
  output: CellOutput | null
  llmDebug: LlmDebugOutput | null
  error: string | null
  isRunning: boolean
  executionTimeMs: number | null
  // UI state (persisted)
  editorHeight?: number
  bannerHeight?: number
  editorCollapsed?: boolean
  outputCollapsed?: boolean
  editorViewState?: EditorViewState
  // Hierarchy
  level: number // 0 = root level
  collapsed: boolean // Are children hidden?
}

// Base tab state shared by all tab types
interface BaseTabState {
  id: string
  title: string
  filePath: string | null
  isDirty: boolean
}

// Notebook tab state
interface NotebookTabState extends BaseTabState {
  type: 'notebook'
  sessionId: string
  cells: CellData[]
  focusCellId: string | null
  hoistRootId: string | null // If set, show only this cell's subtree
}

// Markdown LLM file tab state
interface MarkdownTabState extends BaseTabState {
  type: 'markdown'
  sessionId: string
  frontmatter: string
  promptBody: string
}

// Plain .lat source file tab state
interface LatSourceTabState extends BaseTabState {
  type: 'lat'
  content: string
}

// Union type for all tab states
type TabState = NotebookTabState | MarkdownTabState | LatSourceTabState

function generateId(): string {
  return Math.random().toString(36).substring(2, 9)
}

function createDefaultCell(level: number = 0): CellData {
  return {
    id: generateId(),
    code: '',
    description: '',
    output: null,
    llmDebug: null,
    error: null,
    isRunning: false,
    executionTimeMs: null,
    level,
    collapsed: false,
  }
}

// ============================================
// Hierarchy Utility Functions
// ============================================

// Get descendant count for a cell (children, grandchildren, etc.)
function getDescendantCount(cells: CellData[], index: number): number {
  const parentLevel = cells[index].level
  let count = 0
  for (let j = index + 1; j < cells.length; j++) {
    if (cells[j].level <= parentLevel) break
    count++
  }
  return count
}

// Check if a cell has any children
function hasChildren(cells: CellData[], index: number): boolean {
  if (index >= cells.length - 1) return false
  return cells[index + 1].level > cells[index].level
}

// Check if a cell is visible (no collapsed ancestor)
function isCellVisible(cells: CellData[], index: number): boolean {
  let currentLevel = cells[index].level
  for (let j = index - 1; j >= 0; j--) {
    if (cells[j].level < currentLevel) {
      if (cells[j].collapsed) return false
      currentLevel = cells[j].level
    }
  }
  return true
}

// Get ancestor path for breadcrumbs
function getAncestorPath(cells: CellData[], index: number): { id: string; description: string; level: number }[] {
  const path: { id: string; description: string; level: number }[] = []
  let currentLevel = cells[index].level

  for (let j = index - 1; j >= 0; j--) {
    if (cells[j].level < currentLevel) {
      path.unshift({
        id: cells[j].id,
        description: cells[j].description || `Cell ${j + 1}`,
        level: cells[j].level,
      })
      currentLevel = cells[j].level
    }
  }
  return path
}

// Can indent? (must have a cell above and not already deeper)
function canIndent(cells: CellData[], index: number): boolean {
  if (index === 0) return false
  const prevLevel = cells[index - 1].level
  const currLevel = cells[index].level
  return currLevel <= prevLevel
}

// Can dedent? (must be at level > 0)
function canDedent(cells: CellData[], index: number): boolean {
  return cells[index].level > 0
}

// Indent a cell and its descendants
function indentCell(cells: CellData[], index: number): CellData[] {
  if (!canIndent(cells, index)) return cells

  const newCells = [...cells]
  const currLevel = cells[index].level

  // Indent this cell
  newCells[index] = { ...cells[index], level: currLevel + 1 }

  // Also indent all descendants
  for (let i = index + 1; i < cells.length; i++) {
    if (cells[i].level <= currLevel) break
    newCells[i] = { ...cells[i], level: cells[i].level + 1 }
  }

  return newCells
}

// Dedent a cell and its descendants
function dedentCell(cells: CellData[], index: number): CellData[] {
  if (!canDedent(cells, index)) return cells

  const newCells = [...cells]
  const currLevel = cells[index].level

  // Dedent this cell
  newCells[index] = { ...cells[index], level: currLevel - 1 }

  // Also dedent all descendants
  for (let i = index + 1; i < cells.length; i++) {
    if (cells[i].level <= currLevel) break
    newCells[i] = { ...cells[i], level: cells[i].level - 1 }
  }

  return newCells
}

async function createNewNotebookTab(): Promise<NotebookTabState> {
  const sessionId = await invoke<string>('create_session')
  return {
    type: 'notebook',
    id: generateId(),
    sessionId,
    title: 'Untitled',
    filePath: null,
    isDirty: false,
    cells: [createDefaultCell()],
    focusCellId: null,
    hoistRootId: null,
  }
}

async function createNewMarkdownTab(filePath: string | null = null): Promise<MarkdownTabState> {
  const sessionId = await invoke<string>('create_session')
  return {
    type: 'markdown',
    id: generateId(),
    sessionId,
    title: filePath ? filePath.split('/').pop()?.replace('.md', '') || 'Untitled' : 'New Function',
    filePath,
    isDirty: false,
    frontmatter: `name: my_function
model: gpt-4o-mini
base_url: "https://api.openai.com/v1"
api_key_env: "OPENAI_API_KEY"
input:
  text: String
output: String`,
    promptBody: 'Enter your prompt template here.\n\nUse {text} to reference input variables.',
  }
}

function createNewLatSourceTab(filePath: string | null = null, content: string = ''): LatSourceTabState {
  return {
    type: 'lat',
    id: generateId(),
    title: filePath ? filePath.split('/').pop()?.replace('.lat', '') || 'Untitled' : 'New Source',
    filePath,
    isDirty: false,
    content,
  }
}

// Layout mode for split view
type LayoutMode = 'single' | 'split-horizontal' | 'split-vertical'

function App() {
  const [tabs, setTabs] = useState<TabState[]>([])
  const [activeTabId, setActiveTabId] = useState<string | null>(null)
  const [isInitialized, setIsInitialized] = useState(false)
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false)
  const cellRefs = useRef<Map<string, CellHandle>>(new Map())

  // Split view state
  const [layoutMode, setLayoutMode] = useState<LayoutMode>('single')
  const [leftPaneTabId, setLeftPaneTabId] = useState<string | null>(null)
  const [rightPaneTabId, setRightPaneTabId] = useState<string | null>(null)
  const [splitRatio, setSplitRatio] = useState(0.5)
  const [activePaneId, setActivePaneId] = useState<'left' | 'right'>('left')

  // Initialize on mount - start with no tabs (blank state)
  useEffect(() => {
    if (!isInitialized) {
      setIsInitialized(true)
    }
  }, [isInitialized])

  // Refs for keyboard handler to access latest callback versions
  const newNotebookRef = useRef<() => void>(() => {})
  const openFileRef = useRef<() => void>(() => {})
  const saveNotebookRef = useRef<(saveAs: boolean) => void>(() => {})
  const addCellRef = useRef<() => void>(() => {})
  const runAllCellsRef = useRef<() => void>(() => {})

  // Get current tab
  const currentTab = tabs.find((t) => t.id === activeTabId)

  // Type guard helpers
  const isNotebookTab = (tab: TabState | undefined): tab is NotebookTabState =>
    tab?.type === 'notebook'
  const isMarkdownTab = (tab: TabState | undefined): tab is MarkdownTabState =>
    tab?.type === 'markdown'

  // Get current tab by type (if applicable)
  const currentNotebookTab = isNotebookTab(currentTab) ? currentTab : null
  const currentMarkdownTab = isMarkdownTab(currentTab) ? currentTab : null

  // Update a specific tab
  const updateTab = useCallback((tabId: string, updates: Partial<TabState>) => {
    setTabs((prev) =>
      prev.map((tab) => (tab.id === tabId ? { ...tab, ...updates } as TabState : tab))
    )
  }, [])

  // Update cells for current notebook tab
  const updateCells = useCallback(
    (tabId: string, updater: (cells: CellData[]) => CellData[]) => {
      setTabs((prev) =>
        prev.map((tab) =>
          tab.type === 'notebook' && tab.id === tabId
            ? { ...tab, cells: updater(tab.cells) }
            : tab
        )
      )
    },
    []
  )

  const runCell = useCallback(
    async (id: string, code: string) => {
      if (!currentNotebookTab) return

      // Mark cell as running
      updateCells(currentNotebookTab.id, (cells) =>
        cells.map((cell) =>
          cell.id === id ? { ...cell, isRunning: true, error: null } : cell
        )
      )

      try {
        const result = await invoke<EvalResponse>('eval_cell', {
          sessionId: currentNotebookTab.sessionId,
          code,
          notebookPath: currentNotebookTab.filePath,
        })
        updateCells(currentNotebookTab.id, (cells) =>
          cells.map((cell) =>
            cell.id === id
              ? {
                  ...cell,
                  output: result.output,
                  llmDebug: result.llm_debug ?? null,
                  error: null,
                  isRunning: false,
                  executionTimeMs: result.execution_time_ms,
                }
              : cell
          )
        )
      } catch (e) {
        updateCells(currentNotebookTab.id, (cells) =>
          cells.map((cell) =>
            cell.id === id
              ? { ...cell, error: String(e), isRunning: false, executionTimeMs: null }
              : cell
          )
        )
      }
    },
    [currentNotebookTab, updateCells]
  )

  const addCell = useCallback(
    (afterId?: string, position: 'above' | 'below' = 'below', asChild: boolean = false): string => {
      if (!currentNotebookTab) return ''
      const newId = generateId()
      updateCells(currentNotebookTab.id, (cells) => {
        if (afterId) {
          const index = cells.findIndex((c) => c.id === afterId)
          if (index !== -1) {
            const refCell = cells[index]
            const newCells = [...cells]
            const insertIndex = position === 'above' ? index : index + 1
            // New cell inherits level from reference, or +1 if asChild
            const newLevel = asChild ? refCell.level + 1 : refCell.level
            const newCell = createDefaultCell(newLevel)
            newCell.id = newId
            newCells.splice(insertIndex, 0, newCell)
            return newCells
          }
        }
        return [...cells, { ...createDefaultCell(), id: newId }]
      })
      return newId
    },
    [currentNotebookTab, updateCells]
  )

  const moveCellUp = useCallback(
    (cellId: string) => {
      if (!currentNotebookTab) return
      updateCells(currentNotebookTab.id, (cells) => {
        const index = cells.findIndex((c) => c.id === cellId)
        if (index <= 0) return cells // Already at top or not found

        const newCells = [...cells]
        const [cell] = newCells.splice(index, 1)
        newCells.splice(index - 1, 0, cell)
        return newCells
      })
      updateTab(currentNotebookTab.id, { isDirty: true })
    },
    [currentNotebookTab, updateCells, updateTab]
  )

  const moveCellDown = useCallback(
    (cellId: string) => {
      if (!currentNotebookTab) return
      updateCells(currentNotebookTab.id, (cells) => {
        const index = cells.findIndex((c) => c.id === cellId)
        if (index === -1 || index >= cells.length - 1) return cells // Already at bottom or not found

        const newCells = [...cells]
        const [cell] = newCells.splice(index, 1)
        newCells.splice(index + 1, 0, cell)
        return newCells
      })
      updateTab(currentNotebookTab.id, { isDirty: true })
    },
    [currentNotebookTab, updateCells, updateTab]
  )

  const runCellAndMoveNext = useCallback(
    (id: string, code: string) => {
      if (!currentNotebookTab) return

      // Run the cell - fire and forget
      runCell(id, code).catch(console.error)

      // Check if there's a cell after this one
      const currentIndex = currentNotebookTab.cells.findIndex((c) => c.id === id)
      if (currentIndex !== -1 && currentIndex < currentNotebookTab.cells.length - 1) {
        updateTab(currentNotebookTab.id, { focusCellId: currentNotebookTab.cells[currentIndex + 1].id })
      } else {
        const newId = addCell(id)
        updateTab(currentNotebookTab.id, { focusCellId: newId })
      }
    },
    [currentNotebookTab, runCell, addCell, updateTab]
  )

  const deleteCell = useCallback(
    (id: string) => {
      if (!currentNotebookTab) return
      updateCells(currentNotebookTab.id, (cells) => {
        if (cells.length <= 1) return cells
        return cells.filter((cell) => cell.id !== id)
      })
    },
    [currentNotebookTab, updateCells]
  )

  const updateCellCode = useCallback(
    (id: string, code: string) => {
      if (!currentNotebookTab) return
      updateCells(currentNotebookTab.id, (cells) =>
        cells.map((cell) => (cell.id === id ? { ...cell, code } : cell))
      )
      updateTab(currentNotebookTab.id, { isDirty: true })
    },
    [currentNotebookTab, updateCells, updateTab]
  )

  const updateCellDescription = useCallback(
    (id: string, description: string) => {
      if (!currentNotebookTab) return
      updateCells(currentNotebookTab.id, (cells) =>
        cells.map((cell) => (cell.id === id ? { ...cell, description } : cell))
      )
      updateTab(currentNotebookTab.id, { isDirty: true })
    },
    [currentNotebookTab, updateCells, updateTab]
  )

  const updateCellUiState = useCallback(
    (id: string, state: { editorHeight?: number; bannerHeight?: number; editorCollapsed?: boolean; outputCollapsed?: boolean; editorViewState?: EditorViewState }) => {
      if (!currentNotebookTab) return
      updateCells(currentNotebookTab.id, (cells) =>
        cells.map((cell) => (cell.id === id ? { ...cell, ...state } : cell))
      )
      updateTab(currentNotebookTab.id, { isDirty: true })
    },
    [currentNotebookTab, updateCells, updateTab]
  )

  // Hierarchy callbacks
  const handleIndent = useCallback(
    (id: string) => {
      if (!currentNotebookTab) return
      updateCells(currentNotebookTab.id, (cells) => {
        const index = cells.findIndex((c) => c.id === id)
        if (index === -1) return cells
        return indentCell(cells, index)
      })
      updateTab(currentNotebookTab.id, { isDirty: true })
    },
    [currentNotebookTab, updateCells, updateTab]
  )

  const handleDedent = useCallback(
    (id: string) => {
      if (!currentNotebookTab) return
      updateCells(currentNotebookTab.id, (cells) => {
        const index = cells.findIndex((c) => c.id === id)
        if (index === -1) return cells
        return dedentCell(cells, index)
      })
      updateTab(currentNotebookTab.id, { isDirty: true })
    },
    [currentNotebookTab, updateCells, updateTab]
  )

  const handleToggleCollapse = useCallback(
    (id: string) => {
      if (!currentNotebookTab) return
      updateCells(currentNotebookTab.id, (cells) =>
        cells.map((cell) =>
          cell.id === id ? { ...cell, collapsed: !cell.collapsed } : cell
        )
      )
      updateTab(currentNotebookTab.id, { isDirty: true })
    },
    [currentNotebookTab, updateCells, updateTab]
  )

  const handleHoist = useCallback(
    (id: string | null) => {
      if (!currentNotebookTab) return
      updateTab(currentNotebookTab.id, { hoistRootId: id })
    },
    [currentNotebookTab, updateTab]
  )

  const runAllCells = useCallback(async () => {
    if (!currentNotebookTab) return
    for (const cell of currentNotebookTab.cells) {
      await runCell(cell.id, cell.code)
    }
  }, [currentNotebookTab, runCell])

  const resetRuntime = useCallback(async () => {
    if (!currentNotebookTab) return
    try {
      await invoke('reset_runtime', { sessionId: currentNotebookTab.sessionId })
      // Clear all cell outputs after resetting
      updateCells(currentNotebookTab.id, (cells) =>
        cells.map((cell) => ({
          ...cell,
          output: null,
          llmDebug: null,
          error: null,
        }))
      )
    } catch (e) {
      console.error('Failed to reset runtime:', e)
    }
  }, [currentNotebookTab, updateCells])

  // Test run for markdown LLM files
  const handleTestMarkdownRun = useCallback(
    async (
      frontmatter: string,
      promptBody: string,
      testInputs: Record<string, string | number | boolean>
    ) => {
      if (!currentMarkdownTab) {
        throw new Error('No markdown tab active')
      }

      const result = await invoke<EvalResponse>('test_markdown_llm', {
        sessionId: currentMarkdownTab.sessionId,
        frontmatter,
        promptBody,
        testInputs,
        filePath: currentMarkdownTab.filePath,
      })

      return result
    },
    [currentMarkdownTab]
  )

  // Tab management
  const handleNewNotebookTab = useCallback(async () => {
    const newTab = await createNewNotebookTab()
    setTabs((prev) => [...prev, newTab])
    setActiveTabId(newTab.id)
  }, [])

  const handleNewMarkdownTab = useCallback(async () => {
    const newTab = await createNewMarkdownTab()
    setTabs((prev) => [...prev, newTab])
    setActiveTabId(newTab.id)
  }, [])

  const handleNewLatSourceTab = useCallback(() => {
    const newTab = createNewLatSourceTab()
    setTabs((prev) => [...prev, newTab])
    setActiveTabId(newTab.id)
  }, [])

  const handleCloseTab = useCallback(
    async (tabId: string) => {
      const tab = tabs.find((t) => t.id === tabId)
      if (!tab) return

      if (tab.isDirty) {
        const confirmed = confirm('This tab has unsaved changes. Close anyway?')
        if (!confirmed) return
      }

      // Destroy the VM session if it's a notebook or markdown tab
      if (tab.type === 'notebook' || tab.type === 'markdown') {
        try {
          await invoke('destroy_session', { sessionId: tab.sessionId })
        } catch (e) {
          console.error('Failed to destroy session:', e)
        }
      }

      // Remove tab
      setTabs((prev) => {
        const newTabs = prev.filter((t) => t.id !== tabId)
        // If closing the active tab, switch to another or clear active tab
        if (activeTabId === tabId) {
          if (newTabs.length > 0) {
            setActiveTabId(newTabs[newTabs.length - 1].id)
          } else {
            setActiveTabId(null)
          }
        }
        return newTabs
      })
    },
    [tabs, activeTabId]
  )

  const handleSelectTab = useCallback((tabId: string) => {
    setActiveTabId(tabId)
    // Also update split pane if in split mode
    if (layoutMode !== 'single') {
      if (activePaneId === 'left') {
        setLeftPaneTabId(tabId)
      } else {
        setRightPaneTabId(tabId)
      }
    }
  }, [layoutMode, activePaneId])

  // Split view functions
  const handleSplitView = useCallback((direction: 'horizontal' | 'vertical') => {
    if (!activeTabId) return

    // Enter split mode with current tab on left, and pick another tab for right
    setLayoutMode(direction === 'horizontal' ? 'split-horizontal' : 'split-vertical')
    setLeftPaneTabId(activeTabId)

    // Find another tab to put on the right, or use the same tab
    const otherTab = tabs.find(t => t.id !== activeTabId)
    setRightPaneTabId(otherTab?.id || activeTabId)
    setActivePaneId('left')
  }, [activeTabId, tabs])

  const handleCloseSplit = useCallback(() => {
    // Exit split mode, keep the active pane's tab as the single active tab
    const tabToKeep = activePaneId === 'left' ? leftPaneTabId : rightPaneTabId
    setLayoutMode('single')
    setActiveTabId(tabToKeep)
    setLeftPaneTabId(null)
    setRightPaneTabId(null)
  }, [activePaneId, leftPaneTabId, rightPaneTabId])

  const handlePaneClick = useCallback((paneId: 'left' | 'right') => {
    setActivePaneId(paneId)
    // Update activeTabId to match the clicked pane
    const tabId = paneId === 'left' ? leftPaneTabId : rightPaneTabId
    if (tabId) {
      setActiveTabId(tabId)
    }
  }, [leftPaneTabId, rightPaneTabId])

  const handleOpenInSplit = useCallback((tabId: string, pane: 'left' | 'right') => {
    if (layoutMode === 'single') {
      // Enter split mode
      setLayoutMode('split-horizontal')
      if (pane === 'left') {
        setLeftPaneTabId(tabId)
        setRightPaneTabId(activeTabId)
      } else {
        setLeftPaneTabId(activeTabId)
        setRightPaneTabId(tabId)
      }
    } else {
      // Just update the pane
      if (pane === 'left') {
        setLeftPaneTabId(tabId)
      } else {
        setRightPaneTabId(tabId)
      }
    }
    setActivePaneId(pane)
    setActiveTabId(tabId)
  }, [layoutMode, activeTabId])

  // File operations - Save current tab (handles both notebook and markdown)
  const saveCurrentFile = useCallback(
    async (saveAs: boolean = false) => {
      if (!currentTab) return

      if (currentTab.type === 'notebook') {
        // Save notebook
        let path = currentTab.filePath

        if (!path || saveAs) {
          const selected = await save({
            filters: [
              {
                name: 'Lattice Notebook',
                extensions: ['lat.nb'],
              },
            ],
            defaultPath: currentTab.filePath || 'untitled.lat.nb',
          })

          if (!selected) return
          path = selected
        }

        const notebook: Notebook = {
          version: '1.0',
          created_at: new Date().toISOString(),
          modified_at: new Date().toISOString(),
          cells: currentTab.cells.map((cell) => {
            // Get the current view state from the editor (includes folding state)
            const cellHandle = cellRefs.current.get(cell.id)
            const currentViewState = cellHandle?.getViewState() ?? cell.editorViewState
            const hasUiState = cell.editorHeight || cell.bannerHeight || cell.editorCollapsed || cell.outputCollapsed || currentViewState || cell.level > 0 || cell.collapsed
            return {
              id: cell.id,
              code: cell.code,
              description: cell.description,
              output: cell.output,
              llm_debug: cell.llmDebug,
              ui_state: hasUiState ? {
                editor_height: cell.editorHeight,
                banner_height: cell.bannerHeight,
                editor_collapsed: cell.editorCollapsed,
                output_collapsed: cell.outputCollapsed,
                editor_view_state: currentViewState ? JSON.stringify(currentViewState) : undefined,
                level: cell.level,
                collapsed: cell.collapsed,
              } : null,
            }
          }),
        }

        try {
          await invoke('save_notebook', { path, notebook })
          updateTab(currentTab.id, {
            filePath: path,
            isDirty: false,
            title: path.split('/').pop()?.replace('.lat.nb', '') || 'Untitled',
          })
        } catch (e) {
          console.error('Failed to save:', e)
          alert(`Failed to save: ${e}`)
        }
      } else if (currentTab.type === 'markdown') {
        // Save markdown LLM file
        let path = currentTab.filePath

        if (!path || saveAs) {
          const selected = await save({
            filters: [
              {
                name: 'Markdown LLM Function',
                extensions: ['md'],
              },
            ],
            defaultPath: currentTab.filePath || 'untitled.md',
          })

          if (!selected) return
          path = selected
        }

        try {
          await invoke('save_markdown_llm', {
            path,
            file: {
              frontmatter: currentTab.frontmatter,
              prompt_body: currentTab.promptBody,
            },
          })
          updateTab(currentTab.id, {
            filePath: path,
            isDirty: false,
            title: path.split('/').pop()?.replace('.md', '') || 'Untitled',
          })
        } catch (e) {
          console.error('Failed to save:', e)
          alert(`Failed to save: ${e}`)
        }
      } else if (currentTab.type === 'lat') {
        // Save plain .lat source file
        let path = currentTab.filePath

        if (!path || saveAs) {
          const selected = await save({
            filters: [
              {
                name: 'Lattice Source',
                extensions: ['lat'],
              },
            ],
            defaultPath: currentTab.filePath || 'untitled.lat',
          })

          if (!selected) return
          path = selected
        }

        try {
          await invoke('write_text_file', {
            path,
            contents: currentTab.content,
          })
          updateTab(currentTab.id, {
            filePath: path,
            isDirty: false,
            title: path.split('/').pop()?.replace('.lat', '') || 'Untitled',
          })
        } catch (e) {
          console.error('Failed to save:', e)
          alert(`Failed to save: ${e}`)
        }
      }
    },
    [currentTab, updateTab]
  )

  const newNotebook = useCallback(async () => {
    // Just create a new notebook tab
    await handleNewNotebookTab()
  }, [handleNewNotebookTab])

  const exportLat = useCallback(async () => {
    if (!currentNotebookTab) return

    const selected = await save({
      filters: [
        {
          name: 'Lattice Source',
          extensions: ['lat'],
        },
      ],
      defaultPath: currentNotebookTab.filePath?.replace('.lat.nb', '.lat') || 'untitled.lat',
    })

    if (!selected) return

    const notebookCells: NotebookCell[] = currentNotebookTab.cells.map((cell) => ({
      id: cell.id,
      code: cell.code,
      description: cell.description,
      output: cell.output,
      llm_debug: cell.llmDebug,
      ui_state: null, // Not needed for .lat export
    }))

    try {
      await invoke('export_lat', { path: selected, cells: notebookCells })
    } catch (e) {
      console.error('Failed to export:', e)
      alert(`Failed to export: ${e}`)
    }
  }, [currentNotebookTab])

  const importLat = useCallback(async () => {
    const selected = await open({
      filters: [
        {
          name: 'Lattice Source',
          extensions: ['lat'],
        },
      ],
      multiple: false,
    })

    if (!selected || Array.isArray(selected)) return

    try {
      const importedCells = await invoke<NotebookCell[]>('import_lat', { path: selected })

      // Create a new tab for the imported file
      const newTab = await createNewNotebookTab()

      // Reset the new runtime
      await invoke('reset_runtime', { sessionId: newTab.sessionId })

      // Convert to CellData format
      const loadedCells: CellData[] = importedCells.map((cell) => ({
        id: cell.id,
        code: cell.code,
        description: cell.description,
        output: cell.output,
        llmDebug: cell.llm_debug,
        error: null,
        isRunning: false,
        executionTimeMs: null,
        level: 0,
        collapsed: false,
      }))

      newTab.cells = loadedCells
      newTab.isDirty = false
      newTab.title = selected.split('/').pop()?.replace('.lat', '') || 'Imported'

      setTabs((prev) => [...prev, newTab])
      setActiveTabId(newTab.id)
    } catch (e) {
      console.error('Failed to import:', e)
      alert(`Failed to import: ${e}`)
    }
  }, [])

  // Handle file open from FileExplorer
  const handleFileOpen = useCallback(async (path: string, type: 'notebook' | 'markdown' | 'lat') => {
    // Check if file is already open
    const existingTab = tabs.find(t => t.filePath === path)
    if (existingTab) {
      setActiveTabId(existingTab.id)
      return
    }

    try {
      if (type === 'notebook') {
        const notebook = await invoke<Notebook>('load_notebook', { path })
        const newTab = await createNewNotebookTab()
        await invoke('reset_runtime', { sessionId: newTab.sessionId })

        const loadedCells: CellData[] = notebook.cells.map((cell) => {
          let editorViewState: EditorViewState = null
          if (cell.ui_state?.editor_view_state) {
            try {
              editorViewState = JSON.parse(cell.ui_state.editor_view_state)
            } catch {
              // Ignore parse errors
            }
          }
          return {
            id: cell.id,
            code: cell.code,
            description: cell.description,
            output: cell.output,
            llmDebug: cell.llm_debug,
            error: null,
            isRunning: false,
            executionTimeMs: null,
            editorHeight: cell.ui_state?.editor_height,
            bannerHeight: cell.ui_state?.banner_height,
            editorCollapsed: cell.ui_state?.editor_collapsed,
            outputCollapsed: cell.ui_state?.output_collapsed,
            editorViewState,
            level: cell.ui_state?.level ?? 0,
            collapsed: cell.ui_state?.collapsed ?? false,
          }
        })

        newTab.cells = loadedCells
        newTab.filePath = path
        newTab.isDirty = false
        newTab.title = path.split('/').pop()?.replace('.lat.nb', '') || 'Untitled'

        setTabs((prev) => [...prev, newTab])
        setActiveTabId(newTab.id)
      } else if (type === 'markdown') {
        const mdFile = await invoke<MarkdownLlmFile>('load_markdown_llm', { path })
        const newTab = await createNewMarkdownTab(path)
        newTab.frontmatter = mdFile.frontmatter
        newTab.promptBody = mdFile.prompt_body

        setTabs((prev) => [...prev, newTab])
        setActiveTabId(newTab.id)
      } else if (type === 'lat') {
        // Open as plain text source file
        const content = await invoke<string>('read_text_file', { path })
        const newTab = createNewLatSourceTab(path, content)

        setTabs((prev) => [...prev, newTab])
        setActiveTabId(newTab.id)
      }
    } catch (e) {
      console.error('Failed to open file:', e)
      alert(`Failed to open file: ${e}`)
    }
  }, [tabs])

  // Unified open file - handles all supported file types
  const openFile = useCallback(async () => {
    // Note: On macOS, compound extensions like 'lat.nb' don't work well with native dialogs.
    // We use 'nb' to match .lat.nb files and validate the full extension after selection.
    const selected = await open({
      filters: [
        {
          name: 'All Lattice Files',
          extensions: ['nb', 'lat', 'md'],
        },
        {
          name: 'Lattice Notebook',
          extensions: ['nb'],
        },
        {
          name: 'Lattice Source',
          extensions: ['lat'],
        },
        {
          name: 'Markdown LLM Function',
          extensions: ['md'],
        },
      ],
      multiple: false,
    })

    if (!selected || Array.isArray(selected)) return

    // Determine file type from extension
    let fileType: 'notebook' | 'markdown' | 'lat'
    if (selected.endsWith('.lat.nb')) {
      fileType = 'notebook'
    } else if (selected.endsWith('.md')) {
      fileType = 'markdown'
    } else if (selected.endsWith('.lat')) {
      fileType = 'lat'
    } else {
      alert('Unsupported file type')
      return
    }

    // Use existing handleFileOpen logic
    await handleFileOpen(selected, fileType)
  }, [handleFileOpen])

  // Update refs whenever callbacks change (for keyboard handler)
  useEffect(() => {
    newNotebookRef.current = newNotebook
    openFileRef.current = openFile
    saveNotebookRef.current = saveCurrentFile
    addCellRef.current = () => addCell()
    runAllCellsRef.current = runAllCells
  }, [newNotebook, openFile, saveCurrentFile, addCell, runAllCells])

  // Global keyboard shortcut handler
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const isMod = e.metaKey || e.ctrlKey

      if (isMod && !e.shiftKey) {
        switch (e.key.toLowerCase()) {
          case 'n':
            e.preventDefault()
            newNotebookRef.current()
            break
          case 'o':
            e.preventDefault()
            openFileRef.current()
            break
          case 's':
            e.preventDefault()
            saveNotebookRef.current(false)
            break
        }
      } else if (isMod && e.shiftKey) {
        switch (e.key.toLowerCase()) {
          case 's':
            e.preventDefault()
            saveNotebookRef.current(true)
            break
        }
      }

      // Cmd/Ctrl+Enter for Add Cell
      if (isMod && !e.shiftKey && e.key === 'Enter') {
        e.preventDefault()
        addCellRef.current()
      }

      // Shift+Cmd/Ctrl+Enter for Run All
      if (isMod && e.shiftKey && e.key === 'Enter') {
        e.preventDefault()
        runAllCellsRef.current()
      }
    }

    document.addEventListener('keydown', handleKeyDown)
    return () => document.removeEventListener('keydown', handleKeyDown)
  }, [])

  // Build tabs for TabBar
  const tabBarTabs: Tab[] = tabs.map((tab) => ({
    id: tab.id,
    title: tab.title,
    filePath: tab.filePath,
    isDirty: tab.isDirty,
    type: tab.type,
  }))

  // Helper function to render content for a specific tab ID
  const renderTabContent = (tabId: string | null) => {
    if (!tabId) return null

    const tab = tabs.find(t => t.id === tabId)
    if (!tab) return null

    if (tab.type === 'notebook') {
      const notebookTab = tab as NotebookTabState
      return (
        <main className="notebook">
          {/* Breadcrumb navigation when hoisted */}
          {notebookTab.hoistRootId && (() => {
            const hoistIndex = notebookTab.cells.findIndex(c => c.id === notebookTab.hoistRootId)
            if (hoistIndex === -1) return null
            const ancestors = getAncestorPath(notebookTab.cells, hoistIndex)
            return (
              <nav className="breadcrumb">
                <button className="breadcrumb-item breadcrumb-root" onClick={() => handleHoist(null)}>
                  Root
                </button>
                {ancestors.map((ancestor) => (
                  <span key={ancestor.id}>
                    <span className="breadcrumb-separator">&rsaquo;</span>
                    <button className="breadcrumb-item" onClick={() => handleHoist(ancestor.id)}>
                      {ancestor.description}
                    </button>
                  </span>
                ))}
                <span className="breadcrumb-separator">&rsaquo;</span>
                <span className="breadcrumb-current">
                  {notebookTab.cells[hoistIndex]?.description || 'Current'}
                </span>
              </nav>
            )
          })()}
          {notebookTab.cells.map((cell, index) => {
            // Filter: skip cells not visible due to collapse or hoisting
            if (!isCellVisible(notebookTab.cells, index)) return null

            // If hoisted, only show cells in the hoisted subtree
            if (notebookTab.hoistRootId) {
              const hoistIndex = notebookTab.cells.findIndex(c => c.id === notebookTab.hoistRootId)
              if (hoistIndex === -1) return null
              const hoistLevel = notebookTab.cells[hoistIndex].level

              // Skip cells before hoist root
              if (index < hoistIndex) return null
              // Skip cells after hoist subtree
              if (index > hoistIndex) {
                const isInSubtree = cell.level > hoistLevel
                if (!isInSubtree) return null
              }
            }

            const cellHasChildren = hasChildren(notebookTab.cells, index)
            const descendantCount = getDescendantCount(notebookTab.cells, index)

            return (
              <Cell
                key={cell.id}
                ref={(ref) => {
                  if (ref) {
                    cellRefs.current.set(cell.id, ref)
                  } else {
                    cellRefs.current.delete(cell.id)
                  }
                }}
                id={cell.id}
                initialCode={cell.code}
                initialDescription={cell.description}
                output={cell.output}
                llmDebug={cell.llmDebug}
                error={cell.error}
                isRunning={cell.isRunning}
                executionTimeMs={cell.executionTimeMs}
                onRun={runCell}
                onRunAndAddCell={runCellAndMoveNext}
                onDelete={notebookTab.cells.length > 1 ? deleteCell : undefined}
                onCodeChange={updateCellCode}
                onDescriptionChange={updateCellDescription}
                shouldFocus={notebookTab.focusCellId === cell.id}
                onFocused={() => updateTab(notebookTab.id, { focusCellId: null })}
                initialEditorHeight={cell.editorHeight}
                initialBannerHeight={cell.bannerHeight}
                initialEditorCollapsed={cell.editorCollapsed}
                initialOutputCollapsed={cell.outputCollapsed}
                initialEditorViewState={cell.editorViewState}
                onUiStateChange={updateCellUiState}
                onAddCellAbove={(id) => {
                  const newId = addCell(id, 'above')
                  updateTab(notebookTab.id, { focusCellId: newId })
                }}
                onAddCellBelow={(id) => {
                  const newId = addCell(id, 'below')
                  updateTab(notebookTab.id, { focusCellId: newId })
                }}
                onAddChild={(id) => {
                  const newId = addCell(id, 'below', true)
                  updateTab(notebookTab.id, { focusCellId: newId })
                }}
                onMoveUp={moveCellUp}
                onMoveDown={moveCellDown}
                // Hierarchy props
                level={cell.level}
                collapsed={cell.collapsed}
                hasChildren={cellHasChildren}
                descendantCount={descendantCount}
                onIndent={handleIndent}
                onDedent={handleDedent}
                onToggleCollapse={handleToggleCollapse}
                onHoist={handleHoist}
                canIndent={canIndent(notebookTab.cells, index)}
                canDedent={canDedent(notebookTab.cells, index)}
              />
            )
          })}
        </main>
      )
    } else if (tab.type === 'markdown') {
      const markdownTab = tab as MarkdownTabState
      return (
        <MarkdownLlmEditor
          frontmatter={markdownTab.frontmatter}
          promptBody={markdownTab.promptBody}
          onFrontmatterChange={(value) => {
            updateTab(markdownTab.id, { frontmatter: value, isDirty: true })
          }}
          onPromptBodyChange={(value) => {
            updateTab(markdownTab.id, { promptBody: value, isDirty: true })
          }}
          isDirty={markdownTab.isDirty}
          onTestRun={handleTestMarkdownRun}
        />
      )
    } else if (tab.type === 'lat') {
      const latTab = tab as LatSourceTabState
      return (
        <LatSourceEditor
          content={latTab.content}
          onChange={(value) => {
            updateTab(latTab.id, { content: value, isDirty: true })
          }}
          isDirty={latTab.isDirty}
          fileName={latTab.title}
        />
      )
    }

    return null
  }

  // Show loading state
  if (!isInitialized) {
    return (
      <div className="app">
        <div className="loading">Loading...</div>
      </div>
    )
  }

  return (
    <div className="app">
      <header className="toolbar">
        <h1>Lattice Notebook</h1>
        <nav className="toolbar-menus">
          <DropdownMenu label="File">
            <SubMenu label="New">
              <MenuItem onClick={newNotebook} shortcut="⌘N">Notebook</MenuItem>
              <MenuItem onClick={handleNewLatSourceTab}>Source (.lat)</MenuItem>
              <MenuItem onClick={handleNewMarkdownTab}>LLM Function (.md)</MenuItem>
            </SubMenu>
            <MenuItem onClick={openFile} shortcut="⌘O">
              Open...
            </MenuItem>
            <MenuDivider />
            <MenuItem onClick={() => saveCurrentFile(false)} shortcut="⌘S" disabled={!currentTab}>
              Save
            </MenuItem>
            <MenuItem onClick={() => saveCurrentFile(true)} shortcut="⇧⌘S" disabled={!currentTab}>
              Save As...
            </MenuItem>
            <MenuDivider />
            <MenuItem onClick={importLat} disabled={!currentNotebookTab}>Import .lat to Notebook</MenuItem>
            <MenuItem onClick={exportLat} disabled={!currentNotebookTab}>Export Notebook as .lat</MenuItem>
          </DropdownMenu>
          {currentNotebookTab && (
            <>
              <DropdownMenu label="Cell">
                <MenuItem onClick={() => addCell()} shortcut="⌘⏎">
                  Add Cell
                </MenuItem>
              </DropdownMenu>
              <DropdownMenu label="Run">
                <MenuItem onClick={runAllCells} shortcut="⇧⌘⏎">
                  Run All
                </MenuItem>
                <MenuDivider />
                <MenuItem onClick={resetRuntime}>Restart Runtime</MenuItem>
              </DropdownMenu>
            </>
          )}
          <DropdownMenu label="View">
            <MenuItem
              onClick={() => handleSplitView('horizontal')}
              disabled={!activeTabId || tabs.length < 1}
            >
              Split Right
            </MenuItem>
            <MenuItem
              onClick={() => handleSplitView('vertical')}
              disabled={!activeTabId || tabs.length < 1}
            >
              Split Down
            </MenuItem>
            {layoutMode !== 'single' && (
              <>
                <MenuDivider />
                <MenuItem onClick={handleCloseSplit}>
                  Close Split
                </MenuItem>
              </>
            )}
          </DropdownMenu>
        </nav>
      </header>
      <TabBar
        tabs={tabBarTabs}
        activeTabId={activeTabId || ''}
        onSelectTab={handleSelectTab}
        onCloseTab={handleCloseTab}
        onNewTab={handleNewNotebookTab}
        layoutMode={layoutMode}
        leftPaneTabId={leftPaneTabId}
        rightPaneTabId={rightPaneTabId}
        onOpenInPane={handleOpenInSplit}
      />
      <div className="app-body">
        <FileExplorer
          onFileOpen={handleFileOpen}
          isCollapsed={sidebarCollapsed}
          onToggleCollapse={() => setSidebarCollapsed(!sidebarCollapsed)}
        />
        {/* Render content: single view or split view */}
        {layoutMode === 'single' ? (
          // Single pane mode
          tabs.length > 0 && activeTabId ? (
            renderTabContent(activeTabId)
          ) : (
            <WelcomeScreen
              onNewNotebook={newNotebook}
              onOpenFile={openFile}
            />
          )
        ) : (
          // Split view mode
          <SplitContainer
            left={
              <div
                className={`split-pane-wrapper ${activePaneId === 'left' ? 'active-pane' : ''}`}
                onClick={() => handlePaneClick('left')}
              >
                <div className="split-pane-header">
                  <span>{tabs.find(t => t.id === leftPaneTabId)?.title || 'Empty'}</span>
                  <button
                    className="split-pane-close"
                    onClick={(e) => {
                      e.stopPropagation()
                      handleCloseSplit()
                    }}
                    title="Close split"
                  >
                    ×
                  </button>
                </div>
                {renderTabContent(leftPaneTabId)}
              </div>
            }
            right={
              <div
                className={`split-pane-wrapper ${activePaneId === 'right' ? 'active-pane' : ''}`}
                onClick={() => handlePaneClick('right')}
              >
                <div className="split-pane-header">
                  <span>{tabs.find(t => t.id === rightPaneTabId)?.title || 'Empty'}</span>
                  <button
                    className="split-pane-close"
                    onClick={(e) => {
                      e.stopPropagation()
                      handleCloseSplit()
                    }}
                    title="Close split"
                  >
                    ×
                  </button>
                </div>
                {renderTabContent(rightPaneTabId)}
              </div>
            }
            direction={layoutMode === 'split-horizontal' ? 'horizontal' : 'vertical'}
            ratio={splitRatio}
            onRatioChange={setSplitRatio}
          />
        )}
      </div>
    </div>
  )
}

export default App
