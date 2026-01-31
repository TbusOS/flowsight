"use client"

import * as React from "react"
import { Provider as JotaiProvider, useAtom } from "jotai"
import { listen } from "./lib/tauri-api"
import { MainLayout } from "./components/layout"
import { cn } from "./lib/utils"
import { initThemeAtom } from "./lib/atoms/layout-atoms"
import { useAnalysisStore } from "./store/analysisStore"

// ============================================================================
// Theme Initializer Component
// ============================================================================

function ThemeInitializer() {
  const [, initTheme] = useAtom(initThemeAtom)

  React.useEffect(() => {
    initTheme()
  }, [initTheme])

  return null
}

// ============================================================================
// Tauri Event Listener Component
// ============================================================================

function TauriEventListener() {
  const setIndexProgress = useAnalysisStore((state) => state.setIndexProgress)
  const setCurrentProject = useAnalysisStore((state) => state.setCurrentProject)

  React.useEffect(() => {
    // 监听索引进度事件
    const unlistenProgress = listen<{
      phase: string
      current: number
      total: number
      message: string
    }>("index-progress", (event) => {
      const { phase, current, total, message } = event.payload
      setIndexProgress({
        phase: phase as 'scanning' | 'parsing' | 'indexing' | 'complete' | 'error',
        current,
        total,
        message,
      })
      
      // 索引完成时更新项目信息
      if (phase === 'complete') {
        console.log('索引完成:', message)
      }
    })

    // 监听索引完成事件
    const unlistenComplete = listen<{
      path: string
      files_count: number
      functions_count: number
      structs_count: number
    }>("index-complete", (event) => {
      setCurrentProject({
        ...event.payload,
        indexed: true,
      })
      setIndexProgress(null)
    })

    return () => {
      unlistenProgress.then((fn) => fn())
      unlistenComplete.then((fn) => fn())
    }
  }, [setIndexProgress, setCurrentProject])

  return null
}

// ============================================================================
// New FlowSight App - Minimal Design with shadcn/ui
// ============================================================================

export function App() {
  return (
    <JotaiProvider>
      <ThemeInitializer />
      <TauriEventListener />
      {/* Skip Link for keyboard users */}
      <a
        href="#main-content"
        className="sr-only focus:not-sr-only focus:absolute focus:top-4 focus:left-4 focus:z-[100] focus:px-4 focus:py-2 focus:bg-[var(--accent)] focus:text-white focus:font-medium rounded-md"
      >
        跳转到主内容
      </a>
      <MainLayout />
    </JotaiProvider>
  )
}

export default App
