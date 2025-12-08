import { useRef, useCallback, useEffect, useState } from 'react'
import './SplitContainer.css'

export type SplitDirection = 'horizontal' | 'vertical'

interface SplitContainerProps {
  left: React.ReactNode
  right: React.ReactNode
  direction: SplitDirection
  ratio: number // 0-1, percentage of space for left/top pane
  onRatioChange: (ratio: number) => void
  minRatio?: number // minimum ratio (default 0.2)
  maxRatio?: number // maximum ratio (default 0.8)
}

export function SplitContainer({
  left,
  right,
  direction,
  ratio,
  onRatioChange,
  minRatio = 0.2,
  maxRatio = 0.8,
}: SplitContainerProps) {
  const containerRef = useRef<HTMLDivElement>(null)
  const [isDragging, setIsDragging] = useState(false)

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault()
    setIsDragging(true)
  }, [])

  useEffect(() => {
    if (!isDragging) return

    const handleMouseMove = (e: MouseEvent) => {
      if (!containerRef.current) return

      const rect = containerRef.current.getBoundingClientRect()
      let newRatio: number

      if (direction === 'horizontal') {
        newRatio = (e.clientX - rect.left) / rect.width
      } else {
        newRatio = (e.clientY - rect.top) / rect.height
      }

      // Clamp to min/max
      newRatio = Math.max(minRatio, Math.min(maxRatio, newRatio))
      onRatioChange(newRatio)
    }

    const handleMouseUp = () => {
      setIsDragging(false)
    }

    document.addEventListener('mousemove', handleMouseMove)
    document.addEventListener('mouseup', handleMouseUp)

    return () => {
      document.removeEventListener('mousemove', handleMouseMove)
      document.removeEventListener('mouseup', handleMouseUp)
    }
  }, [isDragging, direction, minRatio, maxRatio, onRatioChange])

  const leftStyle = direction === 'horizontal'
    ? { width: `${ratio * 100}%` }
    : { height: `${ratio * 100}%` }

  const rightStyle = direction === 'horizontal'
    ? { width: `${(1 - ratio) * 100}%` }
    : { height: `${(1 - ratio) * 100}%` }

  return (
    <div
      ref={containerRef}
      className={`split-container split-${direction} ${isDragging ? 'dragging' : ''}`}
    >
      <div className="split-pane split-pane-left" style={leftStyle}>
        {left}
      </div>
      <div
        className={`split-divider split-divider-${direction}`}
        onMouseDown={handleMouseDown}
      >
        <div className="split-divider-handle" />
      </div>
      <div className="split-pane split-pane-right" style={rightStyle}>
        {right}
      </div>
    </div>
  )
}
