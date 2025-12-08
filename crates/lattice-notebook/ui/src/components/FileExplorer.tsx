import { useState, useEffect, useCallback, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import './FileExplorer.css'
import { FileContextMenu, ContextMenuPosition } from './FileContextMenu'

// File entry from Rust backend
interface FileEntry {
  name: string
  path: string
  is_dir: boolean
  extension?: string
}

interface FileExplorerProps {
  onFileOpen: (path: string, type: 'notebook' | 'markdown' | 'lat') => void
  isCollapsed: boolean
  onToggleCollapse: () => void
}

interface TreeNode extends FileEntry {
  children?: TreeNode[]
  isExpanded?: boolean
  isLoading?: boolean
}

interface ContextMenuState {
  position: ContextMenuPosition
  node: TreeNode
  parentPath: string
}

interface RenameState {
  nodePath: string
  currentName: string
  parentPath: string
}

export function FileExplorer({ onFileOpen, isCollapsed, onToggleCollapse }: FileExplorerProps) {
  const [rootPath, setRootPath] = useState<string>('')
  const [tree, setTree] = useState<TreeNode[]>([])
  const [error, setError] = useState<string | null>(null)
  const [contextMenu, setContextMenu] = useState<ContextMenuState | null>(null)
  const [renameState, setRenameState] = useState<RenameState | null>(null)
  const [newItemState, setNewItemState] = useState<{ parentPath: string; type: 'file' | 'folder' } | null>(null)
  const renameInputRef = useRef<HTMLInputElement>(null)
  const newItemInputRef = useRef<HTMLInputElement>(null)

  // Load initial directory
  useEffect(() => {
    const loadInitialDirectory = async () => {
      try {
        const currentDir = await invoke<string>('get_current_directory')
        setRootPath(currentDir)
        const entries = await invoke<FileEntry[]>('list_directory', { path: currentDir })
        setTree(entries.map(e => ({ ...e, isExpanded: false })))
      } catch (e) {
        setError(String(e))
      }
    }
    loadInitialDirectory()
  }, [])

  // Load directory contents
  const loadDirectory = useCallback(async (path: string): Promise<TreeNode[]> => {
    try {
      const entries = await invoke<FileEntry[]>('list_directory', { path })
      return entries.map(e => ({ ...e, isExpanded: false }))
    } catch (e) {
      console.error('Failed to load directory:', e)
      return []
    }
  }, [])

  // Toggle directory expansion
  const toggleDirectory = useCallback(async (nodePath: string) => {
    const updateTree = async (nodes: TreeNode[]): Promise<TreeNode[]> => {
      const result: TreeNode[] = []
      for (const node of nodes) {
        if (node.path === nodePath) {
          if (node.isExpanded) {
            // Collapse
            result.push({ ...node, isExpanded: false })
          } else {
            // Expand - load children if needed
            if (!node.children) {
              const children = await loadDirectory(node.path)
              result.push({ ...node, isExpanded: true, children })
            } else {
              result.push({ ...node, isExpanded: true })
            }
          }
        } else if (node.children) {
          result.push({ ...node, children: await updateTree(node.children) })
        } else {
          result.push(node)
        }
      }
      return result
    }

    setTree(await updateTree(tree))
  }, [tree, loadDirectory])

  // Handle file click
  const handleFileClick = useCallback((node: TreeNode) => {
    if (node.is_dir) {
      toggleDirectory(node.path)
    } else {
      // Determine file type
      let type: 'notebook' | 'markdown' | 'lat' = 'lat'
      if (node.extension === 'lat.nb' || node.name.endsWith('.lat.nb')) {
        type = 'notebook'
      } else if (node.extension === 'md') {
        type = 'markdown'
      } else if (node.extension === 'lat') {
        type = 'lat'
      }
      onFileOpen(node.path, type)
    }
  }, [toggleDirectory, onFileOpen])

  // Navigate to parent directory
  const navigateUp = useCallback(async () => {
    const parentPath = rootPath.split('/').slice(0, -1).join('/')
    if (parentPath) {
      setRootPath(parentPath)
      const entries = await loadDirectory(parentPath)
      setTree(entries)
    }
  }, [rootPath, loadDirectory])

  // Navigate to home directory
  const navigateHome = useCallback(async () => {
    try {
      const homePath = await invoke<string>('get_home_directory')
      setRootPath(homePath)
      const entries = await loadDirectory(homePath)
      setTree(entries)
    } catch (e) {
      console.error('Failed to navigate home:', e)
    }
  }, [loadDirectory])

  // Navigate into a directory (set it as new root)
  const navigateInto = useCallback(async (dirPath: string) => {
    setRootPath(dirPath)
    const entries = await loadDirectory(dirPath)
    setTree(entries)
  }, [loadDirectory])

  // Refresh a specific directory in the tree
  const refreshDirectory = useCallback(async (dirPath: string) => {
    if (dirPath === rootPath) {
      const entries = await loadDirectory(dirPath)
      setTree(entries)
    } else {
      const updateTree = async (nodes: TreeNode[]): Promise<TreeNode[]> => {
        const result: TreeNode[] = []
        for (const node of nodes) {
          if (node.path === dirPath && node.is_dir) {
            const children = await loadDirectory(node.path)
            result.push({ ...node, children, isExpanded: true })
          } else if (node.children) {
            result.push({ ...node, children: await updateTree(node.children) })
          } else {
            result.push(node)
          }
        }
        return result
      }
      setTree(await updateTree(tree))
    }
  }, [rootPath, tree, loadDirectory])

  // Handle context menu
  const handleContextMenu = useCallback((e: React.MouseEvent, node: TreeNode, parentPath: string) => {
    e.preventDefault()
    e.stopPropagation()
    setContextMenu({
      position: { x: e.clientX, y: e.clientY },
      node,
      parentPath,
    })
  }, [])

  // Handle context menu on empty area (root directory)
  const handleRootContextMenu = useCallback((e: React.MouseEvent) => {
    e.preventDefault()
    setContextMenu({
      position: { x: e.clientX, y: e.clientY },
      node: { name: '', path: rootPath, is_dir: true },
      parentPath: rootPath,
    })
  }, [rootPath])

  // Close context menu
  const closeContextMenu = useCallback(() => {
    setContextMenu(null)
  }, [])

  // Create new file
  const handleNewFile = useCallback(() => {
    if (!contextMenu) return
    const parentPath = contextMenu.node.is_dir ? contextMenu.node.path : contextMenu.parentPath
    setNewItemState({ parentPath, type: 'file' })
    // Expand the parent directory if it's in the tree
    if (contextMenu.node.is_dir && contextMenu.node.path !== rootPath) {
      toggleDirectory(contextMenu.node.path)
    }
  }, [contextMenu, rootPath, toggleDirectory])

  // Create new folder
  const handleNewFolder = useCallback(() => {
    if (!contextMenu) return
    const parentPath = contextMenu.node.is_dir ? contextMenu.node.path : contextMenu.parentPath
    setNewItemState({ parentPath, type: 'folder' })
    // Expand the parent directory if it's in the tree
    if (contextMenu.node.is_dir && contextMenu.node.path !== rootPath) {
      toggleDirectory(contextMenu.node.path)
    }
  }, [contextMenu, rootPath, toggleDirectory])

  // Start rename
  const handleStartRename = useCallback(() => {
    if (!contextMenu) return
    setRenameState({
      nodePath: contextMenu.node.path,
      currentName: contextMenu.node.name,
      parentPath: contextMenu.parentPath,
    })
  }, [contextMenu])

  // Complete rename
  const handleCompleteRename = useCallback(async (newName: string) => {
    if (!renameState || !newName.trim() || newName === renameState.currentName) {
      setRenameState(null)
      return
    }

    try {
      const newPath = `${renameState.parentPath}/${newName}`
      await invoke('rename_path', { oldPath: renameState.nodePath, newPath })
      await refreshDirectory(renameState.parentPath)
    } catch (e) {
      console.error('Failed to rename:', e)
      setError(String(e))
    }
    setRenameState(null)
  }, [renameState, refreshDirectory])

  // Complete new item creation
  const handleCompleteNewItem = useCallback(async (name: string) => {
    if (!newItemState || !name.trim()) {
      setNewItemState(null)
      return
    }

    try {
      const newPath = `${newItemState.parentPath}/${name}`
      if (newItemState.type === 'folder') {
        await invoke('create_directory', { path: newPath })
      } else {
        await invoke('create_file', { path: newPath, content: null })
      }
      await refreshDirectory(newItemState.parentPath)
    } catch (e) {
      console.error('Failed to create:', e)
      setError(String(e))
    }
    setNewItemState(null)
  }, [newItemState, refreshDirectory])

  // Delete file or directory
  const handleDelete = useCallback(async () => {
    if (!contextMenu) return

    const confirmMessage = contextMenu.node.is_dir
      ? `Delete folder "${contextMenu.node.name}"? (must be empty)`
      : `Delete file "${contextMenu.node.name}"?`

    if (!confirm(confirmMessage)) return

    try {
      await invoke('delete_path', { path: contextMenu.node.path })
      await refreshDirectory(contextMenu.parentPath)
    } catch (e) {
      console.error('Failed to delete:', e)
      setError(String(e))
    }
  }, [contextMenu, refreshDirectory])

  // Focus rename input when it appears
  useEffect(() => {
    if (renameState && renameInputRef.current) {
      renameInputRef.current.focus()
      // Select the name part without extension
      const dotIndex = renameState.currentName.lastIndexOf('.')
      if (dotIndex > 0) {
        renameInputRef.current.setSelectionRange(0, dotIndex)
      } else {
        renameInputRef.current.select()
      }
    }
  }, [renameState])

  // Focus new item input when it appears
  useEffect(() => {
    if (newItemState && newItemInputRef.current) {
      newItemInputRef.current.focus()
    }
  }, [newItemState])

  // Render new item input row
  const renderNewItemInput = (parentPath: string, depth: number): JSX.Element | null => {
    if (!newItemState || newItemState.parentPath !== parentPath) return null

    const indent = depth * 16

    return (
      <div
        className="file-row file-row-input"
        style={{ paddingLeft: `${indent + 8}px` }}
      >
        <span className="file-icon">
          {newItemState.type === 'folder' ? '▶' : '+'}
        </span>
        <input
          ref={newItemInputRef}
          className="file-name-input"
          type="text"
          placeholder={newItemState.type === 'folder' ? 'folder name' : 'filename.lat'}
          onKeyDown={(e) => {
            if (e.key === 'Enter') {
              handleCompleteNewItem(e.currentTarget.value)
            } else if (e.key === 'Escape') {
              setNewItemState(null)
            }
          }}
          onBlur={(e) => handleCompleteNewItem(e.currentTarget.value)}
        />
      </div>
    )
  }

  // Render a tree node
  const renderNode = (node: TreeNode, depth: number = 0, parentPath: string = rootPath): JSX.Element => {
    const indent = depth * 16
    const isRenaming = renameState?.nodePath === node.path

    return (
      <div key={node.path} className="file-node">
        <div
          className={`file-row ${!node.is_dir ? 'file-item' : 'dir-item'}${isRenaming ? ' renaming' : ''}`}
          style={{ paddingLeft: `${indent + 8}px` }}
          onClick={() => !isRenaming && handleFileClick(node)}
          onDoubleClick={node.is_dir && !isRenaming ? () => navigateInto(node.path) : undefined}
          onContextMenu={(e) => handleContextMenu(e, node, parentPath)}
          title={node.path}
        >
          {node.is_dir ? (
            <>
              <span className="file-icon dir-icon">
                {node.isExpanded ? '▼' : '▶'}
              </span>
              {isRenaming ? (
                <input
                  ref={renameInputRef}
                  className="file-name-input"
                  type="text"
                  defaultValue={renameState.currentName}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') {
                      handleCompleteRename(e.currentTarget.value)
                    } else if (e.key === 'Escape') {
                      setRenameState(null)
                    }
                  }}
                  onBlur={(e) => handleCompleteRename(e.currentTarget.value)}
                  onClick={(e) => e.stopPropagation()}
                />
              ) : (
                <span className="file-name">{node.name}</span>
              )}
            </>
          ) : (
            <>
              <span className={`file-icon file-type-icon ${getFileTypeClass(node)}`}>
                {getFileIcon(node)}
              </span>
              {isRenaming ? (
                <input
                  ref={renameInputRef}
                  className="file-name-input"
                  type="text"
                  defaultValue={renameState.currentName}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') {
                      handleCompleteRename(e.currentTarget.value)
                    } else if (e.key === 'Escape') {
                      setRenameState(null)
                    }
                  }}
                  onBlur={(e) => handleCompleteRename(e.currentTarget.value)}
                  onClick={(e) => e.stopPropagation()}
                />
              ) : (
                <span className="file-name">{node.name}</span>
              )}
            </>
          )}
        </div>
        {node.is_dir && node.isExpanded && node.children && (
          <div className="file-children">
            {renderNewItemInput(node.path, depth + 1)}
            {node.children.map(child => renderNode(child, depth + 1, node.path))}
            {node.children.length === 0 && !newItemState && (
              <div className="file-row empty-dir" style={{ paddingLeft: `${indent + 24}px` }}>
                (empty)
              </div>
            )}
          </div>
        )}
      </div>
    )
  }

  // Get file type class for styling
  const getFileTypeClass = (node: TreeNode): string => {
    if (node.extension === 'lat.nb' || node.name.endsWith('.lat.nb')) {
      return 'type-notebook'
    } else if (node.extension === 'md') {
      return 'type-markdown'
    } else if (node.extension === 'lat') {
      return 'type-lat'
    }
    return ''
  }

  // Get file icon
  const getFileIcon = (node: TreeNode): string => {
    if (node.extension === 'lat.nb' || node.name.endsWith('.lat.nb')) {
      return 'nb'
    } else if (node.extension === 'md') {
      return 'fn'
    } else if (node.extension === 'lat') {
      return 'lat'
    }
    return '📄'
  }

  // Get current directory name
  const currentDirName = rootPath.split('/').pop() || rootPath

  if (isCollapsed) {
    return (
      <div className="file-explorer collapsed">
        <button className="expand-button" onClick={onToggleCollapse} title="Show file explorer">
          ▶
        </button>
      </div>
    )
  }

  return (
    <div className="file-explorer">
      <div className="file-explorer-header">
        <span className="file-explorer-title">Files</span>
        <div className="file-explorer-actions">
          <button
            className="file-explorer-btn"
            onClick={navigateHome}
            title="Go to home directory"
          >
            ⌂
          </button>
          <button
            className="file-explorer-btn"
            onClick={navigateUp}
            title="Go to parent directory"
          >
            ↑
          </button>
          <button
            className="file-explorer-btn"
            onClick={onToggleCollapse}
            title="Collapse file explorer"
          >
            ◀
          </button>
        </div>
      </div>
      <div className="file-explorer-path" title={rootPath} onContextMenu={handleRootContextMenu}>
        {currentDirName}
      </div>
      {error ? (
        <div className="file-explorer-error">{error}</div>
      ) : (
        <div className="file-tree" onContextMenu={handleRootContextMenu}>
          {renderNewItemInput(rootPath, 0)}
          {tree.map(node => renderNode(node))}
          {tree.length === 0 && !newItemState && (
            <div className="file-row empty-dir">No files found</div>
          )}
        </div>
      )}
      {contextMenu && (
        <FileContextMenu
          position={contextMenu.position}
          isDirectory={contextMenu.node.is_dir}
          onClose={closeContextMenu}
          onNewFile={handleNewFile}
          onNewFolder={handleNewFolder}
          onRename={handleStartRename}
          onDelete={handleDelete}
        />
      )}
    </div>
  )
}
