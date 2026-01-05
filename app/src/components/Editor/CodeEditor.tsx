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
  /** 跳转到指定行 (包含时间戳确保每次都触发) */
  goToLine?: { line: number; timestamp: number }
  /** 高亮行号列表 */
  highlightLines?: number[]
  /** 点击行回调 */
  onLineClick?: (line: number) => void
  /** 光标所在词变化回调（用于代码-图联动） */
  onWordAtCursor?: (word: string | null) => void
  /** 已知函数名列表（用于判断光标词是否为函数） */
  knownFunctions?: string[]
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
  onWordAtCursor,
  knownFunctions = [],
  readOnly = false,
}: CodeEditorProps) {
  const editorRef = useRef<editor.IStandaloneCodeEditor | null>(null)
  const decorationsRef = useRef<string[]>([])
  const lastWordRef = useRef<string | null>(null)
  const knownFunctionsSet = useRef<Set<string>>(new Set(knownFunctions))
  
  // 更新已知函数集合
  useEffect(() => {
    knownFunctionsSet.current = new Set(knownFunctions)
  }, [knownFunctions])

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
    
    // 监听光标位置变化（用于代码-图联动）
    if (onWordAtCursor) {
      editor.onDidChangeCursorPosition((e) => {
        const model = editor.getModel()
        if (!model) return
        
        // 获取光标所在位置的词
        const wordInfo = model.getWordAtPosition(e.position)
        const word = wordInfo?.word || null
        
        // 只有当词变化且是已知函数时才触发回调
        if (word !== lastWordRef.current) {
          lastWordRef.current = word
          
          // 检查是否为已知函数名
          if (word && knownFunctionsSet.current.has(word)) {
            onWordAtCursor(word)
          } else {
            onWordAtCursor(null)
          }
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
      editorRef.current.revealLineInCenter(goToLine.line)
      editorRef.current.setPosition({ lineNumber: goToLine.line, column: 1 })
      editorRef.current.focus()
    }
  }, [goToLine]) // 由于 goToLine 包含 timestamp，每次点击都会触发

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

