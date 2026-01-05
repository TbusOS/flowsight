/**
 * 欢迎页面组件
 * 
 * 显示快速入门指南和快捷键说明
 */

import './Welcome.css'

interface WelcomeProps {
  onOpenFile: () => void
  onOpenProject: () => void
}

export function Welcome({ onOpenFile, onOpenProject }: WelcomeProps) {
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

        {/* Shortcuts */}
        <div className="welcome-shortcuts">
          <h3>⌨️ 快捷键</h3>
          <div className="shortcuts-grid">
            <div className="shortcut">
              <kbd>Ctrl</kbd> + <kbd>P</kbd>
              <span>命令面板</span>
            </div>
            <div className="shortcut">
              <kbd>Ctrl</kbd> + <kbd>S</kbd>
              <span>保存文件</span>
            </div>
            <div className="shortcut">
              <kbd>Alt</kbd> + <kbd>←</kbd>
              <span>后退导航</span>
            </div>
            <div className="shortcut">
              <kbd>Alt</kbd> + <kbd>→</kbd>
              <span>前进导航</span>
            </div>
            <div className="shortcut">
              <kbd>1</kbd> - <kbd>5</kbd>
              <span>按层级折叠</span>
            </div>
            <div className="shortcut">
              <kbd>右键</kbd>
              <span>子树聚焦</span>
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

