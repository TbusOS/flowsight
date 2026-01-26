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
} from "lucide-react"
import {
  Dialog,
  DialogContent,
  DialogTitle,
  DialogPortal,
  DialogOverlay,
} from "./dialog"
import { cn } from "../../lib/utils"

interface CommandMenuProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function CommandMenu({ open, onOpenChange }: CommandMenuProps) {
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
            <CommandList className="max-h-[300px] overflow-y-auto overflow-x-hidden py-2">
              <CommandEmpty className="py-6 text-center text-sm text-[var(--text-muted)]">
                无结果
              </CommandEmpty>
              <CommandGroup heading="快速操作">
                <CommandItem className="relative flex cursor-pointer select-none items-center rounded-lg px-2 py-2 text-sm outline-none aria-selected:bg-[var(--bg-tertiary)] aria-selected:text-[var(--text-primary)] data-[disabled]:pointer-events-none data-[disabled]:opacity-50">
                  <Layout className="mr-2 h-4 w-4" />
                  <span>切换侧边栏</span>
                  <kbd className="ml-auto text-xs text-[var(--text-muted)]">⌘B</kbd>
                </CommandItem>
                <CommandItem className="relative flex cursor-pointer select-none items-center rounded-lg px-2 py-2 text-sm outline-none aria-selected:bg-[var(--bg-tertiary)] aria-selected:text-[var(--text-primary)] data-[disabled]:pointer-events-none data-[disabled]:opacity-50">
                  <Terminal className="mr-2 h-4 w-4" />
                  <span>切换终端</span>
                  <kbd className="ml-auto text-xs text-[var(--text-muted)]">⌘J</kbd>
                </CommandItem>
                <CommandItem className="relative flex cursor-pointer select-none items-center rounded-lg px-2 py-2 text-sm outline-none aria-selected:bg-[var(--bg-tertiary)] aria-selected:text-[var(--text-primary)] data-[disabled]:pointer-events-none data-[disabled]:opacity-50">
                  <Settings className="mr-2 h-4 w-4" />
                  <span>打开设置</span>
                  <kbd className="ml-auto text-xs text-[var(--text-muted)]">⌘,</kbd>
                </CommandItem>
              </CommandGroup>
              <CommandSeparator className="my-2 -mx-2 h-px bg-[var(--border-light)]" />
              <CommandGroup heading="视图">
                <CommandItem className="relative flex cursor-pointer select-none items-center rounded-lg px-2 py-2 text-sm outline-none aria-selected:bg-[var(--bg-tertiary)] aria-selected:text-[var(--text-primary)] data-[disabled]:pointer-events-none data-[disabled]:opacity-50">
                  <File className="mr-2 h-4 w-4" />
                  <span>代码视图</span>
                </CommandItem>
                <CommandItem className="relative flex cursor-pointer select-none items-center rounded-lg px-2 py-2 text-sm outline-none aria-selected:bg-[var(--bg-tertiary)] aria-selected:text-[var(--text-primary)] data-[disabled]:pointer-events-none data-[disabled]:opacity-50">
                  <Zap className="mr-2 h-4 w-4" />
                  <span>执行流视图</span>
                </CommandItem>
                <CommandItem className="relative flex cursor-pointer select-none items-center rounded-lg px-2 py-2 text-sm outline-none aria-selected:bg-[var(--bg-tertiary)] aria-selected:text-[var(--text-primary)] data-[disabled]:pointer-events-none data-[disabled]:opacity-50">
                  <BookOpen className="mr-2 h-4 w-4" />
                  <span>大纲视图</span>
                </CommandItem>
              </CommandGroup>
              <CommandSeparator className="my-2 -mx-2 h-px bg-[var(--border-light)]" />
              <CommandGroup heading="命令">
                <CommandItem className="relative flex cursor-pointer select-none items-center rounded-lg px-2 py-2 text-sm outline-none aria-selected:bg-[var(--bg-tertiary)] aria-selected:text-[var(--text-primary)] data-[disabled]:pointer-events-none data-[disabled]:opacity-50">
                  <CommandIcon className="mr-2 h-4 w-4" />
                  <span>运行分析</span>
                  <kbd className="ml-auto text-xs text-[var(--text-muted)]">⌘⇧A</kbd>
                </CommandItem>
                <CommandItem className="relative flex cursor-pointer select-none items-center rounded-lg px-2 py-2 text-sm outline-none aria-selected:bg-[var(--bg-tertiary)] aria-selected:text-[var(--text-primary)] data-[disabled]:pointer-events-none data-[disabled]:opacity-50">
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
