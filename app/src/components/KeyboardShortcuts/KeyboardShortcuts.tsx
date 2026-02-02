/**
 * 快捷键帮助面板 - 现代化 UI
 */

import * as React from "react"
import { X, Keyboard, Command, Search } from "lucide-react"
import { cn } from "../../lib/utils"
import { useDebounce } from "../../hooks/usePerformance"

interface KeyboardShortcutsProps {
  isOpen: boolean
  onClose: () => void
}

interface ShortcutItem {
  keys: string[]
  description: string
}

interface ShortcutSection {
  category: string
  icon: React.ReactNode
  items: ShortcutItem[]
}

const isMac = typeof navigator !== "undefined" && navigator.platform.toUpperCase().indexOf("MAC") >= 0

// 根据平台显示正确的修饰键
const mod = isMac ? "⌘" : "Ctrl"
const alt = isMac ? "⌥" : "Alt"
const shift = "⇧"

const shortcuts: ShortcutSection[] = [
  {
    category: "命令面板",
    icon: <Command className="h-4 w-4" />,
    items: [
      { keys: [mod, "K"], description: "打开命令面板" },
      { keys: [mod, "P"], description: "快速打开文件" },
      { keys: ["?"], description: "显示快捷键帮助" },
    ]
  },
  {
    category: "文件操作",
    icon: <span>📁</span>,
    items: [
      { keys: [mod, "S"], description: "保存当前文件" },
      { keys: [mod, "O"], description: "打开文件" },
      { keys: [mod, shift, "O"], description: "打开项目" },
      { keys: [mod, "W"], description: "关闭当前标签" },
    ]
  },
  {
    category: "导航",
    icon: <span>🧭</span>,
    items: [
      { keys: [mod, "G"], description: "跳转到行号" },
      { keys: [alt, "←"], description: "后退" },
      { keys: [alt, "→"], description: "前进" },
      { keys: ["F12"], description: "跳转到定义" },
    ]
  },
  {
    category: "编辑",
    icon: <span>✏️</span>,
    items: [
      { keys: [mod, "F"], description: "查找" },
      { keys: [mod, "H"], description: "查找替换" },
      { keys: [mod, "Z"], description: "撤销" },
      { keys: [mod, shift, "Z"], description: "重做" },
      { keys: [mod, "D"], description: "选择下一个匹配" },
      { keys: [mod, "/"], description: "切换注释" },
    ]
  },
  {
    category: "视图切换",
    icon: <span>👁️</span>,
    items: [
      { keys: [mod, "B"], description: "切换侧边栏" },
      { keys: [mod, "1"], description: "代码视图" },
      { keys: [mod, "2"], description: "分屏视图" },
      { keys: [mod, "3"], description: "执行流视图" },
      { keys: [mod, "J"], description: "切换底部面板" },
    ]
  },
  {
    category: "执行流图",
    icon: <span>🔄</span>,
    items: [
      { keys: ["1-5"], description: "折叠到对应深度" },
      { keys: ["0"], description: "展开全部" },
      { keys: ["右键"], description: "聚焦子树" },
      { keys: ["Esc"], description: "清除聚焦" },
      { keys: ["滚轮"], description: "缩放视图" },
      { keys: ["拖拽"], description: "平移视图" },
    ]
  },
  {
    category: "分析",
    icon: <span>🔬</span>,
    items: [
      { keys: [mod, shift, "A"], description: "分析当前函数" },
      { keys: [mod, "E"], description: "导出执行流" },
    ]
  }
]

// 单个快捷键项 - memoized
const ShortcutRow = React.memo(function ShortcutRow({ item }: { item: ShortcutItem }) {
  return (
    <div className="flex items-center justify-between py-1.5 px-2 rounded hover:bg-[var(--bg-hover)] transition-colors">
      <span className="text-xs text-[var(--text-secondary)]">{item.description}</span>
      <div className="flex items-center gap-1">
        {item.keys.map((key, k) => (
          <React.Fragment key={k}>
            <kbd className="px-1.5 py-0.5 text-[10px] font-mono bg-[var(--bg-tertiary)] border border-[var(--border-light)] rounded text-[var(--text-primary)] min-w-[20px] text-center">
              {key}
            </kbd>
            {k < item.keys.length - 1 && (
              <span className="text-[10px] text-[var(--text-muted)]">+</span>
            )}
          </React.Fragment>
        ))}
      </div>
    </div>
  )
})

// 快捷键分组 - memoized
const ShortcutGroup = React.memo(function ShortcutGroup({ 
  section, 
  searchQuery 
}: { 
  section: ShortcutSection
  searchQuery: string 
}) {
  const filteredItems = React.useMemo(() => {
    if (!searchQuery) return section.items
    const query = searchQuery.toLowerCase()
    return section.items.filter(item => 
      item.description.toLowerCase().includes(query) ||
      item.keys.some(k => k.toLowerCase().includes(query))
    )
  }, [section.items, searchQuery])

  if (filteredItems.length === 0) return null

  return (
    <div className="mb-4">
      <div className="flex items-center gap-2 mb-2 px-2">
        {section.icon}
        <h3 className="text-xs font-medium text-[var(--text-primary)]">{section.category}</h3>
        <span className="text-[10px] text-[var(--text-muted)]">({filteredItems.length})</span>
      </div>
      <div className="space-y-0.5">
        {filteredItems.map((item, j) => (
          <ShortcutRow key={j} item={item} />
        ))}
      </div>
    </div>
  )
})

export function KeyboardShortcuts({ isOpen, onClose }: KeyboardShortcutsProps) {
  const [searchText, setSearchText] = React.useState("")
  const debouncedSearch = useDebounce(searchText, 150)
  const inputRef = React.useRef<HTMLInputElement>(null)

  // 关闭时清空搜索
  React.useEffect(() => {
    if (!isOpen) {
      setSearchText("")
    } else {
      // 打开时聚焦搜索框
      setTimeout(() => inputRef.current?.focus(), 100)
    }
  }, [isOpen])

  // ESC 关闭
  React.useEffect(() => {
    if (!isOpen) return
    
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault()
        onClose()
      }
    }
    
    window.addEventListener("keydown", handleKeyDown)
    return () => window.removeEventListener("keydown", handleKeyDown)
  }, [isOpen, onClose])

  if (!isOpen) return null

  const totalShortcuts = shortcuts.reduce((acc, s) => acc + s.items.length, 0)

  return (
    <div 
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
      onClick={onClose}
    >
      <div 
        className="w-[600px] max-h-[80vh] bg-[var(--bg-primary)] border border-[var(--border-subtle)] rounded-xl shadow-2xl overflow-hidden flex flex-col"
        onClick={e => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-4 py-3 border-b border-[var(--border-subtle)] bg-[var(--bg-secondary)]">
          <div className="flex items-center gap-2">
            <Keyboard className="h-5 w-5 text-[var(--accent)]" />
            <h2 className="text-sm font-medium text-[var(--text-primary)]">快捷键</h2>
            <span className="text-[10px] px-1.5 py-0.5 rounded-full bg-[var(--bg-tertiary)] text-[var(--text-muted)]">
              {totalShortcuts} 个
            </span>
          </div>
          <button 
            onClick={onClose}
            className="p-1 rounded hover:bg-[var(--bg-hover)] text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors"
          >
            <X className="h-4 w-4" />
          </button>
        </div>
        
        {/* Search */}
        <div className="px-4 py-2 border-b border-[var(--border-subtle)]">
          <div className="relative">
            <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-[var(--text-muted)]" />
            <input
              ref={inputRef}
              type="text"
              value={searchText}
              onChange={(e) => setSearchText(e.target.value)}
              placeholder="搜索快捷键..."
              className="w-full h-8 pl-8 pr-3 text-xs bg-[var(--bg-tertiary)] border border-[var(--border-light)] rounded-md text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)]"
            />
          </div>
        </div>
        
        {/* Content */}
        <div className="flex-1 overflow-y-auto p-4">
          <div className="grid grid-cols-2 gap-4">
            <div>
              {shortcuts.slice(0, 4).map((section, i) => (
                <ShortcutGroup key={i} section={section} searchQuery={debouncedSearch} />
              ))}
            </div>
            <div>
              {shortcuts.slice(4).map((section, i) => (
                <ShortcutGroup key={i + 4} section={section} searchQuery={debouncedSearch} />
              ))}
            </div>
          </div>
        </div>
        
        {/* Footer */}
        <div className="px-4 py-2 border-t border-[var(--border-subtle)] bg-[var(--bg-secondary)]">
          <div className="flex items-center justify-between text-[10px] text-[var(--text-muted)]">
            <span>
              提示: 按 <kbd className="px-1 py-0.5 bg-[var(--bg-tertiary)] border border-[var(--border-light)] rounded">?</kbd> 可快速打开此面板
            </span>
            <span>
              平台: {isMac ? "macOS" : "Windows/Linux"}
            </span>
          </div>
        </div>
      </div>
    </div>
  )
}

export default KeyboardShortcuts
