import { useMemo, useState, useCallback } from 'react'
import {
  useReactTable,
  getCoreRowModel,
  getPaginationRowModel,
  flexRender,
  createColumnHelper,
} from '@tanstack/react-table'

interface PaginatedTableProps {
  headers: string[]
  rows: string[][]
  pageSize?: number
}

type RowData = Record<string, string>

// Threshold for considering text as "long"
const LONG_TEXT_THRESHOLD = 50
// Maximum characters to render in collapsed state (for performance)
const MAX_COLLAPSED_LENGTH = 200

export function PaginatedTable({ headers, rows, pageSize = 20 }: PaginatedTableProps) {
  const [pagination, setPagination] = useState({
    pageIndex: 0,
    pageSize,
  })
  // Track which cells are expanded: key = "rowIndex-colIndex"
  const [expandedCells, setExpandedCells] = useState<Set<string>>(new Set())

  // Convert rows to objects with header keys
  const data = useMemo(() => {
    return rows.map((row) => {
      const obj: RowData = {}
      headers.forEach((header, i) => {
        obj[header] = row[i] ?? ''
      })
      return obj
    })
  }, [headers, rows])

  // Create columns from headers
  const columnHelper = createColumnHelper<RowData>()
  const columns = useMemo(() => {
    return headers.map((header) =>
      columnHelper.accessor(header, {
        header: () => header,
        cell: (info) => info.getValue(),
      })
    )
  }, [headers, columnHelper])

  const toggleCell = useCallback((cellKey: string) => {
    setExpandedCells((prev) => {
      const next = new Set(prev)
      if (next.has(cellKey)) {
        next.delete(cellKey)
      } else {
        next.add(cellKey)
      }
      return next
    })
  }, [])

  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    onPaginationChange: setPagination,
    state: {
      pagination,
    },
  })

  const totalRows = rows.length
  const showPagination = totalRows > pageSize

  return (
    <div className="table-container">
      <table className="output-table">
        <thead>
          {table.getHeaderGroups().map((headerGroup) => (
            <tr key={headerGroup.id}>
              {headerGroup.headers.map((header) => (
                <th key={header.id}>
                  {header.isPlaceholder
                    ? null
                    : flexRender(header.column.columnDef.header, header.getContext())}
                </th>
              ))}
            </tr>
          ))}
        </thead>
        <tbody>
          {table.getRowModel().rows.map((row, rowIndex) => (
            <tr key={row.id}>
              {row.getVisibleCells().map((cell, colIndex) => {
                const cellKey = `${rowIndex}-${colIndex}`
                const value = cell.getValue() as string
                const isLong = value.length > LONG_TEXT_THRESHOLD
                const isExpanded = expandedCells.has(cellKey)

                // Truncate long text in collapsed state for performance
                const displayValue = isLong && !isExpanded && value.length > MAX_COLLAPSED_LENGTH
                  ? value.slice(0, MAX_COLLAPSED_LENGTH)
                  : value

                return (
                  <td
                    key={cell.id}
                    className={`${isExpanded ? 'expanded' : ''} ${isLong ? 'expandable' : ''}`}
                    onClick={isLong ? () => toggleCell(cellKey) : undefined}
                    title={isLong && !isExpanded ? 'Click to expand' : undefined}
                  >
                    {displayValue}
                  </td>
                )
              })}
            </tr>
          ))}
        </tbody>
      </table>

      {showPagination && (
        <div className="table-pagination">
          <div className="pagination-info">
            Showing {pagination.pageIndex * pagination.pageSize + 1} to{' '}
            {Math.min((pagination.pageIndex + 1) * pagination.pageSize, totalRows)} of{' '}
            {totalRows} rows
          </div>
          <div className="pagination-controls">
            <button
              className="pagination-btn"
              onClick={() => table.firstPage()}
              disabled={!table.getCanPreviousPage()}
              title="First page"
            >
              {'<<'}
            </button>
            <button
              className="pagination-btn"
              onClick={() => table.previousPage()}
              disabled={!table.getCanPreviousPage()}
              title="Previous page"
            >
              {'<'}
            </button>
            <span className="pagination-page">
              Page {pagination.pageIndex + 1} of {table.getPageCount()}
            </span>
            <button
              className="pagination-btn"
              onClick={() => table.nextPage()}
              disabled={!table.getCanNextPage()}
              title="Next page"
            >
              {'>'}
            </button>
            <button
              className="pagination-btn"
              onClick={() => table.lastPage()}
              disabled={!table.getCanNextPage()}
              title="Last page"
            >
              {'>>'}
            </button>
          </div>
        </div>
      )}
    </div>
  )
}
