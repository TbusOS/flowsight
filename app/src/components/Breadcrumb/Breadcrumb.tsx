/**
 * 面包屑导航组件
 * 
 * 显示当前文件路径和函数位置
 */

import './Breadcrumb.css'

interface BreadcrumbProps {
  /** 项目根目录 */
  projectRoot?: string
  /** 当前文件路径 */
  filePath: string
  /** 当前函数名 */
  currentFunction?: string | null
  /** 点击路径部分的回调 */
  onPathClick?: (path: string) => void
  /** 点击函数的回调 */
  onFunctionClick?: () => void
}

export function Breadcrumb({
  projectRoot,
  filePath,
  currentFunction,
  onPathClick,
  onFunctionClick,
}: BreadcrumbProps) {
  if (!filePath) {
    return (
      <div className="breadcrumb">
        <span className="breadcrumb-empty">未打开文件</span>
      </div>
    )
  }

  // 计算相对路径
  let displayPath = filePath
  if (projectRoot && filePath.startsWith(projectRoot)) {
    displayPath = filePath.slice(projectRoot.length).replace(/^\//, '')
  }

  // 分割路径
  const parts = displayPath.split('/')
  const fileName = parts.pop() || ''
  const directories = parts

  // 构建可点击的路径部分
  const buildPath = (index: number): string => {
    if (projectRoot) {
      return projectRoot + '/' + directories.slice(0, index + 1).join('/')
    }
    return '/' + directories.slice(0, index + 1).join('/')
  }

  return (
    <div className="breadcrumb">
      {/* 项目根目录图标 */}
      {projectRoot && (
        <>
          <span 
            className="breadcrumb-item clickable"
            onClick={() => onPathClick?.(projectRoot)}
            title={projectRoot}
          >
            📁
          </span>
          <span className="breadcrumb-separator">/</span>
        </>
      )}

      {/* 目录路径 */}
      {directories.map((dir, index) => (
        <span key={index}>
          <span
            className="breadcrumb-item clickable"
            onClick={() => onPathClick?.(buildPath(index))}
            title={buildPath(index)}
          >
            {dir}
          </span>
          <span className="breadcrumb-separator">/</span>
        </span>
      ))}

      {/* 文件名 */}
      <span className="breadcrumb-item file">
        {getFileIcon(fileName)} {fileName}
      </span>

      {/* 当前函数 */}
      {currentFunction && (
        <>
          <span className="breadcrumb-separator">›</span>
          <span
            className="breadcrumb-item function clickable"
            onClick={onFunctionClick}
            title={`跳转到 ${currentFunction}()`}
          >
            ƒ {currentFunction}()
          </span>
        </>
      )}
    </div>
  )
}

// 获取文件图标
function getFileIcon(fileName: string): string {
  const ext = fileName.split('.').pop()?.toLowerCase()
  switch (ext) {
    case 'c': return '📄'
    case 'h': return '📋'
    case 'cpp':
    case 'cc':
    case 'cxx': return '📄'
    case 'rs': return '🦀'
    case 'py': return '🐍'
    case 'js':
    case 'ts': return '📜'
    case 'md': return '📝'
    case 'json': return '⚙️'
    default: return '📄'
  }
}

export default Breadcrumb

