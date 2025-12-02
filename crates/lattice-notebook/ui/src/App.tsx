import { useState, useCallback, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Cell, CellHandle, CellOutput, EvalResponse, LlmDebugOutput } from './components/Cell'
import './App.css'

interface CellData {
  id: string
  code: string
  output: CellOutput | null
  llmDebug: LlmDebugOutput | null
  error: string | null
  isRunning: boolean
}

function generateId(): string {
  return Math.random().toString(36).substring(2, 9)
}

function App() {
  const [cells, setCells] = useState<CellData[]>([
    {
      id: generateId(),
      code: '// Define a function\ndef multiply(a: int, b: int) {\n    a * b\n}\n\nmultiply(3, 4)',
      output: null,
      llmDebug: null,
      error: null,
      isRunning: false,
    },
  ])
  const [focusCellId, setFocusCellId] = useState<string | null>(null)
  const cellRefs = useRef<Map<string, CellHandle>>(new Map())

  const runCell = useCallback(async (id: string, code: string) => {
    // Mark cell as running
    setCells((prev) =>
      prev.map((cell) =>
        cell.id === id ? { ...cell, isRunning: true, error: null } : cell
      )
    )

    try {
      const result = await invoke<EvalResponse>('eval_cell', { code })
      setCells((prev) =>
        prev.map((cell) =>
          cell.id === id
            ? {
                ...cell,
                output: result.output,
                llmDebug: result.llm_debug ?? null,
                error: null,
                isRunning: false,
              }
            : cell
        )
      )
    } catch (e) {
      setCells((prev) =>
        prev.map((cell) =>
          cell.id === id
            ? { ...cell, error: String(e), isRunning: false }
            : cell
        )
      )
    }
  }, [])

  const addCell = useCallback((afterId?: string): string => {
    const newId = generateId()
    setCells((prev) => {
      if (afterId) {
        const index = prev.findIndex((c) => c.id === afterId)
        if (index !== -1) {
          const newCells = [...prev]
          newCells.splice(index + 1, 0, {
            id: newId,
            code: '',
            output: null,
            llmDebug: null,
            error: null,
            isRunning: false,
          })
          return newCells
        }
      }
      return [
        ...prev,
        {
          id: newId,
          code: '',
          output: null,
          llmDebug: null,
          error: null,
          isRunning: false,
        },
      ]
    })
    return newId
  }, [])

  const runCellAndAddNew = useCallback(
    (id: string, code: string) => {
      // Run the cell - fire and forget, don't await
      runCell(id, code).catch(console.error)
      // Add a new cell after this one and focus it
      const newId = addCell(id)
      setFocusCellId(newId)
    },
    [runCell, addCell]
  )

  const deleteCell = useCallback((id: string) => {
    setCells((prev) => {
      // Don't delete if it's the last cell
      if (prev.length <= 1) return prev
      return prev.filter((cell) => cell.id !== id)
    })
  }, [])

  const updateCellCode = useCallback((id: string, code: string) => {
    setCells((prev) =>
      prev.map((cell) => (cell.id === id ? { ...cell, code } : cell))
    )
  }, [])

  const runAllCells = useCallback(async () => {
    for (const cell of cells) {
      await runCell(cell.id, cell.code)
    }
  }, [cells, runCell])

  const resetVM = useCallback(async () => {
    try {
      await invoke('reset_vm')
      // Clear all cell outputs after resetting
      setCells((prev) =>
        prev.map((cell) => ({
          ...cell,
          output: null,
          llmDebug: null,
          error: null,
        }))
      )
    } catch (e) {
      console.error('Failed to reset VM:', e)
    }
  }, [])

  return (
    <div className="app">
      <header className="toolbar">
        <h1>Lattice Notebook</h1>
        <div className="toolbar-actions">
          <button onClick={() => addCell()}>+ Add Cell</button>
          <button onClick={runAllCells}>Run All</button>
          <button onClick={resetVM}>Restart</button>
        </div>
      </header>
      <main className="notebook">
        {cells.map((cell) => (
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
            output={cell.output}
            llmDebug={cell.llmDebug}
            error={cell.error}
            isRunning={cell.isRunning}
            onRun={runCell}
            onRunAndAddCell={runCellAndAddNew}
            onDelete={cells.length > 1 ? deleteCell : undefined}
            onCodeChange={updateCellCode}
            shouldFocus={focusCellId === cell.id}
            onFocused={() => setFocusCellId(null)}
          />
        ))}
      </main>
    </div>
  )
}

export default App
