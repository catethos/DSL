import { useEffect, useRef } from 'react'
import './WelcomeScreen.css'

interface WelcomeScreenProps {
  onNewNotebook: () => void
  onOpenFile: () => void
}

// Matrix-style symbols: Greek letters and math symbols
const MATRIX_SYMBOLS = [
  // Greek letters
  'α', 'β', 'γ', 'δ', 'ε', 'ζ', 'η', 'θ', 'ι', 'κ', 'λ', 'μ',
  'ν', 'ξ', 'π', 'ρ', 'σ', 'τ', 'υ', 'φ', 'χ', 'ψ', 'ω',
  'Γ', 'Δ', 'Θ', 'Λ', 'Ξ', 'Π', 'Σ', 'Φ', 'Ψ', 'Ω',
  // Math symbols
  '∫', '∂', '∇', '∞', '∑', '∏', '√', '∈', '∉', '⊂', '⊃',
  '∪', '∩', '∀', '∃', '∅', '⊕', '⊗', '≈', '≠', '≤', '≥',
  '±', '×', '÷', '→', '←', '↔', '⇒', '⇔', '∧', '∨', '¬',
  '∮', '∯', '∰', '⊥', '∥', '∠', '∡', '∢', '⊄', '⊅', '⊆', '⊇',
  'ℵ', 'ℶ', 'ℷ', 'ℸ', '℘', 'ℑ', 'ℜ', 'ℂ', 'ℕ', 'ℚ', 'ℝ', 'ℤ',
]

interface Column {
  x: number
  y: number
  speed: number
  symbols: string[]
  length: number
  opacity: number
}

export function WelcomeScreen({ onNewNotebook, onOpenFile }: WelcomeScreenProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null)

  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas) return

    const ctx = canvas.getContext('2d')
    if (!ctx) return

    // Configuration
    const fontSize = 18
    const columnWidth = fontSize * 1.2

    // Track dimensions
    let width = 0
    let height = 0
    let columns: Column[] = []

    // Set canvas size and recreate columns
    const resizeCanvas = () => {
      const dpr = window.devicePixelRatio || 1
      const newWidth = canvas.offsetWidth
      const newHeight = canvas.offsetHeight

      // Only rebuild if size actually changed
      if (newWidth === width && newHeight === height) return

      width = newWidth
      height = newHeight

      canvas.width = width * dpr
      canvas.height = height * dpr
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0)

      // Recreate columns for new width
      const numColumns = Math.ceil(width / columnWidth)
      columns = []
      for (let i = 0; i < numColumns; i++) {
        columns.push({
          x: i * columnWidth,
          y: Math.random() * height * -2,
          speed: 2 + Math.random() * 4,
          symbols: Array.from({ length: 25 + Math.floor(Math.random() * 15) }, () =>
            MATRIX_SYMBOLS[Math.floor(Math.random() * MATRIX_SYMBOLS.length)]
          ),
          length: 15 + Math.floor(Math.random() * 20),
          opacity: 0.5 + Math.random() * 0.5,
        })
      }
    }

    resizeCanvas()

    // Color palette matching app theme (warm botanical)
    const colors = {
      terracotta: { r: 193, g: 124, b: 90 },    // #c17c5a
      gold: { r: 201, g: 169, b: 87 },          // #c9a957
      sage: { r: 125, g: 156, b: 122 },         // #7d9c7a
      rust: { r: 166, g: 93, b: 63 },           // #a65d3f
      text: { r: 242, g: 235, b: 224 },         // #f2ebe0
    }
    const colorKeys = ['terracotta', 'gold', 'sage'] as const

    // Assign each column a color
    columns.forEach((col, i) => {
      (col as any).colorKey = colorKeys[i % colorKeys.length]
    })

    // Animation
    let animationId: number
    let lastTime = 0

    const animate = (currentTime: number) => {
      const deltaTime = currentTime - lastTime
      lastTime = currentTime

      // Clear canvas completely each frame
      ctx.clearRect(0, 0, width, height)
      ctx.fillStyle = '#1a1612'
      ctx.fillRect(0, 0, width, height)

      // Draw and update columns
      columns.forEach((column) => {
        // Update position
        column.y += column.speed * (deltaTime / 16)

        // Reset if fully off screen
        if (column.y - column.length * fontSize > height) {
          column.y = -column.length * fontSize - Math.random() * height * 0.5
          column.speed = 2 + Math.random() * 4
          column.opacity = 0.6 + Math.random() * 0.4
          // Shuffle symbols occasionally
          if (Math.random() > 0.7) {
            column.symbols = Array.from({ length: column.symbols.length }, () =>
              MATRIX_SYMBOLS[Math.floor(Math.random() * MATRIX_SYMBOLS.length)]
            )
          }
        }

        // Get column color
        const colorKey = (column as any).colorKey as keyof typeof colors
        const baseColor = colors[colorKey] || colors.gold

        // Draw symbols
        ctx.font = `bold ${fontSize}px "Fira Code", "JetBrains Mono", serif`
        ctx.textAlign = 'center'

        for (let i = 0; i < column.length; i++) {
          const y = column.y - i * fontSize
          if (y < -fontSize || y > height + fontSize) continue

          const symbolIndex = (Math.floor(column.y / fontSize) + i) % column.symbols.length
          const symbol = column.symbols[symbolIndex]

          // Calculate opacity based on position in trail
          const trailProgress = i / column.length
          const alpha = (1 - trailProgress * 0.8) * column.opacity

          // Leading character is brightest (cream/white)
          if (i === 0) {
            const { r, g, b } = colors.text
            ctx.fillStyle = `rgba(${r}, ${g}, ${b}, ${Math.min(1, alpha * 1.5)})`
            ctx.shadowColor = `rgb(${baseColor.r}, ${baseColor.g}, ${baseColor.b})`
            ctx.shadowBlur = 12
          } else if (i < 3) {
            // Near-leading characters are bright base color
            const { r, g, b } = baseColor
            ctx.fillStyle = `rgba(${r}, ${g}, ${b}, ${Math.min(1, alpha * 1.3)})`
            ctx.shadowColor = `rgb(${r}, ${g}, ${b})`
            ctx.shadowBlur = 8
          } else {
            // Rest fade to darker
            const { r, g, b } = baseColor
            const fade = 1 - trailProgress * 0.5
            ctx.fillStyle = `rgba(${Math.floor(r * fade)}, ${Math.floor(g * fade)}, ${Math.floor(b * fade)}, ${alpha})`
            ctx.shadowColor = 'transparent'
            ctx.shadowBlur = 0
          }

          ctx.fillText(symbol, column.x + columnWidth / 2, y)

          // Randomly change a symbol occasionally for that "living" effect
          if (Math.random() > 0.995) {
            column.symbols[symbolIndex] = MATRIX_SYMBOLS[Math.floor(Math.random() * MATRIX_SYMBOLS.length)]
          }
        }
      })

      animationId = requestAnimationFrame(animate)
    }

    animationId = requestAnimationFrame(animate)

    // Handle resize
    const handleResize = () => {
      resizeCanvas()
    }
    window.addEventListener('resize', handleResize)

    // Cleanup
    return () => {
      cancelAnimationFrame(animationId)
      window.removeEventListener('resize', handleResize)
    }
  }, [])

  return (
    <div className="welcome-screen">
      <canvas ref={canvasRef} className="matrix-canvas" />

      <div className="welcome-content">
        {/* Main title */}
        <div className="welcome-header">
          <h1 className="welcome-title">
            <span className="title-char" style={{ animationDelay: '0s' }}>L</span>
            <span className="title-char" style={{ animationDelay: '0.05s' }}>a</span>
            <span className="title-char" style={{ animationDelay: '0.1s' }}>t</span>
            <span className="title-char" style={{ animationDelay: '0.15s' }}>t</span>
            <span className="title-char" style={{ animationDelay: '0.2s' }}>i</span>
            <span className="title-char" style={{ animationDelay: '0.25s' }}>c</span>
            <span className="title-char" style={{ animationDelay: '0.3s' }}>e</span>
          </h1>
          <p className="welcome-subtitle">Computational Notebook</p>
        </div>

        {/* Tagline */}
        <p className="welcome-tagline">
          Where code meets computation
        </p>

        {/* Action buttons */}
        <div className="welcome-actions">
          <button className="welcome-action-btn primary" onClick={onNewNotebook}>
            <span className="action-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
                <path d="M12 5v14M5 12h14" />
              </svg>
            </span>
            <span className="action-content">
              <span className="action-label">New Notebook</span>
              <span className="action-hint">Create something new</span>
            </span>
            <kbd className="action-shortcut">N</kbd>
          </button>

          <button className="welcome-action-btn" onClick={onOpenFile}>
            <span className="action-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
                <path d="M3 7v13a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
              </svg>
            </span>
            <span className="action-content">
              <span className="action-label">Open File</span>
              <span className="action-hint">Continue your work</span>
            </span>
            <kbd className="action-shortcut">O</kbd>
          </button>
        </div>

        {/* Footer */}
        <div className="welcome-footer">
          <div className="footer-symbols">
            <span>∫</span>
            <span>∇</span>
            <span>∞</span>
          </div>
        </div>
      </div>
    </div>
  )
}
