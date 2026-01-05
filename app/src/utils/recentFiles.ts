/**
 * 最近文件管理工具
 */

const STORAGE_KEY = 'flowsight_recent_files'
const MAX_RECENT = 10

export interface RecentFile {
  path: string
  name: string
  timestamp: number
  isProject?: boolean
}

/**
 * 获取最近文件列表
 */
export function getRecentFiles(): RecentFile[] {
  try {
    const stored = localStorage.getItem(STORAGE_KEY)
    if (!stored) return []
    return JSON.parse(stored)
  } catch {
    return []
  }
}

/**
 * 添加最近文件
 */
export function addRecentFile(path: string, isProject = false): void {
  try {
    const recent = getRecentFiles()
    
    // 移除已存在的相同路径
    const filtered = recent.filter(f => f.path !== path)
    
    // 添加到开头
    const name = path.split('/').pop() || path
    filtered.unshift({
      path,
      name,
      timestamp: Date.now(),
      isProject
    })
    
    // 限制数量
    const limited = filtered.slice(0, MAX_RECENT)
    
    localStorage.setItem(STORAGE_KEY, JSON.stringify(limited))
  } catch (e) {
    console.error('Failed to save recent file:', e)
  }
}

/**
 * 清除最近文件列表
 */
export function clearRecentFiles(): void {
  try {
    localStorage.removeItem(STORAGE_KEY)
  } catch (e) {
    console.error('Failed to clear recent files:', e)
  }
}

/**
 * 格式化时间显示
 */
export function formatTimestamp(timestamp: number): string {
  const now = Date.now()
  const diff = now - timestamp
  
  const minutes = Math.floor(diff / 60000)
  const hours = Math.floor(diff / 3600000)
  const days = Math.floor(diff / 86400000)
  
  if (minutes < 1) return '刚刚'
  if (minutes < 60) return `${minutes} 分钟前`
  if (hours < 24) return `${hours} 小时前`
  if (days < 7) return `${days} 天前`
  
  return new Date(timestamp).toLocaleDateString('zh-CN')
}

