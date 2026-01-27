"use client"

import * as React from "react"
import { Provider as JotaiProvider, useAtom } from "jotai"
import { MainLayout } from "./components/layout"
import { cn } from "./lib/utils"
import { initThemeAtom } from "./lib/atoms/layout-atoms"

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
// New FlowSight App - Minimal Design with shadcn/ui
// ============================================================================

export function App() {
  return (
    <JotaiProvider>
      <ThemeInitializer />
      <MainLayout />
    </JotaiProvider>
  )
}

export default App
