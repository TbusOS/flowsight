"use client"

import * as React from "react"
import Editor, { OnMount } from "@monaco-editor/react"
import { invoke } from "../../lib/tauri-api"
import { Loader2, FileCode, X, Save, Check, AlertCircle } from "lucide-react"
import { cn } from "../../lib/utils"
import { useAnalysisStore } from "../../store/analysisStore"
import { useAtom, useSetAtom } from "jotai"
import { jumpTargetAtom, cursorPositionAtom } from "../../lib/atoms/layout-atoms"
import { 
  getLanguageFromPath, 
  getEditorOptions, 
  cLanguageConfig,
  flowsightDarkTheme,
} from "../../lib/monaco-config"

interface CodeEditorProps {
  className?: string
  filePath?: string | null
  onClose?: () => void
  readOnly?: boolean
}

type SaveStatus = "saved" | "modified" | "saving" | "error"

// 使用 React.memo 优化重渲染
export const CodeEditor = React.memo(function CodeEditor({ 
  className, 
  filePath, 
  onClose, 
  readOnly = false 
}: CodeEditorProps) {
  const [content, setContent] = React.useState<string>("")
  const [originalContent, setOriginalContent] = React.useState<string>("")
  const [loading, setLoading] = React.useState(false)
  const [error, setError] = React.useState<string | null>(null)
  const [saveStatus, setSaveStatus] = React.useState<SaveStatus>("saved")
  const [saveError, setSaveError] = React.useState<string | null>(null)
  const editorRef = React.useRef<any>(null)
  const monacoRef = React.useRef<any>(null)
  const [jumpTarget, setJumpTarget] = useAtom(jumpTargetAtom)
  const setCursorPosition = useSetAtom(cursorPositionAtom)

  // 加载文件内容
  React.useEffect(() => {
    if (!filePath) {
      setContent("")
      setOriginalContent("")
      setSaveStatus("saved")
      return
    }

    const loadFile = async () => {
      setLoading(true)
      setError(null)
      setSaveError(null)
      try {
        const result = await invoke<string>("read_file", { path: filePath })
        setContent(result)
        setOriginalContent(result)
        setSaveStatus("saved")
      } catch (err) {
        console.error("读取文件失败:", err)
        setError(String(err))
      } finally {
        setLoading(false)
      }
    }

    loadFile()
  }, [filePath])

  // 监听跳转目标，执行行号跳转
  React.useEffect(() => {
    if (jumpTarget && jumpTarget.filePath === filePath && editorRef.current) {
      const editor = editorRef.current
      const line = jumpTarget.line
      const column = jumpTarget.column || 1
      
      // 跳转到指定行
      editor.revealLineInCenter(line)
      editor.setPosition({ lineNumber: line, column })
      editor.focus()
      
      // 高亮当前行
      if (monacoRef.current) {
        const decorations = editor.deltaDecorations([], [
          {
            range: new monacoRef.current.Range(line, 1, line, 1),
            options: {
              isWholeLine: true,
              className: 'highlight-line',
              glyphMarginClassName: 'highlight-glyph'
            }
          }
        ])
        // 2秒后移除高亮
        setTimeout(() => {
          editor.deltaDecorations(decorations, [])
        }, 2000)
      }
      
      console.log('[CodeEditor] Jumped to line:', line)
      
      // 清除跳转目标
      setJumpTarget(null)
    }
  }, [jumpTarget, filePath, setJumpTarget])

  // 保存文件
  const saveFile = React.useCallback(async () => {
    if (!filePath || readOnly || saveStatus === "saving") return

    setSaveStatus("saving")
    setSaveError(null)
    try {
      await invoke("write_file", { path: filePath, content })
      setOriginalContent(content)
      setSaveStatus("saved")
    } catch (err) {
      console.error("保存文件失败:", err)
      setSaveError(String(err))
      setSaveStatus("error")
      // 3秒后恢复为 modified 状态
      setTimeout(() => {
        if (content !== originalContent) {
          setSaveStatus("modified")
        }
      }, 3000)
    }
  }, [filePath, content, originalContent, readOnly, saveStatus])

  // 监听内容变化，更新保存状态
  const handleContentChange = React.useCallback((value: string | undefined) => {
    const newContent = value || ""
    setContent(newContent)
    if (newContent !== originalContent) {
      setSaveStatus("modified")
    } else {
      setSaveStatus("saved")
    }
  }, [originalContent])

  // 键盘快捷键
  React.useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Cmd/Ctrl + S 保存
      if ((e.metaKey || e.ctrlKey) && e.key === "s") {
        e.preventDefault()
        saveFile()
      }
    }

    window.addEventListener("keydown", handleKeyDown)
    return () => window.removeEventListener("keydown", handleKeyDown)
  }, [saveFile])

  // 编辑器挂载
  const handleEditorMount: OnMount = (editor, monaco) => {
    editorRef.current = editor
    monacoRef.current = monaco

    // 添加保存快捷键到编辑器
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {
      saveFile()
    })

    // 添加搜索快捷键
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyF, () => {
      editor.getAction("actions.find")?.run()
    })

    // 配置 C 语言语法高亮增强
    monaco.languages.setLanguageConfiguration("c", cLanguageConfig)
    
    // 注册 FlowSight 主题
    monaco.editor.defineTheme("flowsight-dark", flowsightDarkTheme)
    
    // 监听光标位置变化 - 更新状态栏
    editor.onDidChangeCursorPosition((e: any) => {
      const position = e.position
      const selection = editor.getSelection()
      
      if (selection && !selection.isEmpty()) {
        setCursorPosition({
          line: position.lineNumber,
          column: position.column,
          selection: {
            startLine: selection.startLineNumber,
            startColumn: selection.startColumn,
            endLine: selection.endLineNumber,
            endColumn: selection.endColumn,
          }
        })
      } else {
        setCursorPosition({
          line: position.lineNumber,
          column: position.column,
        })
      }
    })
    
    // 监听选区变化
    editor.onDidChangeCursorSelection((e: any) => {
      const selection = e.selection
      const position = editor.getPosition()
      
      if (selection && !selection.isEmpty()) {
        setCursorPosition({
          line: position?.lineNumber || 1,
          column: position?.column || 1,
          selection: {
            startLine: selection.startLineNumber,
            startColumn: selection.startColumn,
            endLine: selection.endLineNumber,
            endColumn: selection.endColumn,
          }
        })
      }
    })
  }

  // 缓存编辑器选项（性能优化：根据文件大小自动调整）
  const editorOptions = React.useMemo(() => 
    getEditorOptions(content, { readOnly }),
    [content, readOnly]
  )

  // 获取保存状态图标和文本
  const getSaveStatusDisplay = () => {
    switch (saveStatus) {
      case "saved":
        return { icon: <Check className="h-3 w-3" />, text: "已保存", color: "text-green-500" }
      case "modified":
        return { icon: <Save className="h-3 w-3" />, text: "未保存", color: "text-yellow-500" }
      case "saving":
        return { icon: <Loader2 className="h-3 w-3 animate-spin" />, text: "保存中...", color: "text-blue-500" }
      case "error":
        return { icon: <AlertCircle className="h-3 w-3" />, text: "保存失败", color: "text-red-500" }
    }
  }

  const statusDisplay = getSaveStatusDisplay()

  // 没有文件时显示空状态
  if (!filePath) {
    return (
      <div 
        data-testid="code-editor"
        className={cn("flex h-full w-full items-center justify-center bg-[var(--bg-primary)]", className)}
      >
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
      <div 
        data-testid="code-editor"
        className={cn("flex h-full w-full items-center justify-center bg-[var(--bg-primary)]", className)}
      >
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
      <div 
        data-testid="code-editor"
        className={cn("flex h-full w-full items-center justify-center bg-[var(--bg-primary)]", className)}
      >
        <div className="text-center">
          <p className="text-sm text-red-400">加载失败: {error}</p>
        </div>
      </div>
    )
  }

  const fileName = filePath.split("/").pop() || filePath
  const language = getLanguageFromPath(filePath)

  return (
    <div 
      data-testid="code-editor"
      className={cn("flex h-full w-full flex-col bg-[var(--bg-primary)]", className)}
    >
      {/* 文件标签栏 */}
      <div className="flex h-9 items-center justify-between border-b border-[var(--border-subtle)] bg-[var(--bg-secondary)] px-2">
        <div className="flex items-center gap-2 rounded px-2 py-1 bg-[var(--bg-tertiary)]">
          <FileCode className="h-3.5 w-3.5 text-[var(--accent)]" />
          <span 
            data-testid="editor-filename"
            className="text-xs text-[var(--text-primary)]"
            title={filePath}
          >
            {fileName}
          </span>
          {onClose && (
            <button
              onClick={onClose}
              className="ml-1 rounded p-0.5 hover:bg-[var(--bg-hover)] text-[var(--text-muted)] hover:text-[var(--text-primary)]"
              aria-label="关闭文件"
            >
              <X className="h-3 w-3" />
            </button>
          )}
        </div>

        {/* 保存状态指示器 */}
        <div className="flex items-center gap-2">
          {!readOnly && (
            <>
              <div 
                data-testid="editor-save-status"
                data-status={saveStatus}
                className={cn("flex items-center gap-1 text-xs", statusDisplay.color)}
                title={saveError || statusDisplay.text}
              >
                {statusDisplay.icon}
                <span>{statusDisplay.text}</span>
              </div>
              {saveStatus === "modified" && (
                <button
                  onClick={saveFile}
                  className="flex items-center gap-1 rounded px-2 py-0.5 text-xs bg-[var(--accent)] text-white hover:bg-[var(--accent-hover)] transition-colors"
                  title="保存文件 (⌘S)"
                >
                  <Save className="h-3 w-3" />
                  保存
                </button>
              )}
            </>
          )}
          {readOnly && (
            <span 
              data-testid="editor-save-status"
              data-status="readonly"
              className="text-xs text-[var(--text-muted)]"
            >
              只读
            </span>
          )}
        </div>
      </div>

      {/* Monaco 编辑器 */}
      <div className="flex-1 overflow-hidden">
        <Editor
          height="100%"
          language={language}
          value={content}
          onChange={handleContentChange}
          theme="flowsight-dark"
          onMount={handleEditorMount}
          options={editorOptions}
        />
      </div>
    </div>
  )
})
