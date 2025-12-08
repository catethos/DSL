import { useEffect, useRef } from 'react'
import './FileContextMenu.css'

export interface ContextMenuPosition {
  x: number
  y: number
}

export interface FileContextMenuProps {
  position: ContextMenuPosition
  isDirectory: boolean
  onClose: () => void
  onNewFile: () => void
  onNewFolder: () => void
  onRename: () => void
  onDelete: () => void
}

export function FileContextMenu({
  position,
  isDirectory,
  onClose,
  onNewFile,
  onNewFolder,
  onRename,
  onDelete,
}: FileContextMenuProps) {
  const menuRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        onClose()
      }
    }

    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose()
      }
    }

    document.addEventListener('mousedown', handleClickOutside)
    document.addEventListener('keydown', handleEscape)

    return () => {
      document.removeEventListener('mousedown', handleClickOutside)
      document.removeEventListener('keydown', handleEscape)
    }
  }, [onClose])

  // Adjust position if menu would go off screen
  useEffect(() => {
    if (menuRef.current) {
      const rect = menuRef.current.getBoundingClientRect()
      const viewportWidth = window.innerWidth
      const viewportHeight = window.innerHeight

      let adjustedX = position.x
      let adjustedY = position.y

      if (position.x + rect.width > viewportWidth) {
        adjustedX = viewportWidth - rect.width - 8
      }

      if (position.y + rect.height > viewportHeight) {
        adjustedY = viewportHeight - rect.height - 8
      }

      if (adjustedX !== position.x || adjustedY !== position.y) {
        menuRef.current.style.left = `${adjustedX}px`
        menuRef.current.style.top = `${adjustedY}px`
      }
    }
  }, [position])

  const handleItemClick = (action: () => void) => {
    action()
    onClose()
  }

  return (
    <div
      ref={menuRef}
      className="file-context-menu"
      style={{ left: position.x, top: position.y }}
    >
      {isDirectory && (
        <>
          <button
            className="context-menu-item"
            onClick={() => handleItemClick(onNewFile)}
          >
            <span className="context-menu-icon">+</span>
            New File
          </button>
          <button
            className="context-menu-item"
            onClick={() => handleItemClick(onNewFolder)}
          >
            <span className="context-menu-icon">+</span>
            New Folder
          </button>
          <div className="context-menu-separator" />
        </>
      )}
      <button
        className="context-menu-item"
        onClick={() => handleItemClick(onRename)}
      >
        <span className="context-menu-icon">R</span>
        Rename
      </button>
      <button
        className="context-menu-item context-menu-item-danger"
        onClick={() => handleItemClick(onDelete)}
      >
        <span className="context-menu-icon">x</span>
        Delete
      </button>
    </div>
  )
}
