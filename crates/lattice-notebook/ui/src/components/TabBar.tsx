import { useRef, useEffect, useState } from 'react'
import './TabBar.css'

export interface Tab {
  id: string
  title: string
  filePath: string | null
  isDirty: boolean
  type: 'notebook' | 'markdown' | 'lat'
}

type LayoutMode = 'single' | 'split-horizontal' | 'split-vertical'

interface TabBarProps {
  tabs: Tab[]
  activeTabId: string
  onSelectTab: (tabId: string) => void
  onCloseTab: (tabId: string) => void
  onNewTab: () => void
  // Split view props
  layoutMode?: LayoutMode
  leftPaneTabId?: string | null
  rightPaneTabId?: string | null
  onOpenInPane?: (tabId: string, pane: 'left' | 'right') => void
}

export function TabBar({
  tabs,
  activeTabId,
  onSelectTab,
  onCloseTab,
  onNewTab,
  layoutMode = 'single',
  leftPaneTabId,
  rightPaneTabId,
  onOpenInPane,
}: TabBarProps) {
  const scrollRef = useRef<HTMLDivElement>(null)

  // Scroll active tab into view
  useEffect(() => {
    if (scrollRef.current) {
      const activeTab = scrollRef.current.querySelector('.tab.active')
      if (activeTab) {
        activeTab.scrollIntoView({ behavior: 'smooth', block: 'nearest', inline: 'center' })
      }
    }
  }, [activeTabId])

  const handleTabClick = (tabId: string) => {
    onSelectTab(tabId)
  }

  const handleCloseClick = (e: React.MouseEvent, tabId: string) => {
    e.stopPropagation()
    onCloseTab(tabId)
  }

  const getTabTitle = (tab: Tab) => {
    if (tab.filePath) {
      const filename = tab.filePath.split('/').pop() || tab.filePath
      if (tab.type === 'notebook') {
        return filename.replace('.lat.nb', '')
      } else if (tab.type === 'lat') {
        return filename.replace('.lat', '')
      } else {
        return filename.replace('.md', '')
      }
    }
    return tab.title
  }

  // Icon for tab type
  const getTabIcon = (tab: Tab) => {
    if (tab.type === 'markdown') {
      return <span className="tab-icon md-icon">fn</span>
    }
    if (tab.type === 'lat') {
      return <span className="tab-icon lat-icon">lat</span>
    }
    return <span className="tab-icon nb-icon">nb</span>
  }

  // Get pane indicator for split mode
  const getPaneIndicator = (tabId: string) => {
    if (layoutMode === 'single') return null
    if (tabId === leftPaneTabId && tabId === rightPaneTabId) {
      return <span className="pane-indicator both">L+R</span>
    }
    if (tabId === leftPaneTabId) {
      return <span className="pane-indicator left">L</span>
    }
    if (tabId === rightPaneTabId) {
      return <span className="pane-indicator right">R</span>
    }
    return null
  }

  // Context menu state
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number; tabId: string } | null>(null)

  const handleContextMenu = (e: React.MouseEvent, tabId: string) => {
    e.preventDefault()
    setContextMenu({ x: e.clientX, y: e.clientY, tabId })
  }

  const closeContextMenu = () => {
    setContextMenu(null)
  }

  // Close context menu on click outside
  useEffect(() => {
    if (contextMenu) {
      const handleClick = () => closeContextMenu()
      document.addEventListener('click', handleClick)
      return () => document.removeEventListener('click', handleClick)
    }
  }, [contextMenu])

  return (
    <div className="tab-bar">
      <div className="tab-list" ref={scrollRef}>
        {tabs.map((tab) => (
          <div
            key={tab.id}
            className={`tab ${tab.id === activeTabId ? 'active' : ''} ${tab.type === 'markdown' ? 'md-file' : ''}`}
            onClick={() => handleTabClick(tab.id)}
            onContextMenu={(e) => handleContextMenu(e, tab.id)}
          >
            {getTabIcon(tab)}
            <span className="tab-title">
              {tab.isDirty && <span className="dirty-indicator">*</span>}
              {getTabTitle(tab)}
            </span>
            {getPaneIndicator(tab.id)}
            <button
              className="tab-close"
              onClick={(e) => handleCloseClick(e, tab.id)}
              title="Close tab"
            >
              ×
            </button>
          </div>
        ))}
      </div>
      <button className="new-tab-button" onClick={onNewTab} title="New tab">
        +
      </button>

      {/* Context menu for split options */}
      {contextMenu && onOpenInPane && (
        <div
          className="tab-context-menu"
          style={{ left: contextMenu.x, top: contextMenu.y }}
        >
          <button
            className="context-menu-item"
            onClick={() => {
              onOpenInPane(contextMenu.tabId, 'left')
              closeContextMenu()
            }}
          >
            Open in Left Pane
          </button>
          <button
            className="context-menu-item"
            onClick={() => {
              onOpenInPane(contextMenu.tabId, 'right')
              closeContextMenu()
            }}
          >
            Open in Right Pane
          </button>
        </div>
      )}
    </div>
  )
}
