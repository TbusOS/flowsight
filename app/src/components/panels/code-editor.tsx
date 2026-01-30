"use client"

import * as React from "react"
import Editor, { OnMount, loader } from "@monaco-editor/react"
import { invoke } from "@tauri-apps/api/core"
import { Loader2, FileCode, X } from "lucide-react"
import { cn } from "../../lib/utils"
import { useAnalysisStore } from "../../store/analysisStore"

// 配置 Monaco 使用本地资源
loader.config({
  paths: {
    vs: "https://cdn.jsdelivr.net/npm/monaco-editor@0.55.1/min/vs"
  }
})

interface CodeEditorProps {
  className?: string
  filePath?: string | null
  onClose?: () => void
}

// 获取语言模式
function getLanguageFromPath(path: string): string {
  const ext = path.split(".").pop()?.toLowerCase()
  switch (ext) {
    case "c":
    case "h":
      return "c"
    case "cpp":
    case "hpp":
    case "cc":
    case "cxx":
      return "cpp"
    case "rs":
      return "rust"
    case "ts":
    case "tsx":
      return "typescript"
    case "js":
    case "jsx":
      return "javascript"
    case "json":
      return "json"
    case "yaml":
    case "yml":
      return "yaml"
    case "md":
      return "markdown"
    case "py":
      return "python"
    default:
      return "plaintext"
  }
}

export function CodeEditor({ className, filePath, onClose }: CodeEditorProps) {
  const [content, setContent] = React.useState<string>("")
  const [loading, setLoading] = React.useState(false)
  const [error, setError] = React.useState<string | null>(null)
  const editorRef = React.useRef<any>(null)

  // 加载文件内容
  React.useEffect(() => {
    if (!filePath) {
      setContent("")
      return
    }

    const loadFile = async () => {
      setLoading(true)
      setError(null)
      try {
        const result = await invoke<string>("read_file", { path: filePath })
        setContent(result)
      } catch (err) {
        console.error("读取文件失败:", err)
        setError(String(err))
      } finally {
        setLoading(false)
      }
    }

    loadFile()
  }, [filePath])

  // 编辑器挂载
  const handleEditorMount: OnMount = (editor, monaco) => {
    editorRef.current = editor

    // 配置 C 语言语法高亮增强
    monaco.languages.setLanguageConfiguration("c", {
      comments: {
        lineComment: "//",
        blockComment: ["/*", "*/"],
      },
      brackets: [
        ["{", "}"],
        ["[", "]"],
        ["(", ")"],
      ],
      autoClosingPairs: [
        { open: "{", close: "}" },
        { open: "[", close: "]" },
        { open: "(", close: ")" },
        { open: '"', close: '"' },
        { open: "'", close: "'" },
      ],
    })
  }

  // 没有文件时显示空状态
  if (!filePath) {
    return (
      <div className={cn("flex h-full w-full items-center justify-center bg-[var(--bg-primary)]", className)}>
        <div className="text-center">
          <div className="mb-4 inline-flex h-12 w-12 items-center justify-center rounded-xl bg-[var(--accent)]/10">
            <FileCode className="h-6 w-6 text-[var(--accent)]" />
          </div>
          <h3 className="mb-2 text-sm font-medium text-[var(--text-primary)]">代码编辑器</h3>
          <p className="text-xs text-[var(--text-muted)]">选择一个文件开始编辑</p>
        </div>
      </div>
    )
  }

  // 加载中
  if (loading) {
    return (
      <div className={cn("flex h-full w-full items-center justify-center bg-[var(--bg-primary)]", className)}>
        <div className="flex items-center gap-2 text-[var(--text-muted)]">
          <Loader2 className="h-4 w-4 animate-spin" />
          <span className="text-sm">加载中...</span>
        </div>
      </div>
    )
  }

  // 错误状态
  if (error) {
    return (
      <div className={cn("flex h-full w-full items-center justify-center bg-[var(--bg-primary)]", className)}>
        <div className="text-center">
          <p className="text-sm text-red-400">加载失败: {error}</p>
        </div>
      </div>
    )
  }

  const fileName = filePath.split("/").pop() || filePath
  const language = getLanguageFromPath(filePath)

  return (
    <div className={cn("flex h-full w-full flex-col bg-[var(--bg-primary)]", className)}>
      {/* 文件标签栏 */}
      <div className="flex h-9 items-center border-b border-[var(--border-subtle)] bg-[var(--bg-secondary)] px-2">
        <div className="flex items-center gap-2 rounded px-2 py-1 bg-[var(--bg-tertiary)]">
          <FileCode className="h-3.5 w-3.5 text-[var(--accent)]" />
          <span className="text-xs text-[var(--text-primary)]">{fileName}</span>
          {onClose && (
            <button
              onClick={onClose}
              className="ml-1 rounded p-0.5 hover:bg-[var(--bg-hover)] text-[var(--text-muted)] hover:text-[var(--text-primary)]"
            >
              <X className="h-3 w-3" />
            </button>
          )}
        </div>
      </div>

      {/* Monaco 编辑器 */}
      <div className="flex-1 overflow-hidden">
        <Editor
          height="100%"
          language={language}
          value={content}
          theme="vs-dark"
          onMount={handleEditorMount}
          options={{
            readOnly: true, // 目前只读
            fontSize: 13,
            fontFamily: "'JetBrains Mono', 'Fira Code', 'SF Mono', Menlo, Monaco, 'Courier New', monospace",
            fontLigatures: true,
            minimap: {
              enabled: true,
              maxColumn: 80,
            },
            scrollBeyondLastLine: false,
            wordWrap: "off",
            lineNumbers: "on",
            renderLineHighlight: "line",
            cursorBlinking: "smooth",
            smoothScrolling: true,
            padding: { top: 8, bottom: 8 },
            automaticLayout: true,
            // 语法高亮增强
            bracketPairColorization: { enabled: true },
            guides: {
              bracketPairs: true,
              indentation: true,
            },
            "semanticHighlighting.enabled": true,
          }}
        />
      </div>
    </div>
  )
}
