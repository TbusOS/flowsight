/**
 * Monaco Editor 代码编辑器组件
 * 
 * 提供语法高亮、代码导航等功能
 */

import { useRef, useEffect } from 'react'
import Editor, { OnMount, OnChange } from '@monaco-editor/react'
import type { editor } from 'monaco-editor'

interface CodeEditorProps {
  /** 文件内容 */
  content: string
  /** 文件路径（用于语言检测） */
  filePath?: string
  /** 内容变更回调 */
  onChange?: (value: string) => void
  /** 跳转到指定行 */
  goToLine?: number
  /** 高亮行号列表 */
  highlightLines?: number[]
  /** 点击行回调 */
  onLineClick?: (line: number) => void
  /** 是否只读 */
  readOnly?: boolean
}

// 根据文件扩展名获取语言
function getLanguage(filePath: string): string {
  const ext = filePath.split('.').pop()?.toLowerCase()
  switch (ext) {
    case 'c':
    case 'h':
      return 'c'
    case 'cpp':
    case 'cc':
    case 'cxx':
    case 'hpp':
      return 'cpp'
    case 'rs':
      return 'rust'
    case 'py':
      return 'python'
    case 'js':
      return 'javascript'
    case 'ts':
      return 'typescript'
    case 'tsx':
      return 'typescript'
    case 'json':
      return 'json'
    case 'yaml':
    case 'yml':
      return 'yaml'
    case 'md':
      return 'markdown'
    default:
      return 'plaintext'
  }
}

// FlowSight 深色主题
const flowsightTheme: editor.IStandaloneThemeData = {
  base: 'vs-dark',
  inherit: true,
  rules: [
    { token: 'comment', foreground: '6e7681', fontStyle: 'italic' },
    { token: 'keyword', foreground: 'ff79c6' },
    { token: 'type', foreground: '8be9fd' },
    { token: 'function', foreground: '50fa7b' },
    { token: 'string', foreground: 'f1fa8c' },
    { token: 'number', foreground: 'bd93f9' },
    { token: 'variable', foreground: 'e6edf3' },
    { token: 'operator', foreground: 'ff79c6' },
  ],
  colors: {
    'editor.background': '#0f1419',
    'editor.foreground': '#e6edf3',
    'editor.lineHighlightBackground': '#1a1f2e',
    'editor.selectionBackground': '#264f78',
    'editorCursor.foreground': '#58a6ff',
    'editorLineNumber.foreground': '#6e7681',
    'editorLineNumber.activeForeground': '#e6edf3',
    'editor.findMatchBackground': '#ffc83d44',
    'editor.findMatchHighlightBackground': '#ffc83d22',
  },
}

export function CodeEditor({
  content,
  filePath = 'untitled.c',
  onChange,
  goToLine,
  highlightLines = [],
  onLineClick,
  readOnly = false,
}: CodeEditorProps) {
  const editorRef = useRef<editor.IStandaloneCodeEditor | null>(null)
  const decorationsRef = useRef<string[]>([])

  const handleEditorMount: OnMount = (editor, monaco) => {
    editorRef.current = editor

    // 注册主题
    monaco.editor.defineTheme('flowsight', flowsightTheme)
    monaco.editor.setTheme('flowsight')

    // 监听行点击
    if (onLineClick) {
      editor.onMouseDown((e) => {
        if (e.target.position) {
          onLineClick(e.target.position.lineNumber)
        }
      })
    }
  }

  const handleChange: OnChange = (value) => {
    if (onChange && value !== undefined) {
      onChange(value)
    }
  }

  // 跳转到指定行
  useEffect(() => {
    if (editorRef.current && goToLine) {
      editorRef.current.revealLineInCenter(goToLine)
      editorRef.current.setPosition({ lineNumber: goToLine, column: 1 })
      editorRef.current.focus()
    }
  }, [goToLine])

  // 高亮行
  useEffect(() => {
    if (editorRef.current && highlightLines.length > 0) {
      const decorations: editor.IModelDeltaDecoration[] = highlightLines.map(line => ({
        range: {
          startLineNumber: line,
          startColumn: 1,
          endLineNumber: line,
          endColumn: 1,
        },
        options: {
          isWholeLine: true,
          className: 'highlight-line',
          glyphMarginClassName: 'highlight-glyph',
        },
      }))

      decorationsRef.current = editorRef.current.deltaDecorations(
        decorationsRef.current,
        decorations
      )
    }
  }, [highlightLines])

  return (
    <div className="code-editor">
      <Editor
        height="100%"
        language={getLanguage(filePath)}
        value={content}
        theme="flowsight"
        onMount={handleEditorMount}
        onChange={handleChange}
        options={{
          readOnly,
          fontSize: 14,
          fontFamily: "'JetBrains Mono', 'Fira Code', 'Consolas', monospace",
          lineNumbers: 'on',
          minimap: { enabled: true },
          scrollBeyondLastLine: false,
          wordWrap: 'off',
          tabSize: 4,
          insertSpaces: true,
          automaticLayout: true,
          folding: true,
          glyphMargin: true,
          renderLineHighlight: 'all',
          cursorBlinking: 'smooth',
          cursorSmoothCaretAnimation: 'on',
          smoothScrolling: true,
          padding: { top: 8 },
        }}
      />
    </div>
  )
}

export default CodeEditor

