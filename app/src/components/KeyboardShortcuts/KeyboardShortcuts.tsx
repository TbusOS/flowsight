/**
 * 快捷键帮助面板
 */

import './KeyboardShortcuts.css'

interface KeyboardShortcutsProps {
  isOpen: boolean
  onClose: () => void
}

const shortcuts = [
  {
    category: '文件操作',
    items: [
      { keys: ['Ctrl/⌘', 'S'], description: '保存当前文件' },
      { keys: ['Ctrl/⌘', 'O'], description: '打开文件' },
      { keys: ['Ctrl/⌘', 'Shift', 'O'], description: '打开项目' },
      { keys: ['Ctrl/⌘', 'W'], description: '关闭当前标签' },
    ]
  },
  {
    category: '导航',
    items: [
      { keys: ['Ctrl/⌘', 'P'], description: '打开命令面板' },
      { keys: ['Ctrl/⌘', 'E'], description: '快速打开最近文件' },
      { keys: ['Ctrl/⌘', 'G'], description: '跳转到行号' },
      { keys: ['Alt/⌥', '←'], description: '后退' },
      { keys: ['Alt/⌥', '→'], description: '前进' },
    ]
  },
  {
    category: '编辑',
    items: [
      { keys: ['Ctrl/⌘', 'F'], description: '查找' },
      { keys: ['Ctrl/⌘', 'H'], description: '查找替换' },
      { keys: ['Ctrl/⌘', 'Z'], description: '撤销' },
      { keys: ['Ctrl/⌘', 'Shift', 'Z'], description: '重做' },
      { keys: ['Ctrl/⌘', 'D'], description: '选择下一个匹配' },
    ]
  },
  {
    category: '视图',
    items: [
      { keys: ['Ctrl/⌘', 'B'], description: '切换侧边栏' },
      { keys: ['Ctrl/⌘', '1'], description: '代码视图' },
      { keys: ['Ctrl/⌘', '2'], description: '分屏视图' },
      { keys: ['Ctrl/⌘', '3'], description: '执行流视图' },
    ]
  },
  {
    category: '查找面板',
    items: [
      { keys: ['Enter'], description: '下一个匹配' },
      { keys: ['Shift', 'Enter'], description: '上一个匹配' },
      { keys: ['Esc'], description: '关闭查找面板' },
      { keys: ['F3'], description: '下一个匹配' },
      { keys: ['Shift', 'F3'], description: '上一个匹配' },
    ]
  },
  {
    category: '执行流图',
    items: [
      { keys: ['1-5'], description: '折叠到对应深度' },
      { keys: ['右键'], description: '聚焦子树 / 清除聚焦' },
      { keys: ['滚轮'], description: '缩放视图' },
      { keys: ['拖拽'], description: '平移视图' },
    ]
  }
]

export function KeyboardShortcuts({ isOpen, onClose }: KeyboardShortcutsProps) {
  if (!isOpen) return null

  return (
    <div className="shortcuts-overlay" onClick={onClose}>
      <div className="shortcuts-panel" onClick={e => e.stopPropagation()}>
        <div className="shortcuts-header">
          <h2>⌨️ 快捷键</h2>
          <button className="close-btn" onClick={onClose}>✕</button>
        </div>
        
        <div className="shortcuts-content">
          {shortcuts.map((section, i) => (
            <div key={i} className="shortcut-section">
              <h3>{section.category}</h3>
              <div className="shortcut-list">
                {section.items.map((item, j) => (
                  <div key={j} className="shortcut-item">
                    <div className="shortcut-keys">
                      {item.keys.map((key, k) => (
                        <span key={k}>
                          <kbd>{key}</kbd>
                          {k < item.keys.length - 1 && <span className="plus">+</span>}
                        </span>
                      ))}
                    </div>
                    <span className="shortcut-desc">{item.description}</span>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
        
        <div className="shortcuts-footer">
          <span className="hint">提示: 按 <kbd>?</kbd> 可快速打开此面板</span>
        </div>
      </div>
    </div>
  )
}

export default KeyboardShortcuts

