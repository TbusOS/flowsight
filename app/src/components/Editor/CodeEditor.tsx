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
  /** 主题 */
  theme?: 'dark' | 'light'
  /** 字体大小 */
  fontSize?: number
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
const flowsightDarkTheme: editor.IStandaloneThemeData = {
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
    'editor.wordHighlightBackground': '#58a6ff33',
    'editor.wordHighlightStrongBackground': '#58a6ff55',
  },
}

// FlowSight 浅色主题
const flowsightLightTheme: editor.IStandaloneThemeData = {
  base: 'vs',
  inherit: true,
  rules: [
    { token: 'comment', foreground: '6a737d', fontStyle: 'italic' },
    { token: 'keyword', foreground: 'd73a49' },
    { token: 'type', foreground: '005cc5' },
    { token: 'function', foreground: '6f42c1' },
    { token: 'string', foreground: '032f62' },
    { token: 'number', foreground: '005cc5' },
    { token: 'variable', foreground: '24292e' },
    { token: 'operator', foreground: 'd73a49' },
  ],
  colors: {
    'editor.background': '#ffffff',
    'editor.foreground': '#24292e',
    'editor.lineHighlightBackground': '#f6f8fa',
    'editor.selectionBackground': '#0366d625',
    'editorCursor.foreground': '#0366d6',
    'editorLineNumber.foreground': '#959da5',
    'editorLineNumber.activeForeground': '#24292e',
    'editor.findMatchBackground': '#ffdf5d66',
    'editor.findMatchHighlightBackground': '#ffdf5d33',
    'editor.wordHighlightBackground': '#0366d622',
    'editor.wordHighlightStrongBackground': '#0366d644',
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
  theme = 'dark',
  fontSize = 14,
}: CodeEditorProps) {
  const editorRef = useRef<editor.IStandaloneCodeEditor | null>(null)
  const decorationsRef = useRef<string[]>([])
  const lastWordRef = useRef<string | null>(null)
  const knownFunctionsSet = useRef<Set<string>>(new Set(knownFunctions))
  
  // 更新已知函数集合
  useEffect(() => {
    knownFunctionsSet.current = new Set(knownFunctions)
  }, [knownFunctions])
  
  // 主题变化时更新
  useEffect(() => {
    if (editorRef.current) {
      const monaco = (window as any).monaco
      if (monaco) {
        monaco.editor.setTheme(theme === 'dark' ? 'flowsight-dark' : 'flowsight-light')
      }
    }
  }, [theme])

  const handleEditorMount: OnMount = (editor, monaco) => {
    editorRef.current = editor

    // 注册主题
    monaco.editor.defineTheme('flowsight-dark', flowsightDarkTheme)
    monaco.editor.defineTheme('flowsight-light', flowsightLightTheme)
    monaco.editor.setTheme(theme === 'dark' ? 'flowsight-dark' : 'flowsight-light')

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
        theme={theme === 'dark' ? 'flowsight-dark' : 'flowsight-light'}
        onMount={handleEditorMount}
        onChange={handleChange}
        options={{
          readOnly,
          fontSize,
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
          // 选中单词高亮
          occurrencesHighlight: 'singleFile',
          selectionHighlight: true,
          // 括号匹配高亮
          matchBrackets: 'always',
          bracketPairColorization: { enabled: true },
          // 缩进参考线
          guides: {
            indentation: true,
            bracketPairs: true,
          },
        }}
      />
    </div>
  )
}

export default CodeEditor

