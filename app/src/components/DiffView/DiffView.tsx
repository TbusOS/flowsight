/**
 * 差异视图组件
 * 
 * 简单显示文件修改前后的差异
 */

import { useMemo } from 'react'
import './DiffView.css'

interface DiffViewProps {
  original: string
  modified: string
  fileName?: string
}

interface DiffLine {
  type: 'unchanged' | 'added' | 'removed'
  content: string
  lineNumber: { old?: number; new?: number }
}

function computeDiff(original: string, modified: string): DiffLine[] {
  const oldLines = original.split('\n')
  const newLines = modified.split('\n')
  const result: DiffLine[] = []
  
  // 简单的逐行比较（非真正的 diff 算法）
  let oldIdx = 0
  let newIdx = 0
  
  while (oldIdx < oldLines.length || newIdx < newLines.length) {
    const oldLine = oldLines[oldIdx]
    const newLine = newLines[newIdx]
    
    if (oldLine === newLine) {
      result.push({
        type: 'unchanged',
        content: oldLine,
        lineNumber: { old: oldIdx + 1, new: newIdx + 1 }
      })
      oldIdx++
      newIdx++
    } else if (newIdx >= newLines.length) {
      // 剩余的旧行被删除
      result.push({
        type: 'removed',
        content: oldLine,
        lineNumber: { old: oldIdx + 1 }
      })
      oldIdx++
    } else if (oldIdx >= oldLines.length) {
      // 新增的行
      result.push({
        type: 'added',
        content: newLine,
        lineNumber: { new: newIdx + 1 }
      })
      newIdx++
    } else {
      // 行不匹配，标记为删除+新增
      result.push({
        type: 'removed',
        content: oldLine,
        lineNumber: { old: oldIdx + 1 }
      })
      result.push({
        type: 'added',
        content: newLine,
        lineNumber: { new: newIdx + 1 }
      })
      oldIdx++
      newIdx++
    }
  }
  
  return result
}

export function DiffView({ original, modified, fileName }: DiffViewProps) {
  const diff = useMemo(() => computeDiff(original, modified), [original, modified])
  
  const stats = useMemo(() => {
    let added = 0
    let removed = 0
    diff.forEach(line => {
      if (line.type === 'added') added++
      if (line.type === 'removed') removed++
    })
    return { added, removed }
  }, [diff])

  return (
    <div className="diff-view">
      <div className="diff-header">
        <span className="diff-filename">{fileName || '未命名文件'}</span>
        <div className="diff-stats">
          <span className="stat added">+{stats.added}</span>
          <span className="stat removed">-{stats.removed}</span>
        </div>
      </div>
      
      <div className="diff-content">
        {diff.map((line, index) => (
          <div key={index} className={`diff-line ${line.type}`}>
            <span className="line-numbers">
              <span className="old-num">{line.lineNumber.old || ''}</span>
              <span className="new-num">{line.lineNumber.new || ''}</span>
            </span>
            <span className="line-prefix">
              {line.type === 'added' ? '+' : line.type === 'removed' ? '-' : ' '}
            </span>
            <span className="line-content">{line.content || ' '}</span>
          </div>
        ))}
      </div>
    </div>
  )
}

export default DiffView

