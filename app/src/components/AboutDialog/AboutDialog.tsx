/**
 * 关于对话框组件
 */

import './AboutDialog.css'

interface AboutDialogProps {
  isOpen: boolean
  onClose: () => void
}

export function AboutDialog({ isOpen, onClose }: AboutDialogProps) {
  if (!isOpen) return null

  return (
    <div className="about-overlay" onClick={onClose}>
      <div className="about-dialog" onClick={e => e.stopPropagation()}>
        <button className="close-btn" onClick={onClose}>✕</button>
        
        <div className="about-content">
          {/* Logo */}
          <div className="about-logo">🔭</div>
          
          {/* 标题 */}
          <h1 className="about-title">FlowSight</h1>
          <p className="about-version">版本 0.1.0</p>
          <p className="about-tagline">看见代码的"灵魂" — 执行流可视化 IDE</p>
          
          {/* 描述 */}
          <div className="about-description">
            <p>
              FlowSight 是一款专为 Linux 内核开发者设计的 IDE，
              帮助您理解异步调用、回调模式和执行流程。
            </p>
          </div>
          
          {/* 特性 */}
          <div className="about-features">
            <div className="feature">
              <span className="feature-icon">📊</span>
              <span>执行流可视化</span>
            </div>
            <div className="feature">
              <span className="feature-icon">🔍</span>
              <span>函数指针解析</span>
            </div>
            <div className="feature">
              <span className="feature-icon">⚡</span>
              <span>异步机制追踪</span>
            </div>
            <div className="feature">
              <span className="feature-icon">📝</span>
              <span>ftrace 风格输出</span>
            </div>
          </div>
          
          {/* 技术栈 */}
          <div className="about-tech">
            <span>Tauri</span>
            <span>•</span>
            <span>React</span>
            <span>•</span>
            <span>Rust</span>
            <span>•</span>
            <span>Tree-sitter</span>
          </div>
          
          {/* 链接 */}
          <div className="about-links">
            <a 
              href="https://github.com/TbusOS/flowsight" 
              target="_blank" 
              rel="noopener noreferrer"
            >
              🌐 GitHub
            </a>
            <a 
              href="https://github.com/TbusOS/flowsight/issues" 
              target="_blank" 
              rel="noopener noreferrer"
            >
              🐛 报告问题
            </a>
            <a 
              href="https://github.com/TbusOS/flowsight/blob/main/docs/design/PROJECT-PLAN.md" 
              target="_blank" 
              rel="noopener noreferrer"
            >
              📖 文档
            </a>
          </div>
          
          {/* 版权 */}
          <div className="about-copyright">
            <p>© 2026 TbusOS Team</p>
            <p>MIT License</p>
          </div>
        </div>
      </div>
    </div>
  )
}

export default AboutDialog

