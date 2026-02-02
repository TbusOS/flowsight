"use client"

import * as React from "react"
import {
  Command,
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandSeparator,
} from "cmdk"
import {
  Search,
  File,
  Code,
  Command as CommandIcon,
  Settings,
  Terminal,
  Layout,
  Zap,
  BookOpen,
  FolderOpen,
  FileText,
  Folder,
  PanelRight,
  Keyboard,
  Palette,
  Moon,
  Sun,
  GitBranch,
  Download,
  Clock,
  ChevronRight,
  Play,
  LayoutGrid,
  Columns,
  SplitSquareHorizontal,
  PanelLeft,
  PanelBottom,
  HelpCircle,
} from "lucide-react"
import {
  Dialog,
  DialogContent,
  DialogTitle,
  DialogPortal,
  DialogOverlay,
} from "./dialog"
import { cn } from "../../lib/utils"
import { openDialog, invoke } from "../../lib/tauri-api"
import { useAtom, useSetAtom, useAtomValue } from "jotai"
import { 
  rightPanelOpenAtom, 
  rightPanelTabAtom, 
  currentFileAtom,
  viewModeAtom,
  leftPanelOpenAtom,
  bottomPanelOpenAtom,
  bottomPanelTabAtom,
  themeAtom,
  setThemeWithPersistenceAtom,
  type Theme,
} from "../../lib/atoms/layout-atoms"
import { useAnalysisStore } from "../../store/analysisStore"

interface CommandMenuProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  onShowKeyboardShortcuts?: () => void
}

// 主题信息
const themeOptions: Array<{ id: Theme; label: string; icon: React.ReactNode }> = [
  { id: "dark", label: "深色", icon: <Moon className="h-4 w-4" /> },
  { id: "light", label: "浅色", icon: <Sun className="h-4 w-4" /> },
  { id: "slate", label: "Slate", icon: <Palette className="h-4 w-4" /> },
  { id: "forest", label: "Forest", icon: <Palette className="h-4 w-4" /> },
  { id: "sunset", label: "Sunset", icon: <Palette className="h-4 w-4" /> },
  { id: "lavender", label: "Lavender", icon: <Palette className="h-4 w-4" /> },
]

// 命令项样式
const commandItemClass = "relative flex cursor-pointer select-none items-center rounded-md px-3 py-2.5 gap-2 text-sm outline-none aria-selected:bg-[var(--accent)]/10 aria-selected:text-[var(--accent)] data-[disabled]:pointer-events-none data-[disabled]:opacity-50 transition-colors"

export function CommandMenu({ open, onOpenChange, onShowKeyboardShortcuts }: CommandMenuProps) {
  const setRightPanelOpen = useSetAtom(rightPanelOpenAtom)
  const setRightPanelTab = useSetAtom(rightPanelTabAtom)
  const setCurrentFile = useSetAtom(currentFileAtom)
  const [viewMode, setViewMode] = useAtom(viewModeAtom)
  const [leftPanelOpen, setLeftPanelOpen] = useAtom(leftPanelOpenAtom)
  const [bottomPanelOpen, setBottomPanelOpen] = useAtom(bottomPanelOpenAtom)
  const setBottomPanelTab = useSetAtom(bottomPanelTabAtom)
  const currentTheme = useAtomValue(themeAtom)
  const setTheme = useSetAtom(setThemeWithPersistenceAtom)
  
  // 从 store 获取最近文件和项目信息
  const currentProject = useAnalysisStore(state => state.currentProject)
  const openProject = useAnalysisStore(state => state.openProject)
  const analyzeFile = useAnalysisStore(state => state.analyzeFile)
  
  // 子菜单状态
  const [subMenu, setSubMenu] = React.useState<"main" | "theme" | "view">("main")
  
  // 重置子菜单状态
  React.useEffect(() => {
    if (!open) {
      setSubMenu("main")
    }
  }, [open])

  // 关闭并执行
  const closeAndRun = React.useCallback((action: () => void) => {
    action()
    onOpenChange(false)
  }, [onOpenChange])

  // 打开文件浏览器面板
  const handleOpenExplorer = () => closeAndRun(() => {
    setRightPanelTab("explorer")
    setRightPanelOpen(true)
  })

  // 打开大纲面板
  const handleOpenOutline = () => closeAndRun(() => {
    setRightPanelTab("outline")
    setRightPanelOpen(true)
  })

  // 打开项目目录
  const handleOpenProject = async () => {
    try {
      const selected = await openDialog({
        directory: true,
        multiple: false,
        title: "选择项目目录",
      })
      if (selected && typeof selected === 'string') {
        await openProject(selected)
        onOpenChange(false)
      }
    } catch (e) {
      console.error('打开项目失败:', e)
    }
  }

  // 打开文件
  const handleOpenFile = async () => {
    try {
      const selected = await openDialog({
        multiple: false,
        title: "选择文件",
        filters: [
          { name: 'C/H 文件', extensions: ['c', 'h'] },
          { name: '所有文件', extensions: ['*'] },
        ],
      })
      if (selected && typeof selected === 'string') {
        setCurrentFile(selected)
        onOpenChange(false)
      }
    } catch (e) {
      console.error('打开文件失败:', e)
    }
  }

  // 切换视图
  const handleViewMode = (mode: "code" | "flow" | "split") => closeAndRun(() => {
    setViewMode(mode)
  })

  // 切换侧边栏
  const handleToggleSidebar = () => closeAndRun(() => {
    setLeftPanelOpen(!leftPanelOpen)
  })

  // 切换终端
  const handleToggleTerminal = () => closeAndRun(() => {
    setBottomPanelOpen(!bottomPanelOpen)
    if (!bottomPanelOpen) {
      setBottomPanelTab("terminal")
    }
  })

  // 切换主题
  const handleThemeChange = (theme: Theme) => closeAndRun(() => {
    setTheme(theme)
  })

  // 显示快捷键
  const handleShowKeyboardShortcuts = () => closeAndRun(() => {
    onShowKeyboardShortcuts?.()
  })

  // 主菜单内容
  const renderMainMenu = () => (
    <>
      {/* 文件操作 */}
      <CommandGroup heading="文件">
        <CommandItem onSelect={handleOpenProject} className={commandItemClass}>
          <FolderOpen className="h-4 w-4 text-[var(--text-muted)]" />
          <span>打开项目</span>
          <kbd className="ml-auto text-[10px] text-[var(--text-muted)]">⌘⇧O</kbd>
        </CommandItem>
        <CommandItem onSelect={handleOpenFile} className={commandItemClass}>
          <FileText className="h-4 w-4 text-[var(--text-muted)]" />
          <span>打开文件</span>
          <kbd className="ml-auto text-[10px] text-[var(--text-muted)]">⌘O</kbd>
        </CommandItem>
      </CommandGroup>

      <CommandSeparator className="my-1.5 h-px bg-[var(--border-light)]" />

      {/* 视图切换 */}
      <CommandGroup heading="视图">
        <CommandItem onSelect={() => handleViewMode("code")} className={commandItemClass}>
          <File className="h-4 w-4 text-[var(--text-muted)]" />
          <span>代码视图</span>
          {viewMode === "code" && <span className="ml-auto text-[var(--accent)]">✓</span>}
          <kbd className="ml-2 text-[10px] text-[var(--text-muted)]">⌘1</kbd>
        </CommandItem>
        <CommandItem onSelect={() => handleViewMode("split")} className={commandItemClass}>
          <SplitSquareHorizontal className="h-4 w-4 text-[var(--text-muted)]" />
          <span>分屏视图</span>
          {viewMode === "split" && <span className="ml-auto text-[var(--accent)]">✓</span>}
          <kbd className="ml-2 text-[10px] text-[var(--text-muted)]">⌘2</kbd>
        </CommandItem>
        <CommandItem onSelect={() => handleViewMode("flow")} className={commandItemClass}>
          <Zap className="h-4 w-4 text-[var(--text-muted)]" />
          <span>执行流视图</span>
          {viewMode === "flow" && <span className="ml-auto text-[var(--accent)]">✓</span>}
          <kbd className="ml-2 text-[10px] text-[var(--text-muted)]">⌘3</kbd>
        </CommandItem>
      </CommandGroup>

      <CommandSeparator className="my-1.5 h-px bg-[var(--border-light)]" />

      {/* 面板切换 */}
      <CommandGroup heading="面板">
        <CommandItem onSelect={handleToggleSidebar} className={commandItemClass}>
          <PanelLeft className="h-4 w-4 text-[var(--text-muted)]" />
          <span>{leftPanelOpen ? "隐藏" : "显示"}侧边栏</span>
          <kbd className="ml-auto text-[10px] text-[var(--text-muted)]">⌘B</kbd>
        </CommandItem>
        <CommandItem onSelect={handleToggleTerminal} className={commandItemClass}>
          <PanelBottom className="h-4 w-4 text-[var(--text-muted)]" />
          <span>{bottomPanelOpen ? "隐藏" : "显示"}终端</span>
          <kbd className="ml-auto text-[10px] text-[var(--text-muted)]">⌘J</kbd>
        </CommandItem>
        <CommandItem onSelect={handleOpenOutline} className={commandItemClass}>
          <BookOpen className="h-4 w-4 text-[var(--text-muted)]" />
          <span>大纲面板</span>
        </CommandItem>
        <CommandItem onSelect={handleOpenExplorer} className={commandItemClass}>
          <Folder className="h-4 w-4 text-[var(--text-muted)]" />
          <span>文件浏览器</span>
          <kbd className="ml-auto text-[10px] text-[var(--text-muted)]">⌘\</kbd>
        </CommandItem>
      </CommandGroup>

      <CommandSeparator className="my-1.5 h-px bg-[var(--border-light)]" />

      {/* 分析命令 */}
      <CommandGroup heading="分析">
        <CommandItem className={commandItemClass}>
          <Play className="h-4 w-4 text-[var(--text-muted)]" />
          <span>运行分析</span>
          <kbd className="ml-auto text-[10px] text-[var(--text-muted)]">⇧⌘A</kbd>
        </CommandItem>
        <CommandItem className={commandItemClass}>
          <GitBranch className="h-4 w-4 text-[var(--text-muted)]" />
          <span>查找调用链</span>
        </CommandItem>
        <CommandItem className={commandItemClass}>
          <Code className="h-4 w-4 text-[var(--text-muted)]" />
          <span>查找函数引用</span>
          <kbd className="ml-auto text-[10px] text-[var(--text-muted)]">⇧⌘F</kbd>
        </CommandItem>
        <CommandItem className={commandItemClass}>
          <Download className="h-4 w-4 text-[var(--text-muted)]" />
          <span>导出执行流</span>
          <kbd className="ml-auto text-[10px] text-[var(--text-muted)]">⌘E</kbd>
        </CommandItem>
      </CommandGroup>

      <CommandSeparator className="my-1.5 h-px bg-[var(--border-light)]" />

      {/* 设置 */}
      <CommandGroup heading="设置">
        <CommandItem onSelect={() => setSubMenu("theme")} className={commandItemClass}>
          <Palette className="h-4 w-4 text-[var(--text-muted)]" />
          <span>切换主题</span>
          <span className="ml-auto text-[10px] text-[var(--text-muted)] flex items-center gap-1">
            {themeOptions.find(t => t.id === currentTheme)?.label}
            <ChevronRight className="h-3 w-3" />
          </span>
        </CommandItem>
        <CommandItem onSelect={handleShowKeyboardShortcuts} className={commandItemClass}>
          <Keyboard className="h-4 w-4 text-[var(--text-muted)]" />
          <span>快捷键</span>
          <kbd className="ml-auto text-[10px] text-[var(--text-muted)]">?</kbd>
        </CommandItem>
        <CommandItem className={commandItemClass}>
          <Settings className="h-4 w-4 text-[var(--text-muted)]" />
          <span>设置</span>
          <kbd className="ml-auto text-[10px] text-[var(--text-muted)]">⌘,</kbd>
        </CommandItem>
        <CommandItem className={commandItemClass}>
          <HelpCircle className="h-4 w-4 text-[var(--text-muted)]" />
          <span>帮助文档</span>
        </CommandItem>
      </CommandGroup>
    </>
  )

  // 主题子菜单
  const renderThemeMenu = () => (
    <CommandGroup heading="选择主题">
      <CommandItem 
        onSelect={() => setSubMenu("main")} 
        className={cn(commandItemClass, "text-[var(--text-muted)]")}
      >
        <ChevronRight className="h-4 w-4 rotate-180" />
        <span>返回</span>
      </CommandItem>
      <CommandSeparator className="my-1.5 h-px bg-[var(--border-light)]" />
      {themeOptions.map((theme) => (
        <CommandItem 
          key={theme.id}
          onSelect={() => handleThemeChange(theme.id)} 
          className={commandItemClass}
        >
          {theme.icon}
          <span>{theme.label}</span>
          {currentTheme === theme.id && (
            <span className="ml-auto text-[var(--accent)]">✓</span>
          )}
        </CommandItem>
      ))}
    </CommandGroup>
  )

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogPortal>
        <DialogOverlay className="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm animate-in fade-in-0" />
        <DialogContent className="fixed left-1/2 top-[15%] z-50 w-full max-w-[560px] -translate-x-1/2 animate-in fade-in-0 zoom-in-95 slide-in-from-top-4 rounded-xl border border-[var(--border-light)] bg-[var(--bg-secondary)] shadow-2xl shadow-black/50 overflow-hidden">
          <DialogTitle className="sr-only">命令面板</DialogTitle>
          <Command className="[&_[cmdk-group-heading]]:px-3 [&_[cmdk-group-heading]]:py-2 [&_[cmdk-group-heading]]:text-[10px] [&_[cmdk-group-heading]]:font-medium [&_[cmdk-group-heading]]:uppercase [&_[cmdk-group-heading]]:tracking-wider [&_[cmdk-group-heading]]:text-[var(--text-muted)]">
            {/* 搜索框 */}
            <div className="flex items-center border-b border-[var(--border-light)] px-4 bg-[var(--bg-primary)]">
              <Search className="h-4 w-4 shrink-0 text-[var(--text-muted)]" />
              <CommandInput
                placeholder={subMenu === "main" ? "搜索命令..." : "搜索主题..."}
                className="flex h-12 w-full bg-transparent px-3 py-3 text-sm outline-none placeholder:text-[var(--text-muted)]"
              />
              <div className="flex items-center gap-1.5">
                <kbd className="inline-flex h-5 items-center gap-0.5 rounded border border-[var(--border-light)] bg-[var(--bg-tertiary)] px-1.5 font-mono text-[10px]">
                  <span>⌘</span>K
                </kbd>
              </div>
            </div>

            {/* 当前项目信息 */}
            {currentProject && subMenu === "main" && (
              <div className="px-4 py-2 border-b border-[var(--border-light)] bg-[var(--bg-tertiary)]/50">
                <div className="flex items-center gap-2 text-xs text-[var(--text-muted)]">
                  <FolderOpen className="h-3.5 w-3.5" />
                  <span className="truncate">{currentProject.path.split('/').pop()}</span>
                  {currentProject.indexed && (
                    <span className="text-emerald-400 ml-auto">
                      {currentProject.functions_count} 函数
                    </span>
                  )}
                </div>
              </div>
            )}

            {/* 命令列表 */}
            <CommandList className="max-h-[400px] overflow-y-auto py-2 px-2">
              <CommandEmpty className="py-8 text-center text-sm text-[var(--text-muted)]">
                无匹配结果
              </CommandEmpty>
              
              {subMenu === "main" && renderMainMenu()}
              {subMenu === "theme" && renderThemeMenu()}
            </CommandList>

            {/* 底部提示 */}
            <div className="flex items-center justify-between px-4 py-2 border-t border-[var(--border-light)] bg-[var(--bg-tertiary)]/50 text-[10px] text-[var(--text-muted)]">
              <div className="flex items-center gap-3">
                <span className="flex items-center gap-1">
                  <kbd className="px-1 py-0.5 rounded bg-[var(--bg-secondary)] border border-[var(--border-light)]">↑↓</kbd>
                  导航
                </span>
                <span className="flex items-center gap-1">
                  <kbd className="px-1 py-0.5 rounded bg-[var(--bg-secondary)] border border-[var(--border-light)]">↵</kbd>
                  选择
                </span>
                <span className="flex items-center gap-1">
                  <kbd className="px-1 py-0.5 rounded bg-[var(--bg-secondary)] border border-[var(--border-light)]">esc</kbd>
                  关闭
                </span>
              </div>
              <span>FlowSight v0.2.0</span>
            </div>
          </Command>
        </DialogContent>
      </DialogPortal>
    </Dialog>
  )
}

export { CommandMenu as Command }
