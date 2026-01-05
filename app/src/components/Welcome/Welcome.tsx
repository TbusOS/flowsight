/**
 * 欢迎页面组件
 * 
 * 显示快速入门指南和快捷键说明
 */

import { useState, useEffect } from 'react'
import { getRecentFiles, formatTimestamp, clearRecentFiles, type RecentFile } from '../../utils/recentFiles'
import './Welcome.css'

interface WelcomeProps {
  onOpenFile: () => void
  onOpenProject: () => void
  onOpenRecentFile?: (path: string) => void
  onOpenRecentProject?: (path: string) => void
}

export function Welcome({ onOpenFile, onOpenProject, onOpenRecentFile, onOpenRecentProject }: WelcomeProps) {
  const [recentFiles, setRecentFiles] = useState<RecentFile[]>([])

  useEffect(() => {
    setRecentFiles(getRecentFiles())
  }, [])

  const handleClearRecent = () => {
    clearRecentFiles()
    setRecentFiles([])
  }

  const handleOpenRecent = (file: RecentFile) => {
    if (file.isProject && onOpenRecentProject) {
      onOpenRecentProject(file.path)
    } else if (onOpenRecentFile) {
      onOpenRecentFile(file.path)
    }
  }

  return (
    <div className="welcome-container">
      <div className="welcome-content">
        {/* Logo & Title */}
        <div className="welcome-header">
          <div className="welcome-logo">🔭</div>
          <h1>FlowSight</h1>
          <p className="welcome-tagline">看见代码的"灵魂" — 执行流可视化 IDE</p>
        </div>

        {/* Quick Actions */}
        <div className="welcome-actions">
          <button className="action-btn primary" onClick={onOpenProject}>
            <span className="action-icon">📁</span>
            <span className="action-text">
              <strong>打开项目</strong>
              <small>选择代码目录进行分析</small>
            </span>
          </button>
          
          <button className="action-btn" onClick={onOpenFile}>
            <span className="action-icon">📄</span>
            <span className="action-text">
              <strong>打开文件</strong>
              <small>快速查看单个 C/H 文件</small>
            </span>
          </button>
        </div>

        {/* Recent Files */}
        {recentFiles.length > 0 && (
          <div className="welcome-recent">
            <div className="recent-header">
              <h3>🕐 最近打开</h3>
              <button className="clear-btn" onClick={handleClearRecent} title="清除记录">
                清除
              </button>
            </div>
            <div className="recent-list">
              {recentFiles.map((file, index) => (
                <button 
                  key={index} 
                  className="recent-item"
                  onClick={() => handleOpenRecent(file)}
                >
                  <span className="recent-icon">{file.isProject ? '📁' : '📄'}</span>
                  <span className="recent-info">
                    <span className="recent-name">{file.name}</span>
                    <span className="recent-path">{file.path}</span>
                  </span>
                  <span className="recent-time">{formatTimestamp(file.timestamp)}</span>
                </button>
              ))}
            </div>
          </div>
        )}

        {/* Shortcuts */}
        <div className="welcome-shortcuts">
          <h3>⌨️ 快捷键</h3>
          <div className="shortcuts-grid">
            <div className="shortcut">
              <kbd>Ctrl</kbd> + <kbd>P</kbd>
              <span>命令面板</span>
            </div>
            <div className="shortcut">
              <kbd>Ctrl</kbd> + <kbd>F</kbd>
              <span>查找</span>
            </div>
            <div className="shortcut">
              <kbd>Ctrl</kbd> + <kbd>S</kbd>
              <span>保存文件</span>
            </div>
            <div className="shortcut">
              <kbd>?</kbd>
              <span>快捷键帮助</span>
            </div>
            <div className="shortcut">
              <kbd>Ctrl</kbd> + <kbd>B</kbd>
              <span>切换侧边栏</span>
            </div>
            <div className="shortcut">
              <kbd>Ctrl</kbd> + <kbd>1/2/3</kbd>
              <span>切换视图</span>
            </div>
          </div>
        </div>

        {/* Features */}
        <div className="welcome-features">
          <h3>✨ 核心功能</h3>
          <ul>
            <li>
              <span className="feature-icon">📊</span>
              <span>执行流可视化 — 理解异步调用、回调模式</span>
            </li>
            <li>
              <span className="feature-icon">🔍</span>
              <span>函数指针解析 — 追踪 ops 表、变量赋值</span>
            </li>
            <li>
              <span className="feature-icon">⚡</span>
              <span>异步机制追踪 — 工作队列、定时器、中断</span>
            </li>
            <li>
              <span className="feature-icon">📝</span>
              <span>多视图切换 — 图形、ftrace 风格、树形</span>
            </li>
          </ul>
        </div>

        {/* Footer */}
        <div className="welcome-footer">
          <a href="https://github.com/TbusOS/flowsight" target="_blank" rel="noopener noreferrer">
            GitHub
          </a>
          <span className="separator">•</span>
          <a href="https://github.com/TbusOS/flowsight/blob/main/docs/design/PROJECT-PLAN.md" target="_blank" rel="noopener noreferrer">
            文档
          </a>
          <span className="separator">•</span>
          <span className="version">v0.1.0</span>
        </div>
      </div>
    </div>
  )
}

export default Welcome
