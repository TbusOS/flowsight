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
import { useSetAtom } from "jotai"
import { rightPanelOpenAtom, rightPanelTabAtom, currentFileAtom } from "../../lib/atoms/layout-atoms"

interface CommandMenuProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function CommandMenu({ open, onOpenChange }: CommandMenuProps) {
  const setRightPanelOpen = useSetAtom(rightPanelOpenAtom)
  const setRightPanelTab = useSetAtom(rightPanelTabAtom)
  const setCurrentFile = useSetAtom(currentFileAtom)

  // 打开文件浏览器面板
  const handleOpenExplorer = () => {
    setRightPanelTab("explorer")
    setRightPanelOpen(true)
    onOpenChange(false)
  }

  // 打开大纲面板
  const handleOpenOutline = () => {
    setRightPanelTab("outline")
    setRightPanelOpen(true)
    onOpenChange(false)
  }

  // 打开项目目录
  const handleOpenProject = async () => {
    try {
      const selected = await openDialog({
        directory: true,
        multiple: false,
        title: "选择项目目录",
      })
      if (selected && typeof selected === 'string') {
        // 调用后端打开项目
        await invoke('open_project', { path: selected })
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
        // 在代码编辑器中打开文件
        setCurrentFile(selected)
        onOpenChange(false)
        console.log('打开文件:', selected)
      }
    } catch (e) {
      console.error('打开文件失败:', e)
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogPortal>
        <DialogOverlay className="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm animate-in fade-in-0" />
        <DialogContent className="fixed left-1/2 top-[20%] z-50 w-full max-w-[640px] -translate-x-1/2 animate-in fade-in-0 zoom-in-95 slide-in-from-top-4 rounded-xl border border-[var(--border-light)] bg-[var(--bg-secondary)] shadow-2xl shadow-black/50">
          <DialogTitle className="sr-only">
            命令面板
          </DialogTitle>
          <Command className="[&_[cmdk-group]]:px-2 [&_[cmdk-group]]:py-1.5 [&_[cmdk-group]]:text-xs [&_[cmdk-group]]:font-medium [&_[cmdk-group]]:text-[var(--text-muted)]">
            <div className="flex items-center border-b border-[var(--border-light)] px-3">
              <Search className="mr-2 h-4 w-4 shrink-0 opacity-50" />
              <CommandInput
                placeholder="搜索文件、函数、命令..."
                className="flex h-12 w-full rounded-md bg-transparent py-3 text-sm outline-none placeholder:text-[var(--text-muted)] disabled:cursor-not-allowed disabled:opacity-50"
              />
              <kbd className="pointer-events-none inline-flex h-5 select-none items-center gap-1 rounded border border-[var(--border-light)] bg-[var(--bg-tertiary)] px-1.5 font-mono text-[10px] font-medium opacity-100">
                <span className="text-xs">⌘</span>K
              </kbd>
            </div>
            <CommandList className="max-h-[360px] overflow-y-auto overflow-x-hidden py-2 px-1">
              <CommandEmpty className="py-6 text-center text-sm text-[var(--text-muted)]">
                无结果
              </CommandEmpty>
              <CommandGroup heading="文件操作">
                <CommandItem 
                  onSelect={handleOpenProject}
                  className="relative flex cursor-pointer select-none items-center rounded-md px-3 py-2.5 gap-2 text-sm outline-none aria-selected:bg-[var(--bg-tertiary)] aria-selected:text-[var(--text-primary)] data-[disabled]:pointer-events-none data-[disabled]:opacity-50 transition-colors duration-100"
                >
                  <FolderOpen className="mr-2 h-4 w-4" />
                  <span>打开项目</span>
                  <kbd className="ml-auto text-xs text-[var(--text-muted)]">⌘⇧O</kbd>
                </CommandItem>
                <CommandItem 
                  onSelect={handleOpenFile}
                  className="relative flex cursor-pointer select-none items-center rounded-md px-3 py-2.5 gap-2 text-sm outline-none aria-selected:bg-[var(--bg-tertiary)] aria-selected:text-[var(--text-primary)] data-[disabled]:pointer-events-none data-[disabled]:opacity-50 transition-colors duration-100"
                >
                  <FileText className="mr-2 h-4 w-4" />
                  <span>打开文件</span>
                  <kbd className="ml-auto text-xs text-[var(--text-muted)]">⌘O</kbd>
                </CommandItem>
              </CommandGroup>
              <CommandSeparator className="my-2 -mx-2 h-px bg-[var(--border-light)]" />
              <CommandGroup heading="快速操作">
                <CommandItem className="relative flex cursor-pointer select-none items-center rounded-md px-3 py-2.5 gap-2 text-sm outline-none aria-selected:bg-[var(--bg-tertiary)] aria-selected:text-[var(--text-primary)] data-[disabled]:pointer-events-none data-[disabled]:opacity-50 transition-colors duration-100">
                  <Layout className="mr-2 h-4 w-4" />
                  <span>切换侧边栏</span>
                  <kbd className="ml-auto text-xs text-[var(--text-muted)]">⌘B</kbd>
                </CommandItem>
                <CommandItem className="relative flex cursor-pointer select-none items-center rounded-md px-3 py-2.5 gap-2 text-sm outline-none aria-selected:bg-[var(--bg-tertiary)] aria-selected:text-[var(--text-primary)] data-[disabled]:pointer-events-none data-[disabled]:opacity-50 transition-colors duration-100">
                  <Terminal className="mr-2 h-4 w-4" />
                  <span>切换终端</span>
                  <kbd className="ml-auto text-xs text-[var(--text-muted)]">⌘J</kbd>
                </CommandItem>
                <CommandItem className="relative flex cursor-pointer select-none items-center rounded-md px-3 py-2.5 gap-2 text-sm outline-none aria-selected:bg-[var(--bg-tertiary)] aria-selected:text-[var(--text-primary)] data-[disabled]:pointer-events-none data-[disabled]:opacity-50 transition-colors duration-100">
                  <Settings className="mr-2 h-4 w-4" />
                  <span>打开设置</span>
                  <kbd className="ml-auto text-xs text-[var(--text-muted)]">⌘,</kbd>
                </CommandItem>
              </CommandGroup>
              <CommandSeparator className="my-2 -mx-2 h-px bg-[var(--border-light)]" />
              <CommandGroup heading="视图">
                <CommandItem className="relative flex cursor-pointer select-none items-center rounded-md px-3 py-2.5 gap-2 text-sm outline-none aria-selected:bg-[var(--bg-tertiary)] aria-selected:text-[var(--text-primary)] data-[disabled]:pointer-events-none data-[disabled]:opacity-50 transition-colors duration-100">
                  <File className="mr-2 h-4 w-4" />
                  <span>代码视图</span>
                </CommandItem>
                <CommandItem className="relative flex cursor-pointer select-none items-center rounded-md px-3 py-2.5 gap-2 text-sm outline-none aria-selected:bg-[var(--bg-tertiary)] aria-selected:text-[var(--text-primary)] data-[disabled]:pointer-events-none data-[disabled]:opacity-50 transition-colors duration-100">
                  <Zap className="mr-2 h-4 w-4" />
                  <span>执行流视图</span>
                </CommandItem>
                <CommandItem 
                  onSelect={handleOpenOutline}
                  className="relative flex cursor-pointer select-none items-center rounded-md px-3 py-2.5 gap-2 text-sm outline-none aria-selected:bg-[var(--bg-tertiary)] aria-selected:text-[var(--text-primary)] data-[disabled]:pointer-events-none data-[disabled]:opacity-50 transition-colors duration-100"
                >
                  <BookOpen className="mr-2 h-4 w-4" />
                  <span>大纲视图</span>
                </CommandItem>
                <CommandItem 
                  onSelect={handleOpenExplorer}
                  className="relative flex cursor-pointer select-none items-center rounded-md px-3 py-2.5 gap-2 text-sm outline-none aria-selected:bg-[var(--bg-tertiary)] aria-selected:text-[var(--text-primary)] data-[disabled]:pointer-events-none data-[disabled]:opacity-50 transition-colors duration-100"
                >
                  <Folder className="mr-2 h-4 w-4" />
                  <span>文件浏览器</span>
                  <kbd className="ml-auto text-xs text-[var(--text-muted)]">⌘\</kbd>
                </CommandItem>
              </CommandGroup>
              <CommandSeparator className="my-2 -mx-2 h-px bg-[var(--border-light)]" />
              <CommandGroup heading="命令">
                <CommandItem className="relative flex cursor-pointer select-none items-center rounded-md px-3 py-2.5 gap-2 text-sm outline-none aria-selected:bg-[var(--bg-tertiary)] aria-selected:text-[var(--text-primary)] data-[disabled]:pointer-events-none data-[disabled]:opacity-50 transition-colors duration-100">
                  <CommandIcon className="mr-2 h-4 w-4" />
                  <span>运行分析</span>
                  <kbd className="ml-auto text-xs text-[var(--text-muted)]">⌘⇧A</kbd>
                </CommandItem>
                <CommandItem className="relative flex cursor-pointer select-none items-center rounded-md px-3 py-2.5 gap-2 text-sm outline-none aria-selected:bg-[var(--bg-tertiary)] aria-selected:text-[var(--text-primary)] data-[disabled]:pointer-events-none data-[disabled]:opacity-50 transition-colors duration-100">
                  <Code className="mr-2 h-4 w-4" />
                  <span>查找函数引用</span>
                  <kbd className="ml-auto text-xs text-[var(--text-muted)]">⌘⇧F</kbd>
                </CommandItem>
              </CommandGroup>
            </CommandList>
          </Command>
        </DialogContent>
      </DialogPortal>
    </Dialog>
  )
}

export { CommandMenu as Command }
