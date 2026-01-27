/**
 * 状态栏组件
 *
 * 显示当前文件信息、分析状态等
 */

import './StatusBar.css'
import { Loader2, CheckCircle2, XCircle, Circle, FileCode, Dot } from 'lucide-react'

interface StatusBarProps {
  /** 当前文件路径 */
  filePath?: string
  /** 函数数量 */
  functionCount?: number
  /** 分析状态 */
  analysisStatus?: 'idle' | 'analyzing' | 'done' | 'error'
  /** 当前选中行号 */
  currentLine?: number
  /** 当前列号 */
  currentColumn?: number
  /** 文件是否已修改 */
  isDirty?: boolean
  /** 文件内容 (用于统计) */
  fileContent?: string
  /** 选中的文本长度 */
  selectionLength?: number
}

export function StatusBar({
  filePath,
  functionCount = 0,
  analysisStatus = 'idle',
  currentLine,
  currentColumn,
  isDirty = false,
  fileContent,
  selectionLength,
}: StatusBarProps) {
  // 计算文件统计
  const lineCount = fileContent?.split('\n').length || 0
  const charCount = fileContent?.length || 0
  // 获取文件语言
  const getLanguage = (path: string): string => {
    const ext = path.split('.').pop()?.toLowerCase()
    switch (ext) {
      case 'c': return 'C'
      case 'h': return 'C Header'
      case 'cpp':
      case 'cc':
      case 'cxx': return 'C++'
      case 'rs': return 'Rust'
      case 'py': return 'Python'
      case 'js': return 'JavaScript'
      case 'ts':
      case 'tsx': return 'TypeScript'
      default: return ext?.toUpperCase() || 'Plain Text'
    }
  }

  // 获取状态图标组件
  const getStatusIcon = () => {
    const iconProps = {
      className: "w-3.5 h-3.5",
      strokeWidth: 2,
    }
    switch (analysisStatus) {
      case 'analyzing':
        return <Loader2 {...iconProps} className="animate-spin" />
      case 'done':
        return <CheckCircle2 {...iconProps} />
      case 'error':
        return <XCircle {...iconProps} />
      default:
        return <Circle {...iconProps} />
    }
  }

  const getStatusText = () => {
    switch (analysisStatus) {
      case 'analyzing': return '分析中...'
      case 'done': return '分析完成'
      case 'error': return '分析失败'
      default: return '就绪'
    }
  }

  return (
    <div className="status-bar">
      <div className="status-left">
        {/* 分析状态 */}
        <span className={`status-item status-${analysisStatus}`}>
          {getStatusIcon()} <span>{getStatusText()}</span>
        </span>

        {/* 函数数量 */}
        {functionCount > 0 && (
          <span className="status-item">
            <FileCode className="w-3.5 h-3.5" strokeWidth={2} />
            <span>{functionCount} 函数</span>
          </span>
        )}
      </div>

      <div className="status-right">
        {/* 当前位置 */}
        {currentLine && (
          <span className="status-item cursor-pos">
            <span>Ln {currentLine}{currentColumn ? `, Col ${currentColumn}` : ''}</span>
          </span>
        )}

        {/* 选中文本 */}
        {selectionLength && selectionLength > 0 && (
          <span className="status-item selection">
            已选 {selectionLength} 字符
          </span>
        )}

        {/* 文件统计 */}
        {lineCount > 0 && (
          <span className="status-item file-stats">
            <span>{lineCount} 行, {charCount.toLocaleString()} 字符</span>
          </span>
        )}

        {/* 文件修改状态 */}
        {isDirty && (
          <span className="status-item dirty">
            <Dot className="w-3.5 h-3.5" fill="currentColor" />
            <span>未保存</span>
          </span>
        )}

        {/* 文件语言 */}
        {filePath && (
          <span className="status-item language">
            <span>{getLanguage(filePath)}</span>
          </span>
        )}

        {/* FlowSight 版本 */}
        <span className="status-item version">
          <span>FlowSight v0.1.0</span>
        </span>
      </div>
    </div>
  )
}

export default StatusBar

