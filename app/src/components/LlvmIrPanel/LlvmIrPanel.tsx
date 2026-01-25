/**
 * LlvmIrPanel - LLVM IR 可视化面板
 *
 * 功能特性：
 * - 语法高亮显示 LLVM IR 代码
 * - 基本块展开/折叠
 * - 关键指令标注（call, br, store, load, ret）
 * - 支持复制和搜索功能
 * - 多函数对比模式
 * - 指令跳转和高亮
 */

import { useState, useMemo, useCallback, useRef, useEffect } from 'react'
import './LlvmIrPanel.css'

// ============================================
// 类型定义
// ============================================

/** LLVM IR 基本块 */
export interface LlvmBasicBlock {
  /** 块名称 */
  name: string
  /** 指令列表 */
  instructions: LlvmInstruction[]
  /** 前驱块列表 */
  predecessors: string[]
  /** 后继块列表 */
  successors: string[]
  /** 终止符指令 */
  terminator?: LlvmInstruction
}

/** LLVM IR 指令 */
export interface LlvmInstruction {
  /** 操作码 (add, call, ret 等) */
  opcode: string
  /** 目标寄存器 (如果有) */
  dest?: string
  /** 结果类型 */
  typeStr: string
  /** 操作数列表 */
  operands: string[]
  /** 源代码位置 */
  location?: {
    file: string
    line: number
  }
}

/** LLVM IR 函数 */
export interface LlvmFunction {
  /** 函数名称 */
  name: string
  /** 返回类型 */
  returnType: string
  /** 参数列表 */
  parameters: LlvmParameter[]
  /** 基本块列表 */
  blocks: LlvmBasicBlock[]
  /** 是否为回调函数 */
  isCallback: boolean
  /** 回调上下文 */
  callbackContext?: string
}

/** LLVM IR 参数 */
export interface LlvmParameter {
  /** 参数名称 */
  name: string
  /** 参数类型字符串 */
  typeStr: string
}

/** LLVM IR 解析结果 */
export interface LlvmIrParseResult {
  /** 模块名称 */
  moduleName: string
  /** 函数映射 */
  functions: Record<string, LlvmFunction>
}

/** LlvmIrPanel 组件属性 */
export interface LlvmIrPanelProps {
  /** 面板标题 */
  title?: string
  /** LLVM IR 解析结果 */
  parseResult?: LlvmIrParseResult | null
  /** 当前选中的函数名 */
  selectedFunction?: string | null
  /** 是否显示加载状态 */
  loading?: boolean
  /** 错误信息 */
  error?: string | null
  /** 主题模式 */
  theme?: 'dark' | 'light'
  /** 是否启用对比模式 */
  comparisonMode?: boolean
  /** 对比模式下右侧函数名 */
  compareFunction?: string | null
  /** 高亮的指令位置 */
  highlightedInstruction?: {
    functionName: string
    blockName: string
    instructionIndex: number
  } | null
  /** 函数选择变化回调 */
  onFunctionSelect?: (funcName: string | null) => void
  /** 对比函数选择变化回调 */
  onCompareSelect?: (funcName: string | null) => void
  /** 切换对比模式回调 */
  onToggleComparison?: (enabled: boolean) => void
  /** 指令点击回调 */
  onInstructionClick?: (instruction: LlvmInstruction, blockName: string) => void
  /** 块点击回调 */
  onBlockClick?: (blockName: string) => void
}

// ============================================
// 关键指令配置
// ============================================

const KEY_INSTRUCTIONS = {
  call: { label: 'CALL', color: '#22c55e', bgColor: '#22c55e22' },
  br: { label: 'BR', color: '#f59e0b', bgColor: '#f59e0b22' },
  ret: { label: 'RET', color: '#3b82f6', bgColor: '#3b82f622' },
  store: { label: 'STORE', color: '#a855f7', bgColor: '#a855f722' },
  load: { label: 'LOAD', color: '#ec4899', bgColor: '#ec489922' },
  switch: { label: 'SWITCH', color: '#ef4444', bgColor: '#ef444422' },
  invoke: { label: 'INVOKE', color: '#14b8a6', bgColor: '#14b8a622' },
  alloca: { label: 'ALLOC', color: '#f97316', bgColor: '#f9731622' },
  icmp: { label: 'ICMP', color: '#6366f1', bgColor: '#6366f122' },
  fcmp: { label: 'FCMP', color: '#8b5cf6', bgColor: '#8b5cf622' },
  phi: { label: 'PHI', color: '#0ea5e9', bgColor: '#0ea5e922' },
}

// ============================================
// 工具函数
// ============================================

/** 检查是否为关键指令 */
function getKeyInstructionInfo(opcode: string) {
  const lowerOpcode = opcode.toLowerCase()
  return KEY_INSTRUCTIONS[lowerOpcode as keyof typeof KEY_INSTRUCTIONS] || null
}

// ============================================
// 语法高亮组件
// ============================================

interface LlvmCodeLineProps {
  line: string
  lineNumber: number
  instruction?: LlvmInstruction | null
  onClick?: () => void
  theme: 'dark' | 'light'
  highlighted?: boolean
}

function LlvmCodeLine({ line, lineNumber, instruction, onClick, theme, highlighted = false }: LlvmCodeLineProps) {
  const keyInfo = instruction ? getKeyInstructionInfo(instruction.opcode) : null

  // 语法高亮解析
  const highlightedLine = useMemo(() => {
    // 移除首尾空白
    const trimmed = line.trim()
    if (!trimmed) return { parts: [{ text: line, type: 'whitespace' }] }

    // 标签行 (基本块)
    if (trimmed.endsWith(':') && !trimmed.startsWith(';')) {
      return {
        parts: [{ text: trimmed, type: 'label' }],
        isBlockLabel: true
      }
    }

    // 指令行解析
    const parts: { text: string; type: string }[] = []
    let currentText = ''
    let currentType = 'text'
    let inString = false
    let inComment = false
    let i = 0

    while (i < line.length) {
      const char = line[i]

      // 处理注释
      if (char === ';' && !inString) {
        if (currentText) {
          parts.push({ text: currentText, type: currentType })
          currentText = ''
        }
        inComment = true
        currentType = 'comment'
        currentText += char
        i++
        continue
      }

      // 注释内的内容
      if (inComment) {
        currentText += char
        i++
        continue
      }

      // 处理字符串
      if ((char === '"' || char === '<') && !inString) {
        if (currentText) {
          parts.push({ text: currentText, type: currentType })
        }
        inString = true
        currentType = 'string'
        currentText = char
        i++
        continue
      }

      if ((char === '"' || char === '>') && inString) {
        currentText += char
        inString = false
        parts.push({ text: currentText, type: currentType })
        currentText = ''
        currentType = 'text'
        i++
        continue
      }

      // 字符串内
      if (inString) {
        currentText += char
        i++
        continue
      }

      // 寄存器/变量名 (以 % 开头)
      if (char === '%') {
        if (currentText) {
          parts.push({ text: currentText, type: currentType })
        }
        currentText = char
        currentType = 'register'
        i++
        while (i < line.length && (/\w/.test(line[i]) || line[i] === '.')) {
          currentText += line[i]
          i++
        }
        parts.push({ text: currentText, type: currentType })
        currentText = ''
        currentType = 'text'
        continue
      }

      // 类型名 (常见 LLVM 类型)
      if (/^[a-z]+\d*$/.test(char) || char === '*') {
        if (currentText && !/\s/.test(currentText[currentText.length - 1])) {
          // 检查当前文本末尾是否已经有类型
          const lastPart = parts[parts.length - 1]
          if (lastPart && lastPart.type !== 'text') {
            currentText += char
            i++
            continue
          }
        }
      }

      // 关键词检测
      const keywords = ['define', 'declare', 'entry', 'label', 'then', 'else', 'for', 'if', 'switch']
      const keyword = keywords.find(kw => line.substring(i).startsWith(kw + ' ') || line.substring(i).startsWith(kw + '('))

      if (keyword) {
        if (currentText) {
          parts.push({ text: currentText, type: currentType })
        }
        currentText = keyword
        currentType = 'keyword'
        i += keyword.length
        parts.push({ text: currentText, type: currentType })
        currentText = ''
        currentType = 'text'
        continue
      }

      // 操作码检测
      if (/^[a-z]+\.?/.test(char)) {
        if (currentText && !/\s/.test(currentText[currentText.length - 1])) {
          currentText += char
          i++
          continue
        }
        const match = line.substring(i).match(/^[a-z]+/)
        if (match) {
          if (currentText) {
            parts.push({ text: currentText, type: currentType })
          }
          currentText = match[0]
          currentType = 'opcode'
          i += match[0].length
          parts.push({ text: currentText, type: currentType })
          currentText = ''
          currentType = 'text'
          continue
        }
      }

      // 数字常量
      if (/\d/.test(char)) {
        if (currentText && !/\s/.test(currentText[currentText.length - 1])) {
          currentText += char
          i++
          continue
        }
        if (currentText) {
          parts.push({ text: currentText, type: currentType })
        }
        currentText = char
        currentType = 'number'
        i++
        while (i < line.length && (/\d/.test(line[i]) || line[i] === '.')) {
          currentText += line[i]
          i++
        }
        parts.push({ text: currentText, type: currentType })
        currentText = ''
        currentType = 'text'
        continue
      }

      // 默认字符处理
      if (currentText && (/\s/.test(char) !== /\s/.test(currentText[currentText.length - 1]))) {
        parts.push({ text: currentText, type: currentType })
        currentText = ''
        currentType = 'text'
      }
      currentText += char
      i++
    }

    if (currentText) {
      parts.push({ text: currentText, type: currentType })
    }

    return { parts, isBlockLabel: false }
  }, [line])

  return (
    <div
      className={`llvm-code-line ${keyInfo ? 'key-instruction' : ''} ${theme} ${highlighted ? 'highlighted' : ''}`}
      onClick={onClick}
      style={keyInfo ? { backgroundColor: `${keyInfo.bgColor}33` } : highlighted ? { backgroundColor: 'rgba(88, 166, 255, 0.15)' } : undefined}
    >
      <span className="line-number">{lineNumber}</span>
      <span className="line-content">
        {highlightedLine.parts.map((part, idx) => (
          <span key={idx} className={`token ${part.type}`}>
            {part.text}
          </span>
        ))}
      </span>
      {keyInfo && (
        <span className="key-badge" style={{ color: keyInfo.color, backgroundColor: keyInfo.bgColor }}>
          {keyInfo.label}
        </span>
      )}
      {highlighted && !keyInfo && (
        <span className="highlight-indicator">→</span>
      )}
    </div>
  )
}

// ============================================
// 基本块组件
// ============================================

interface BasicBlockProps {
  block: LlvmBasicBlock
  lineOffset: number
  expanded: boolean
  theme: 'dark' | 'light'
  highlighted?: boolean
  highlightedIndex?: number | null
  onToggle: () => void
  onInstructionClick?: (instruction: LlvmInstruction) => void
}

function BasicBlock({
  block,
  lineOffset,
  expanded,
  theme,
  highlighted = false,
  highlightedIndex = null,
  onToggle,
  onInstructionClick
}: BasicBlockProps) {
  // 生成块的完整 IR 文本
  const blockIrText = useMemo(() => {
    const lines: string[] = []

    // 添加标签
    lines.push(`${block.name}:`)

    // 添加指令
    if (expanded) {
      block.instructions.forEach((instr, _idx) => {
        if (instr.dest) {
          lines.push(`  %${instr.dest} = ${instr.opcode} ${instr.typeStr} ${instr.operands.join(', ')}`)
        } else {
          lines.push(`  ${instr.opcode} ${instr.operands.join(', ')}`)
        }
      })

      // 添加终止符
      if (block.terminator) {
        if (block.terminator.dest) {
          lines.push(`  %${block.terminator.dest} = ${block.terminator.opcode} ${block.terminator.typeStr} ${block.terminator.operands.join(', ')}`)
        } else {
          lines.push(`  ${block.terminator.opcode} ${block.terminator.operands.join(', ')}`)
        }
      }
    }

    return lines.join('\n')
  }, [block, expanded])

  // 解析行
  const lines = blockIrText.split('\n')

  // 自动展开高亮的块
  useEffect(() => {
    if (highlighted && !expanded) {
      onToggle()
    }
  }, [highlighted, expanded, onToggle])

  return (
    <div className={`basic-block ${expanded ? 'expanded' : 'collapsed'} ${theme} ${highlighted ? 'highlighted' : ''}`}>
      <div className="block-header" onClick={onToggle}>
        <span className="block-toggle">{expanded ? '▼' : '▶'}</span>
        <span className="block-name">{block.name}</span>
        <span className="block-info">
          {expanded ? `${block.instructions.length} 条指令` : `${block.instructions.length} 条指令 (点击展开)`}
        </span>
        {block.predecessors.length > 0 && (
          <span className="block-predecessors" title="前驱块">
            ← {block.predecessors.join(', ')}
          </span>
        )}
        {block.successors.length > 0 && (
          <span className="block-successors" title="后继块">
            → {block.successors.join(', ')}
          </span>
        )}
      </div>

      {expanded && (
        <div className="block-content">
          {lines.map((line, idx) => {
            const lineNumber = lineOffset + idx + 1
            const isLabel = line.trim().endsWith(':')
            const instructionIdx = isLabel ? 0 : idx - 1
            const instruction = !isLabel && instructionIdx < block.instructions.length
              ? block.instructions[instructionIdx]
              : null
            const isHighlighted = highlighted && highlightedIndex === instructionIdx

            return (
              <LlvmCodeLine
                key={idx}
                line={line}
                lineNumber={lineNumber}
                instruction={instruction}
                theme={theme}
                highlighted={isHighlighted}
                onClick={() => instruction && onInstructionClick?.(instruction)}
              />
            )
          })}
        </div>
      )}
    </div>
  )
}

// ============================================
// 主组件
// ============================================

export function LlvmIrPanel({
  title = 'LLVM IR 可视化',
  parseResult,
  selectedFunction,
  loading = false,
  error = null,
  theme = 'dark',
  comparisonMode = false,
  compareFunction = null,
  highlightedInstruction = null,
  onFunctionSelect,
  onCompareSelect,
  onToggleComparison,
  onInstructionClick,
  onBlockClick,
}: LlvmIrPanelProps) {
  // 展开状态管理
  const [expandedBlocks, setExpandedBlocks] = useState<Record<string, boolean>>({})
  const [searchQuery, setSearchQuery] = useState('')

  // 切换块的展开状态
  const toggleBlock = useCallback((blockName: string) => {
    setExpandedBlocks(prev => ({
      ...prev,
      [blockName]: !prev[blockName]
    }))
  }, [])

  // 获取当前选中的函数
  const currentFunction = selectedFunction && parseResult?.functions
    ? parseResult.functions[selectedFunction]
    : null

  // 获取对比函数
  const compareFunc = compareFunction && parseResult?.functions
    ? parseResult.functions[compareFunction]
    : null

  // 过滤搜索结果
  const filteredBlocks = useMemo(() => {
    if (!currentFunction || !searchQuery.trim()) {
      return currentFunction?.blocks || []
    }

    const query = searchQuery.toLowerCase()
    return currentFunction.blocks.filter(block =>
      block.name.toLowerCase().includes(query) ||
      block.instructions.some(instr =>
        instr.opcode.toLowerCase().includes(query) ||
        instr.operands.some(op => op.toLowerCase().includes(query))
      )
    )
  }, [currentFunction, searchQuery])

  // 计算总行数（用于行号）
  const totalLines = useMemo(() => {
    if (!currentFunction) return 0

    let lines = 0
    currentFunction.blocks.forEach(block => {
      lines += 1 + block.instructions.length + (block.terminator ? 1 : 0)
    })
    return lines
  }, [currentFunction])

  // 检查块是否高亮
  const isBlockHighlighted = useCallback((blockName: string) => {
    if (!highlightedInstruction) return false
    return highlightedInstruction.blockName === blockName
  }, [highlightedInstruction])

  // 获取高亮的指令索引
  const getHighlightedIndex = useCallback((blockName: string) => {
    if (!highlightedInstruction) return null
    if (highlightedInstruction.blockName !== blockName) return null
    return highlightedInstruction.instructionIndex
  }, [highlightedInstruction])

  // 渲染加载状态
  if (loading) {
    return (
      <div className={`llvm-ir-panel ${theme}`}>
        <div className="panel-header">
          <h3>{title}</h3>
        </div>
        <div className="panel-loading">
          <div className="loading-spinner" />
          <span>加载 LLVM IR 数据...</span>
        </div>
      </div>
    )
  }

  // 渲染错误状态
  if (error) {
    return (
      <div className={`llvm-ir-panel ${theme}`}>
        <div className="panel-header">
          <h3>{title}</h3>
        </div>
        <div className="panel-error">
          <span className="error-icon">⚠</span>
          <span>{error}</span>
        </div>
      </div>
    )
  }

  // 渲染空状态
  if (!parseResult) {
    return (
      <div className={`llvm-ir-panel ${theme}`}>
        <div className="panel-header">
          <h3>{title}</h3>
        </div>
        <div className="panel-empty">
          <span className="empty-icon">📋</span>
          <p>暂无 LLVM IR 数据</p>
          <span className="empty-hint">请先加载 LLVM IR 文件</span>
        </div>
      </div>
    )
  }

  // 渲染空搜索结果
  if (currentFunction && filteredBlocks.length === 0 && searchQuery.trim()) {
    return (
      <div className={`llvm-ir-panel ${theme}`}>
        <div className="panel-header">
          <h3>{title}</h3>
          {selectedFunction && <span className="function-badge">{selectedFunction}()</span>}
        </div>
        <div className="panel-search-empty">
          <span className="search-icon">🔍</span>
          <p>未找到匹配 "{searchQuery}" 的基本块</p>
        </div>
      </div>
    )
  }

  // 渲染主要内容
  return (
    <div className={`llvm-ir-panel ${theme} ${comparisonMode ? 'comparison-mode' : ''}`}>
      <div className="panel-header">
        <h3>{title}</h3>
        {selectedFunction && (
          <span className="function-badge">
            {selectedFunction}()
            {currentFunction?.isCallback && (
              <span className="callback-indicator" title="回调函数">⚡</span>
            )}
          </span>
        )}
        <span className="ir-count">
          {Object.keys(parseResult.functions).length} 个函数
        </span>
        <button
          className={`comparison-toggle ${comparisonMode ? 'active' : ''}`}
          onClick={() => onToggleComparison?.(!comparisonMode)}
          title={comparisonMode ? '关闭对比模式' : '开启对比模式'}
        >
          {comparisonMode ? '取消对比' : '对比'}
        </button>
      </div>

      {/* 主函数区域 */}
      <div className={comparisonMode ? 'comparison-panel' : ''}>
        <div className="comparison-header">
          主函数: {selectedFunction || '未选择'}
        </div>

        {/* 函数选择器 */}
        <div className="function-selector">
          <select
            value={selectedFunction || ''}
            onChange={(e) => onFunctionSelect?.(e.target.value || null)}
            className="function-select"
          >
            <option value="">选择函数...</option>
            {Object.keys(parseResult.functions).map(funcName => (
              <option key={funcName} value={funcName}>
                {funcName}
                {parseResult.functions[funcName].isCallback ? ' ⚡' : ''}
              </option>
            ))}
          </select>
        </div>

        {/* 搜索框 */}
        <div className="search-box">
          <input
            type="text"
            placeholder="搜索指令、寄存器..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="search-input"
          />
        </div>

        {/* 函数信息 */}
        {currentFunction && (
          <div className="function-info">
            <div className="function-signature">
              <span className="keyword">define</span>{' '}
              <span className="type">{currentFunction.returnType}</span>{' '}
              <span className="function-name">@{currentFunction.name}</span>
              <span className="params">({currentFunction.parameters.map(p => `${p.typeStr} %${p.name}`).join(', ')})</span>
            </div>
            {currentFunction.isCallback && (
              <div className="callback-badge">
                <span className="callback-icon">⚡</span>
                <span>回调函数</span>
                {currentFunction.callbackContext && (
                  <span className="callback-context">({currentFunction.callbackContext})</span>
                )}
              </div>
            )}
          </div>
        )}

        {/* 基本块列表 */}
        <div className="blocks-container">
          {filteredBlocks.map((block, idx) => {
            const lineOffset = idx // 简化行号计算
            return (
              <BasicBlock
                key={block.name}
                block={block}
                lineOffset={lineOffset}
                expanded={expandedBlocks[block.name] !== false}
                theme={theme}
                highlighted={isBlockHighlighted(block.name)}
                highlightedIndex={getHighlightedIndex(block.name)}
                onToggle={() => {
                  toggleBlock(block.name)
                  onBlockClick?.(block.name)
                }}
                onInstructionClick={(instr) => onInstructionClick?.(instr, block.name)}
              />
            )
          })}
        </div>
      </div>

      {/* 对比函数区域 */}
      {comparisonMode && (
        <div className="comparison-panel">
          <div className="comparison-header">
            对比函数: {compareFunction || '未选择'}
          </div>

          {/* 对比函数选择器 */}
          <div className="function-selector">
            <select
              value={compareFunction || ''}
              onChange={(e) => onCompareSelect?.(e.target.value || null)}
              className="function-select"
            >
              <option value="">选择函数...</option>
              {Object.keys(parseResult.functions)
                .filter(fn => fn !== selectedFunction)
                .map(funcName => (
                <option key={funcName} value={funcName}>
                  {funcName}
                  {parseResult.functions[funcName].isCallback ? ' ⚡' : ''}
                </option>
              ))}
            </select>
          </div>

          {/* 对比函数信息 */}
          {compareFunc && (
            <div className="function-info">
              <div className="function-signature">
                <span className="keyword">define</span>{' '}
                <span className="type">{compareFunc.returnType}</span>{' '}
                <span className="function-name">@{compareFunc.name}</span>
                <span className="params">({compareFunc.parameters.map(p => `${p.typeStr} %${p.name}`).join(', ')})</span>
              </div>
              {compareFunc.isCallback && (
                <div className="callback-badge">
                  <span className="callback-icon">⚡</span>
                  <span>回调函数</span>
                  {compareFunc.callbackContext && (
                    <span className="callback-context">({compareFunc.callbackContext})</span>
                  )}
                </div>
              )}
            </div>
          )}

          {/* 对比函数基本块列表 */}
          <div className="blocks-container">
            {compareFunc?.blocks.map((block, idx) => (
              <BasicBlock
                key={block.name}
                block={block}
                lineOffset={idx}
                expanded={expandedBlocks[`${compareFunction}:${block.name}`] !== false}
                theme={theme}
                onToggle={() => toggleBlock(`${compareFunction}:${block.name}`)}
              />
            ))}
          </div>
        </div>
      )}

      {/* 关键指令图例 */}
      <div className="instruction-legend">
        <span className="legend-title">关键指令:</span>
        {Object.entries(KEY_INSTRUCTIONS).map(([opcode, info]) => (
          <span
            key={opcode}
            className="legend-item"
            style={{ color: info.color, backgroundColor: info.bgColor }}
          >
            {info.label}
          </span>
        ))}
      </div>

      {/* 面板底部 */}
      <div className="panel-footer">
        <span className="module-name">
          模块: {parseResult.moduleName}
        </span>
        <span className="footer-hint">
          {totalLines} 行 IR 代码
          {comparisonMode && compareFunction && ' × 2 (对比模式)'}
        </span>
      </div>
    </div>
  )
}

export default LlvmIrPanel
