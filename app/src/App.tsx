"use client"

import * as React from "react"
import { Provider as JotaiProvider } from "jotai"
import { MainLayout } from "./components/layout"
import { cn } from "./lib/utils"

// ============================================================================
// New FlowSight App - Minimal Design with shadcn/ui
// ============================================================================

export function App() {
  return (
    <JotaiProvider>
      <MainLayout />
    </JotaiProvider>
  )
}

export default App
