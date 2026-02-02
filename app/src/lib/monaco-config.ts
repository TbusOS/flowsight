/**
 * Monaco Editor 配置模块
 * 
 * 提供统一的编辑器配置、主题和性能优化
 */

import type { editor } from 'monaco-editor'
import { loader } from '@monaco-editor/react'

// 配置 Monaco 使用 CDN 资源（性能优化：预加载）
loader.config({
  paths: {
    vs: 'https://cdn.jsdelivr.net/npm/monaco-editor@0.55.1/min/vs'
  }
})

// 预加载 Monaco Editor（提前初始化 web worker）
let monacoPreloaded = false
export function preloadMonaco() {
  if (monacoPreloaded) return
  monacoPreloaded = true
  
  // 预加载 Monaco 核心
  loader.init().then(monaco => {
    // 预注册主题
    monaco.editor.defineTheme('flowsight-dark', flowsightDarkTheme)
    monaco.editor.defineTheme('flowsight-light', flowsightLightTheme)
    
    // 预配置 C 语言
    monaco.languages.setLanguageConfiguration('c', cLanguageConfig)
    
    console.log('[Monaco] Preloaded successfully')
  }).catch(err => {
    console.warn('[Monaco] Preload failed:', err)
  })
}

// FlowSight 深色主题
export const flowsightDarkTheme: editor.IStandaloneThemeData = {
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
    { token: 'delimiter', foreground: 'e6edf3' },
    { token: 'identifier', foreground: 'e6edf3' },
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
    'editorBracketMatch.background': '#58a6ff33',
    'editorBracketMatch.border': '#58a6ff',
  },
}

// FlowSight 浅色主题
export const flowsightLightTheme: editor.IStandaloneThemeData = {
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

// C 语言配置
export const cLanguageConfig: import('monaco-editor').languages.LanguageConfiguration = {
  comments: {
    lineComment: '//',
    blockComment: ['/*', '*/'],
  },
  brackets: [
    ['{', '}'],
    ['[', ']'],
    ['(', ')'],
  ],
  autoClosingPairs: [
    { open: '{', close: '}' },
    { open: '[', close: ']' },
    { open: '(', close: ')' },
    { open: '"', close: '"' },
    { open: "'", close: "'" },
  ],
  surroundingPairs: [
    { open: '{', close: '}' },
    { open: '[', close: ']' },
    { open: '(', close: ')' },
    { open: '"', close: '"' },
    { open: "'", close: "'" },
  ],
  folding: {
    markers: {
      start: /^\s*#pragma\s+region\b/,
      end: /^\s*#pragma\s+endregion\b/,
    },
  },
}

// 编辑器默认选项（性能优化）
export const defaultEditorOptions: editor.IStandaloneEditorConstructionOptions = {
  fontSize: 13,
  fontFamily: "'JetBrains Mono', 'Fira Code', 'SF Mono', Menlo, Monaco, 'Courier New', monospace",
  fontLigatures: true,
  lineNumbers: 'on',
  scrollBeyondLastLine: false,
  wordWrap: 'off',
  tabSize: 4,
  insertSpaces: true,
  automaticLayout: true,
  renderLineHighlight: 'line',
  cursorBlinking: 'smooth',
  cursorSmoothCaretAnimation: 'on',
  smoothScrolling: true,
  padding: { top: 8, bottom: 8 },
  
  // 性能优化选项
  minimap: {
    enabled: true,
    maxColumn: 80,
    renderCharacters: false, // 不渲染字符，只显示颜色块
    scale: 1,
  },
  
  // 语法高亮
  bracketPairColorization: { enabled: true },
  guides: {
    bracketPairs: true,
    indentation: true,
  },
  
  // 代码折叠
  folding: true,
  foldingStrategy: 'auto',
  foldingHighlight: true,
  showFoldingControls: 'mouseover', // 鼠标悬停时显示
  
  // 选中高亮
  occurrencesHighlight: 'singleFile',
  selectionHighlight: true,
  matchBrackets: 'always',
  
  // 渲染优化
  renderWhitespace: 'none',
  renderControlCharacters: false,
  renderValidationDecorations: 'on',
  
  // 滚动优化
  fastScrollSensitivity: 5,
  mouseWheelScrollSensitivity: 1,
  
  // 搜索功能
  find: {
    addExtraSpaceOnTop: true,
    autoFindInSelection: 'multiline',
    seedSearchStringFromSelection: 'selection',
  },
}

// 大文件优化选项（> 100KB）
export const largeFileEditorOptions: Partial<editor.IStandaloneEditorConstructionOptions> = {
  minimap: { enabled: false },
  folding: false,
  bracketPairColorization: { enabled: false },
  guides: { bracketPairs: false, indentation: false },
  occurrencesHighlight: 'off',
  selectionHighlight: false,
  renderWhitespace: 'none',
  wordBasedSuggestions: 'off',
  quickSuggestions: false,
}

// 根据文件扩展名获取语言
export function getLanguageFromPath(path: string): string {
  const ext = path.split('.').pop()?.toLowerCase()
  const languageMap: Record<string, string> = {
    c: 'c',
    h: 'c',
    cpp: 'cpp',
    cc: 'cpp',
    cxx: 'cpp',
    hpp: 'cpp',
    rs: 'rust',
    py: 'python',
    js: 'javascript',
    jsx: 'javascript',
    ts: 'typescript',
    tsx: 'typescript',
    json: 'json',
    yaml: 'yaml',
    yml: 'yaml',
    md: 'markdown',
    sh: 'shell',
    bash: 'shell',
    makefile: 'makefile',
    dockerfile: 'dockerfile',
  }
  return languageMap[ext || ''] || 'plaintext'
}

// 判断是否为大文件
export function isLargeFile(content: string): boolean {
  return content.length > 100 * 1024 // > 100KB
}

// 获取编辑器选项（根据文件大小自动优化）
export function getEditorOptions(
  content: string,
  overrides?: Partial<editor.IStandaloneEditorConstructionOptions>
): editor.IStandaloneEditorConstructionOptions {
  const isLarge = isLargeFile(content)
  
  return {
    ...defaultEditorOptions,
    ...(isLarge ? largeFileEditorOptions : {}),
    ...overrides,
  }
}
