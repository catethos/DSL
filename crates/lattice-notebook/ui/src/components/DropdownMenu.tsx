import { useState, useRef, useEffect, ReactNode } from 'react'
import './DropdownMenu.css'

interface MenuItemProps {
  onClick: () => void
  children: ReactNode
  shortcut?: string
  disabled?: boolean
}

export function MenuItem({ onClick, children, shortcut, disabled }: MenuItemProps) {
  return (
    <button
      className="dropdown-item"
      onClick={onClick}
      disabled={disabled}
    >
      <span className="dropdown-item-label">{children}</span>
      {shortcut && <span className="dropdown-item-shortcut">{shortcut}</span>}
    </button>
  )
}

export function MenuDivider() {
  return <div className="dropdown-divider" />
}

interface DropdownMenuProps {
  label: string
  children: ReactNode
}

export function DropdownMenu({ label, children }: DropdownMenuProps) {
  const [isOpen, setIsOpen] = useState(false)
  const menuRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        setIsOpen(false)
      }
    }

    function handleEscape(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        setIsOpen(false)
      }
    }

    if (isOpen) {
      document.addEventListener('mousedown', handleClickOutside)
      document.addEventListener('keydown', handleEscape)
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside)
      document.removeEventListener('keydown', handleEscape)
    }
  }, [isOpen])

  const handleItemClick = (e: React.MouseEvent) => {
    // Close menu after clicking an item (unless it's the trigger button)
    const target = e.target as HTMLElement
    if (target.closest('.dropdown-item')) {
      setIsOpen(false)
    }
  }

  return (
    <div className="dropdown-menu" ref={menuRef}>
      <button
        className={`dropdown-trigger ${isOpen ? 'active' : ''}`}
        onClick={() => setIsOpen(!isOpen)}
      >
        {label}
        <svg className="dropdown-chevron" viewBox="0 0 12 12" fill="none">
          <path d="M3 4.5L6 7.5L9 4.5" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/>
        </svg>
      </button>
      {isOpen && (
        <div className="dropdown-content" onClick={handleItemClick}>
          {children}
        </div>
      )}
    </div>
  )
}
