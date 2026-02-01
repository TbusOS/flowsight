"use client"

import * as React from "react"
import { Provider as JotaiProvider, useAtom } from "jotai"
import { listen } from "./lib/tauri-api"
import { MainLayout } from "./components/layout"
import { initThemeAtom } from "./lib/atoms/layout-atoms"
import { useAnalysisStore } from "./store/analysisStore"
import { ErrorBoundary } from "./components/ErrorBoundary/ErrorBoundary"

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
  // 使用 ref 来获取最新的 currentProject，避免闭包陈旧问题
  const currentProjectRef = React.useRef(useAnalysisStore.getState().currentProject)

  // 订阅 store 变化更新 ref
  React.useEffect(() => {
    const unsubscribe = useAnalysisStore.subscribe(
      (state) => {
        currentProjectRef.current = state.currentProject
      }
    )
    return unsubscribe
  }, [])

  React.useEffect(() => {
    // 监听索引进度事件
    const unlistenProgress = listen<{
      phase: string
      current: number
      total: number
      message: string
      files?: number
      functions?: number
      structs?: number
    }>("index-progress", (event) => {
      const { phase, current, total, message, files, functions, structs } = event.payload
      const currentProject = currentProjectRef.current
      
      // 更新索引进度
      setIndexProgress({
        phase: phase as 'scanning' | 'parsing' | 'indexing' | 'complete' | 'done' | 'error',
        current,
        total,
        message,
      })
      
      // 索引完成时更新项目信息（后端发送 phase="done"）
      if (phase === 'done' || phase === 'complete') {
        console.log('索引完成:', message, { files, functions, structs, currentPath: currentProject?.path })
        // 从事件中提取统计信息
        if (files !== undefined || functions !== undefined) {
          setCurrentProject({
            path: currentProject?.path || '',
            files_count: files ?? total ?? 0,
            functions_count: functions ?? 0,
            structs_count: structs ?? 0,
            indexed: true,
          })
        }
      }
    })

    return () => {
      unlistenProgress.then((fn) => fn())
    }
  }, [setIndexProgress, setCurrentProject])

  return null
}

// ============================================================================
// New FlowSight App - Minimal Design with shadcn/ui
// ============================================================================

export function App() {
  return (
    <ErrorBoundary>
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
    </ErrorBoundary>
  )
}

export default App
