import { useRef, useEffect } from 'react'
import './TabBar.css'

export interface Tab {
  id: string
  sessionId: string
  title: string
  filePath: string | null
  isDirty: boolean
}

interface TabBarProps {
  tabs: Tab[]
  activeTabId: string
  onSelectTab: (tabId: string) => void
  onCloseTab: (tabId: string) => void
  onNewTab: () => void
}

export function TabBar({ tabs, activeTabId, onSelectTab, onCloseTab, onNewTab }: TabBarProps) {
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
      return filename.replace('.lat.nb', '')
    }
    return tab.title
  }

  return (
    <div className="tab-bar">
      <div className="tab-list" ref={scrollRef}>
        {tabs.map((tab) => (
          <div
            key={tab.id}
            className={`tab ${tab.id === activeTabId ? 'active' : ''}`}
            onClick={() => handleTabClick(tab.id)}
          >
            <span className="tab-title">
              {tab.isDirty && <span className="dirty-indicator">*</span>}
              {getTabTitle(tab)}
            </span>
            {tabs.length > 1 && (
              <button
                className="tab-close"
                onClick={(e) => handleCloseClick(e, tab.id)}
                title="Close tab"
              >
                ×
              </button>
            )}
          </div>
        ))}
      </div>
      <button className="new-tab-button" onClick={onNewTab} title="New tab">
        +
      </button>
    </div>
  )
}
