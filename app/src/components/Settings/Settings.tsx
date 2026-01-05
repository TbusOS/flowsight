/**
 * 设置面板组件
 */

import { useState } from 'react'
import './Settings.css'

export interface AppSettings {
  autoSave: boolean
  autoSaveDelay: number // 毫秒
  fontSize: number
  theme: 'dark' | 'light'
  hideKernelApiByDefault: boolean
}

const defaultSettings: AppSettings = {
  autoSave: true,
  autoSaveDelay: 2000,
  fontSize: 14,
  theme: 'dark',
  hideKernelApiByDefault: false,
}

interface SettingsProps {
  isOpen: boolean
  onClose: () => void
  settings: AppSettings
  onSettingsChange: (settings: AppSettings) => void
}

export function Settings({ isOpen, onClose, settings, onSettingsChange }: SettingsProps) {
  const [localSettings, setLocalSettings] = useState(settings)

  if (!isOpen) return null

  const handleSave = () => {
    onSettingsChange(localSettings)
    onClose()
  }

  const handleReset = () => {
    setLocalSettings(defaultSettings)
  }

  return (
    <div className="settings-overlay" onClick={onClose}>
      <div className="settings-panel" onClick={e => e.stopPropagation()}>
        <div className="settings-header">
          <h2>⚙️ 设置</h2>
          <button className="close-btn" onClick={onClose}>✕</button>
        </div>

        <div className="settings-content">
          {/* 编辑器设置 */}
          <section className="settings-section">
            <h3>📝 编辑器</h3>
            
            <div className="setting-item">
              <label>
                <input
                  type="checkbox"
                  checked={localSettings.autoSave}
                  onChange={e => setLocalSettings(prev => ({
                    ...prev,
                    autoSave: e.target.checked
                  }))}
                />
                自动保存
              </label>
              <span className="setting-desc">文件修改后自动保存</span>
            </div>

            {localSettings.autoSave && (
              <div className="setting-item indent">
                <label>延迟时间</label>
                <select
                  value={localSettings.autoSaveDelay}
                  onChange={e => setLocalSettings(prev => ({
                    ...prev,
                    autoSaveDelay: parseInt(e.target.value)
                  }))}
                >
                  <option value={1000}>1 秒</option>
                  <option value={2000}>2 秒</option>
                  <option value={3000}>3 秒</option>
                  <option value={5000}>5 秒</option>
                </select>
              </div>
            )}

            <div className="setting-item">
              <label>字体大小</label>
              <input
                type="range"
                min="12"
                max="20"
                value={localSettings.fontSize}
                onChange={e => setLocalSettings(prev => ({
                  ...prev,
                  fontSize: parseInt(e.target.value)
                }))}
              />
              <span className="value">{localSettings.fontSize}px</span>
            </div>
          </section>

          {/* 执行流设置 */}
          <section className="settings-section">
            <h3>📊 执行流</h3>
            
            <div className="setting-item">
              <label>
                <input
                  type="checkbox"
                  checked={localSettings.hideKernelApiByDefault}
                  onChange={e => setLocalSettings(prev => ({
                    ...prev,
                    hideKernelApiByDefault: e.target.checked
                  }))}
                />
                默认隐藏内核 API
              </label>
              <span className="setting-desc">自动过滤 kmalloc、printk 等函数</span>
            </div>
          </section>

          {/* 外观设置 */}
          <section className="settings-section">
            <h3>🎨 外观</h3>
            
            <div className="setting-item">
              <label>主题</label>
              <select
                value={localSettings.theme}
                onChange={e => setLocalSettings(prev => ({
                  ...prev,
                  theme: e.target.value as 'dark' | 'light'
                }))}
              >
                <option value="dark">深色</option>
                <option value="light">浅色 (开发中)</option>
              </select>
            </div>
          </section>
        </div>

        <div className="settings-footer">
          <button className="btn secondary" onClick={handleReset}>
            恢复默认
          </button>
          <button className="btn primary" onClick={handleSave}>
            保存设置
          </button>
        </div>
      </div>
    </div>
  )
}

export { defaultSettings }
export default Settings

