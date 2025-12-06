import { useState, useCallback, useRef, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { save, open } from '@tauri-apps/plugin-dialog'
import { Cell, CellHandle, CellOutput, EvalResponse, LlmDebugOutput } from './components/Cell'
import { DropdownMenu, MenuItem, MenuDivider } from './components/DropdownMenu'
import { TabBar, Tab } from './components/TabBar'
import './App.css'

import type { editor } from 'monaco-editor'

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

interface TabState {
  id: string
  sessionId: string
  title: string
  filePath: string | null
  isDirty: boolean
  cells: CellData[]
  focusCellId: string | null
  hoistRootId: string | null // If set, show only this cell's subtree
}

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

async function createNewTab(): Promise<TabState> {
  const sessionId = await invoke<string>('create_session')
  return {
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

function App() {
  const [tabs, setTabs] = useState<TabState[]>([])
  const [activeTabId, setActiveTabId] = useState<string | null>(null)
  const [isInitialized, setIsInitialized] = useState(false)
  const cellRefs = useRef<Map<string, CellHandle>>(new Map())

  // Initialize first tab on mount
  useEffect(() => {
    if (!isInitialized) {
      createNewTab().then((tab) => {
        // Add a sample cell
        tab.cells = [{
          id: generateId(),
          code: '// Define a function\ndef multiply(a: int, b: int) {\n    a * b\n}\n\nmultiply(3, 4)',
          description: '',
          output: null,
          llmDebug: null,
          error: null,
          isRunning: false,
          executionTimeMs: null,
          level: 0,
          collapsed: false,
        }]
        setTabs([tab])
        setActiveTabId(tab.id)
        setIsInitialized(true)
      })
    }
  }, [isInitialized])

  // Refs for keyboard handler to access latest callback versions
  const newNotebookRef = useRef<() => void>(() => {})
  const loadNotebookRef = useRef<() => void>(() => {})
  const saveNotebookRef = useRef<(saveAs: boolean) => void>(() => {})
  const addCellRef = useRef<() => void>(() => {})
  const runAllCellsRef = useRef<() => void>(() => {})

  // Get current tab
  const currentTab = tabs.find((t) => t.id === activeTabId)

  // Update a specific tab
  const updateTab = useCallback((tabId: string, updates: Partial<TabState>) => {
    setTabs((prev) =>
      prev.map((tab) => (tab.id === tabId ? { ...tab, ...updates } : tab))
    )
  }, [])

  // Update cells for current tab
  const updateCells = useCallback(
    (tabId: string, updater: (cells: CellData[]) => CellData[]) => {
      setTabs((prev) =>
        prev.map((tab) =>
          tab.id === tabId ? { ...tab, cells: updater(tab.cells) } : tab
        )
      )
    },
    []
  )

  const runCell = useCallback(
    async (id: string, code: string) => {
      if (!currentTab) return

      // Mark cell as running
      updateCells(currentTab.id, (cells) =>
        cells.map((cell) =>
          cell.id === id ? { ...cell, isRunning: true, error: null } : cell
        )
      )

      try {
        const result = await invoke<EvalResponse>('eval_cell', {
          sessionId: currentTab.sessionId,
          code,
        })
        updateCells(currentTab.id, (cells) =>
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
        updateCells(currentTab.id, (cells) =>
          cells.map((cell) =>
            cell.id === id
              ? { ...cell, error: String(e), isRunning: false, executionTimeMs: null }
              : cell
          )
        )
      }
    },
    [currentTab, updateCells]
  )

  const addCell = useCallback(
    (afterId?: string, position: 'above' | 'below' = 'below', asChild: boolean = false): string => {
      if (!currentTab) return ''
      const newId = generateId()
      updateCells(currentTab.id, (cells) => {
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
    [currentTab, updateCells]
  )

  const moveCellUp = useCallback(
    (cellId: string) => {
      if (!currentTab) return
      updateCells(currentTab.id, (cells) => {
        const index = cells.findIndex((c) => c.id === cellId)
        if (index <= 0) return cells // Already at top or not found

        const newCells = [...cells]
        const [cell] = newCells.splice(index, 1)
        newCells.splice(index - 1, 0, cell)
        return newCells
      })
      updateTab(currentTab.id, { isDirty: true })
    },
    [currentTab, updateCells, updateTab]
  )

  const moveCellDown = useCallback(
    (cellId: string) => {
      if (!currentTab) return
      updateCells(currentTab.id, (cells) => {
        const index = cells.findIndex((c) => c.id === cellId)
        if (index === -1 || index >= cells.length - 1) return cells // Already at bottom or not found

        const newCells = [...cells]
        const [cell] = newCells.splice(index, 1)
        newCells.splice(index + 1, 0, cell)
        return newCells
      })
      updateTab(currentTab.id, { isDirty: true })
    },
    [currentTab, updateCells, updateTab]
  )

  const runCellAndMoveNext = useCallback(
    (id: string, code: string) => {
      if (!currentTab) return

      // Run the cell - fire and forget
      runCell(id, code).catch(console.error)

      // Check if there's a cell after this one
      const currentIndex = currentTab.cells.findIndex((c) => c.id === id)
      if (currentIndex !== -1 && currentIndex < currentTab.cells.length - 1) {
        updateTab(currentTab.id, { focusCellId: currentTab.cells[currentIndex + 1].id })
      } else {
        const newId = addCell(id)
        updateTab(currentTab.id, { focusCellId: newId })
      }
    },
    [currentTab, runCell, addCell, updateTab]
  )

  const deleteCell = useCallback(
    (id: string) => {
      if (!currentTab) return
      updateCells(currentTab.id, (cells) => {
        if (cells.length <= 1) return cells
        return cells.filter((cell) => cell.id !== id)
      })
    },
    [currentTab, updateCells]
  )

  const updateCellCode = useCallback(
    (id: string, code: string) => {
      if (!currentTab) return
      updateCells(currentTab.id, (cells) =>
        cells.map((cell) => (cell.id === id ? { ...cell, code } : cell))
      )
      updateTab(currentTab.id, { isDirty: true })
    },
    [currentTab, updateCells, updateTab]
  )

  const updateCellDescription = useCallback(
    (id: string, description: string) => {
      if (!currentTab) return
      updateCells(currentTab.id, (cells) =>
        cells.map((cell) => (cell.id === id ? { ...cell, description } : cell))
      )
      updateTab(currentTab.id, { isDirty: true })
    },
    [currentTab, updateCells, updateTab]
  )

  const updateCellUiState = useCallback(
    (id: string, state: { editorHeight?: number; bannerHeight?: number; editorCollapsed?: boolean; outputCollapsed?: boolean; editorViewState?: EditorViewState }) => {
      if (!currentTab) return
      updateCells(currentTab.id, (cells) =>
        cells.map((cell) => (cell.id === id ? { ...cell, ...state } : cell))
      )
      updateTab(currentTab.id, { isDirty: true })
    },
    [currentTab, updateCells, updateTab]
  )

  // Hierarchy callbacks
  const handleIndent = useCallback(
    (id: string) => {
      if (!currentTab) return
      updateCells(currentTab.id, (cells) => {
        const index = cells.findIndex((c) => c.id === id)
        if (index === -1) return cells
        return indentCell(cells, index)
      })
      updateTab(currentTab.id, { isDirty: true })
    },
    [currentTab, updateCells, updateTab]
  )

  const handleDedent = useCallback(
    (id: string) => {
      if (!currentTab) return
      updateCells(currentTab.id, (cells) => {
        const index = cells.findIndex((c) => c.id === id)
        if (index === -1) return cells
        return dedentCell(cells, index)
      })
      updateTab(currentTab.id, { isDirty: true })
    },
    [currentTab, updateCells, updateTab]
  )

  const handleToggleCollapse = useCallback(
    (id: string) => {
      if (!currentTab) return
      updateCells(currentTab.id, (cells) =>
        cells.map((cell) =>
          cell.id === id ? { ...cell, collapsed: !cell.collapsed } : cell
        )
      )
      updateTab(currentTab.id, { isDirty: true })
    },
    [currentTab, updateCells, updateTab]
  )

  const handleHoist = useCallback(
    (id: string | null) => {
      if (!currentTab) return
      updateTab(currentTab.id, { hoistRootId: id })
    },
    [currentTab, updateTab]
  )

  const runAllCells = useCallback(async () => {
    if (!currentTab) return
    for (const cell of currentTab.cells) {
      await runCell(cell.id, cell.code)
    }
  }, [currentTab, runCell])

  const resetRuntime = useCallback(async () => {
    if (!currentTab) return
    try {
      await invoke('reset_runtime', { sessionId: currentTab.sessionId })
      // Clear all cell outputs after resetting
      updateCells(currentTab.id, (cells) =>
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
  }, [currentTab, updateCells])

  // Tab management
  const handleNewTab = useCallback(async () => {
    const newTab = await createNewTab()
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

      // Destroy the VM session
      try {
        await invoke('destroy_session', { sessionId: tab.sessionId })
      } catch (e) {
        console.error('Failed to destroy session:', e)
      }

      // Remove tab
      setTabs((prev) => {
        const newTabs = prev.filter((t) => t.id !== tabId)
        // If closing the active tab, switch to another
        if (activeTabId === tabId && newTabs.length > 0) {
          setActiveTabId(newTabs[newTabs.length - 1].id)
        }
        return newTabs
      })
    },
    [tabs, activeTabId]
  )

  const handleSelectTab = useCallback((tabId: string) => {
    setActiveTabId(tabId)
  }, [])

  // File operations
  const saveNotebook = useCallback(
    async (saveAs: boolean = false) => {
      if (!currentTab) return

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
    },
    [currentTab, updateTab]
  )

  const loadNotebook = useCallback(async () => {
    const selected = await open({
      filters: [
        {
          name: 'Lattice Notebook',
          extensions: ['lat.nb'],
        },
      ],
      multiple: false,
    })

    if (!selected || Array.isArray(selected)) return

    try {
      const notebook = await invoke<Notebook>('load_notebook', { path: selected })

      // Create a new tab for the loaded notebook
      const newTab = await createNewTab()

      // Reset the new runtime
      await invoke('reset_runtime', { sessionId: newTab.sessionId })

      // Convert notebook cells to CellData format
      const loadedCells: CellData[] = notebook.cells.map((cell) => {
        // Parse the serialized view state if present
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
          // Restore UI state
          editorHeight: cell.ui_state?.editor_height,
          bannerHeight: cell.ui_state?.banner_height,
          editorCollapsed: cell.ui_state?.editor_collapsed,
          outputCollapsed: cell.ui_state?.output_collapsed,
          editorViewState,
          // Hierarchy state
          level: cell.ui_state?.level ?? 0,
          collapsed: cell.ui_state?.collapsed ?? false,
        }
      })

      newTab.cells = loadedCells
      newTab.filePath = selected
      newTab.isDirty = false
      newTab.title = selected.split('/').pop()?.replace('.lat.nb', '') || 'Untitled'

      setTabs((prev) => [...prev, newTab])
      setActiveTabId(newTab.id)
    } catch (e) {
      console.error('Failed to load:', e)
      alert(`Failed to load: ${e}`)
    }
  }, [])

  const newNotebook = useCallback(async () => {
    // Just create a new tab
    await handleNewTab()
  }, [handleNewTab])

  const exportLat = useCallback(async () => {
    if (!currentTab) return

    const selected = await save({
      filters: [
        {
          name: 'Lattice Source',
          extensions: ['lat'],
        },
      ],
      defaultPath: currentTab.filePath?.replace('.lat.nb', '.lat') || 'untitled.lat',
    })

    if (!selected) return

    const notebookCells: NotebookCell[] = currentTab.cells.map((cell) => ({
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
  }, [currentTab])

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
      const newTab = await createNewTab()

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

  // Update refs whenever callbacks change (for keyboard handler)
  useEffect(() => {
    newNotebookRef.current = newNotebook
    loadNotebookRef.current = loadNotebook
    saveNotebookRef.current = saveNotebook
    addCellRef.current = () => addCell()
    runAllCellsRef.current = runAllCells
  }, [newNotebook, loadNotebook, saveNotebook, addCell, runAllCells])

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
            loadNotebookRef.current()
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
    sessionId: tab.sessionId,
    title: tab.title,
    filePath: tab.filePath,
    isDirty: tab.isDirty,
  }))

  // Show loading state
  if (!isInitialized || !currentTab) {
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
            <MenuItem onClick={newNotebook} shortcut="⌘N">
              New
            </MenuItem>
            <MenuItem onClick={loadNotebook} shortcut="⌘O">
              Open
            </MenuItem>
            <MenuDivider />
            <MenuItem onClick={() => saveNotebook(false)} shortcut="⌘S">
              Save
            </MenuItem>
            <MenuItem onClick={() => saveNotebook(true)} shortcut="⇧⌘S">
              Save As
            </MenuItem>
            <MenuDivider />
            <MenuItem onClick={importLat}>Import .lat</MenuItem>
            <MenuItem onClick={exportLat}>Export .lat</MenuItem>
          </DropdownMenu>
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
        </nav>
      </header>
      <TabBar
        tabs={tabBarTabs}
        activeTabId={activeTabId || ''}
        onSelectTab={handleSelectTab}
        onCloseTab={handleCloseTab}
        onNewTab={handleNewTab}
      />
      <main className="notebook">
        {/* Breadcrumb navigation when hoisted */}
        {currentTab.hoistRootId && (() => {
          const hoistIndex = currentTab.cells.findIndex(c => c.id === currentTab.hoistRootId)
          if (hoistIndex === -1) return null
          const ancestors = getAncestorPath(currentTab.cells, hoistIndex)
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
                {currentTab.cells[hoistIndex]?.description || 'Current'}
              </span>
            </nav>
          )
        })()}
        {currentTab.cells.map((cell, index) => {
          // Filter: skip cells not visible due to collapse or hoisting
          if (!isCellVisible(currentTab.cells, index)) return null

          // If hoisted, only show cells in the hoisted subtree
          if (currentTab.hoistRootId) {
            const hoistIndex = currentTab.cells.findIndex(c => c.id === currentTab.hoistRootId)
            if (hoistIndex === -1) return null
            const hoistLevel = currentTab.cells[hoistIndex].level

            // Skip cells before hoist root
            if (index < hoistIndex) return null
            // Skip cells after hoist subtree
            if (index > hoistIndex) {
              const isInSubtree = cell.level > hoistLevel
              if (!isInSubtree) return null
            }
          }

          const cellHasChildren = hasChildren(currentTab.cells, index)
          const descendantCount = getDescendantCount(currentTab.cells, index)

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
              onDelete={currentTab.cells.length > 1 ? deleteCell : undefined}
              onCodeChange={updateCellCode}
              onDescriptionChange={updateCellDescription}
              shouldFocus={currentTab.focusCellId === cell.id}
              onFocused={() => updateTab(currentTab.id, { focusCellId: null })}
              initialEditorHeight={cell.editorHeight}
              initialBannerHeight={cell.bannerHeight}
              initialEditorCollapsed={cell.editorCollapsed}
              initialOutputCollapsed={cell.outputCollapsed}
              initialEditorViewState={cell.editorViewState}
              onUiStateChange={updateCellUiState}
              onAddCellAbove={(id) => {
                const newId = addCell(id, 'above')
                updateTab(currentTab.id, { focusCellId: newId })
              }}
              onAddCellBelow={(id) => {
                const newId = addCell(id, 'below')
                updateTab(currentTab.id, { focusCellId: newId })
              }}
              onAddChild={(id) => {
                const newId = addCell(id, 'below', true)
                updateTab(currentTab.id, { focusCellId: newId })
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
              canIndent={canIndent(currentTab.cells, index)}
              canDedent={canDedent(currentTab.cells, index)}
            />
          )
        })}
      </main>
    </div>
  )
}

export default App
