/**
 * 状态栏组件
 * 
 * 显示当前文件信息、分析状态等
 */

import './StatusBar.css'

interface StatusBarProps {
  /** 当前文件路径 */
  filePath?: string
  /** 函数数量 */
  functionCount?: number
  /** 分析状态 */
  analysisStatus?: 'idle' | 'analyzing' | 'done' | 'error'
  /** 当前选中行号 */
  currentLine?: number
  /** 文件是否已修改 */
  isDirty?: boolean
}

export function StatusBar({
  filePath,
  functionCount = 0,
  analysisStatus = 'idle',
  currentLine,
  isDirty = false,
}: StatusBarProps) {
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

  // 获取状态图标
  const getStatusIcon = () => {
    switch (analysisStatus) {
      case 'analyzing': return '⏳'
      case 'done': return '✅'
      case 'error': return '❌'
      default: return '💤'
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
          {getStatusIcon()} {getStatusText()}
        </span>
        
        {/* 函数数量 */}
        {functionCount > 0 && (
          <span className="status-item">
            ƒ {functionCount} 函数
          </span>
        )}
      </div>
      
      <div className="status-right">
        {/* 当前行号 */}
        {currentLine && (
          <span className="status-item">
            行 {currentLine}
          </span>
        )}
        
        {/* 文件修改状态 */}
        {isDirty && (
          <span className="status-item dirty">
            ● 未保存
          </span>
        )}
        
        {/* 文件语言 */}
        {filePath && (
          <span className="status-item language">
            {getLanguage(filePath)}
          </span>
        )}
        
        {/* FlowSight 版本 */}
        <span className="status-item version">
          FlowSight v0.1.0
        </span>
      </div>
    </div>
  )
}

export default StatusBar

