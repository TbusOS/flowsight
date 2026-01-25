/**
 * Icons - 现代化 SVG 图标组件
 *
 * 简洁的图标设计，用于替代 emoji
 */

import React from 'react'

// SVG 属性类型
interface IconProps {
  size?: number
  className?: string
  color?: string
  strokeWidth?: number
  style?: React.CSSProperties
}

// 图标组件包装器
const IconWrapper: React.FC<{
  children: React.ReactNode
  className?: string
  size?: number
  style?: React.CSSProperties
}> = ({ children, className, size = 16, style }) => (
  <svg
    width={size}
    height={size}
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
    className={className}
    style={style}
  >
    {children}
  </svg>
)

// 基础图标
export const Icons = {
  // 文件夹图标
  Folder: ({ size = 14, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
    </IconWrapper>
  ),

  // 展开的文件夹
  FolderOpen: ({ size = 14, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
      <line x1="9" y1="13" x2="15" y2="13" />
      <line x1="9" y1="17" x2="15" y2="17" />
    </IconWrapper>
  ),

  // 函数/方法图标
  Function: ({ size = 14, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <polyline points="16 18 22 12 16 6" />
      <polyline points="8 6 2 12 8 18" />
    </IconWrapper>
  ),

  // 入口点图标
  EntryPoint: ({ size = 14, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <circle cx="12" cy="12" r="10" />
      <polygon points="10 8 16 12 10 16 10 8" fill="currentColor" stroke="none" />
    </IconWrapper>
  ),

  // API/齿轮图标
  Api: ({ size = 14, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <circle cx="12" cy="12" r="3" />
      <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
    </IconWrapper>
  ),

  // 外部链接
  ExternalLink: ({ size = 14, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
      <polyline points="15 3 21 3 21 9" />
      <line x1="10" y1="14" x2="21" y2="3" />
    </IconWrapper>
  ),

  // 闪电/异步图标
  Zap: ({ size = 14, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2" />
    </IconWrapper>
  ),

  // 搜索
  Search: ({ size = 14, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <circle cx="11" cy="11" r="8" />
      <line x1="21" y1="21" x2="16.65" y2="16.65" />
    </IconWrapper>
  ),

  // 缩放
  ZoomIn: ({ size = 14, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <circle cx="11" cy="11" r="8" />
      <line x1="21" y1="21" x2="16.65" y2="16.65" />
      <line x1="11" y1="8" x2="11" y2="14" />
      <line x1="8" y1="11" x2="14" y2="11" />
    </IconWrapper>
  ),

  ZoomOut: ({ size = 14, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <circle cx="11" cy="11" r="8" />
      <line x1="21" y1="21" x2="16.65" y2="16.65" />
      <line x1="8" y1="11" x2="14" y2="11" />
    </IconWrapper>
  ),

  // 适合视图
  Maximize: ({ size = 14, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <path d="M8 3H5a2 2 0 0 0-2 2v3m18 0V5a2 2 0 0 0-2-2h-3m0 18h3a2 2 0 0 0 2-2v-3M3 16v3a2 2 0 0 0 2 2h3" />
    </IconWrapper>
  ),

  // 导出
  Download: ({ size = 14, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
      <polyline points="7 10 12 15 17 10" />
      <line x1="12" y1="15" x2="12" y2="3" />
    </IconWrapper>
  ),

  Image: ({ size = 14, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
      <circle cx="8.5" cy="8.5" r="1.5" />
      <polyline points="21 15 16 10 5 21" />
    </IconWrapper>
  ),

  // 箭头
  ChevronRight: ({ size = 12, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <polyline points="9 18 15 12 9 6" />
    </IconWrapper>
  ),

  ChevronDown: ({ size = 12, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <polyline points="6 9 12 15 18 9" />
    </IconWrapper>
  ),

  ArrowUp: ({ size = 12, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <line x1="12" y1="19" x2="12" y2="5" />
      <polyline points="5 12 12 5 19 12" />
    </IconWrapper>
  ),

  ArrowDown: ({ size = 12, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <line x1="12" y1="5" x2="12" y2="19" />
      <polyline points="19 12 12 19 5 12" />
    </IconWrapper>
  ),

  // X/关闭
  X: ({ size = 14, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <line x1="18" y1="6" x2="6" y2="18" />
      <line x1="6" y1="6" x2="18" y2="18" />
    </IconWrapper>
  ),

  // 设置
  Settings: ({ size = 14, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <circle cx="12" cy="12" r="3" />
      <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4" />
      <path d="M19.4 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3" />
      <path d="M9 4.6a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 1.82-.33H9" />
    </IconWrapper>
  ),

  // 位置标记
  MapPin: ({ size = 12, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <path d="M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0 1 18 0z" />
      <circle cx="12" cy="10" r="3" />
    </IconWrapper>
  ),

  // 信息
  Info: ({ size = 14, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <circle cx="12" cy="12" r="10" />
      <line x1="12" y1="16" x2="12" y2="12" />
      <line x1="12" y1="8" x2="12.01" y2="8" />
    </IconWrapper>
  ),

  // 刷新
  Refresh: ({ size = 14, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <polyline points="23 4 23 10 17 10" />
      <polyline points="1 20 1 14 7 14" />
      <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
    </IconWrapper>
  ),

  // 图表/可视化
  BarChart: ({ size = 14, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <line x1="12" y1="20" x2="12" y2="10" />
      <line x1="18" y1="20" x2="18" y2="4" />
      <line x1="6" y1="20" x2="6" y2="16" />
    </IconWrapper>
  ),

  // 代码
  Code: ({ size = 14, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <polyline points="16 18 22 12 16 6" />
      <polyline points="8 6 2 12 8 18" />
    </IconWrapper>
  ),

  // 复制
  Copy: ({ size = 14, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
      <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
    </IconWrapper>
  ),

  // 检查标记
  Check: ({ size = 14, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <polyline points="20 6 9 17 4 12" />
    </IconWrapper>
  ),

  // 锁定
  Lock: ({ size = 14, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
      <path d="M7 11V7a5 5 0 0 1 10 0v4" />
    </IconWrapper>
  ),

  // 警告
  Alert: ({ size = 14, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" />
      <line x1="12" y1="9" x2="12" y2="13" />
      <line x1="12" y1="17" x2="12.01" y2="17" />
    </IconWrapper>
  ),

  // 交叉引用
  Link: ({ size = 12, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" />
      <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" />
    </IconWrapper>
  ),

  // 空状态图标
  Empty: ({ size = 32, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <circle cx="12" cy="12" r="10" />
      <line x1="12" y1="8" x2="12" y2="12" />
      <line x1="12" y1="16" x2="12.01" y2="16" />
    </IconWrapper>
  ),

  // 项目/文件夹
  FolderProject: ({ size = 14, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
    </IconWrapper>
  ),

  // 文件
  File: ({ size = 14, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
      <polyline points="14 2 14 8 20 8" />
    </IconWrapper>
  ),

  // 退出箭头
  LogOut: ({ size = 14, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" />
      <polyline points="16 17 21 12 16 7" />
      <line x1="21" y1="12" x2="9" y2="12" />
    </IconWrapper>
  ),

  // 左箭头
  ChevronLeft: ({ size = 12, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <polyline points="15 18 9 12 15 6" />
    </IconWrapper>
  ),

  // 列表视图
  List: ({ size = 14, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <line x1="8" y1="6" x2="21" y2="6" />
      <line x1="8" y1="12" x2="21" y2="12" />
      <line x1="8" y1="18" x2="21" y2="18" />
      <line x1="3" y1="6" x2="3.01" y2="6" />
      <line x1="3" y1="12" x2="3.01" y2="12" />
      <line x1="3" y1="18" x2="3.01" y2="18" />
    </IconWrapper>
  ),

  // 网格视图
  Grid: ({ size = 14, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <rect x="3" y="3" width="7" height="7" />
      <rect x="14" y="3" width="7" height="7" />
      <rect x="14" y="14" width="7" height="7" />
      <rect x="3" y="14" width="7" height="7" />
    </IconWrapper>
  ),

  // 分割视图
  Split: ({ size = 14, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
      <line x1="12" y1="3" x2="12" y2="21" />
    </IconWrapper>
  ),

  // 闪电/快速
  Lightning: ({ size = 14, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2" />
    </IconWrapper>
  ),

  // 图表/统计
  Chart: ({ size = 14, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <path d="M21.21 15.89A10 10 0 1 1 8 2.83" />
      <path d="M22 12A10 10 0 0 0 12 2v10z" />
    </IconWrapper>
  ),

  // 大纲
  ListOrdered: ({ size = 14, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <line x1="10" y1="6" x2="21" y2="6" />
      <line x1="10" y1="12" x2="21" y2="12" />
      <line x1="10" y1="18" x2="21" y2="18" />
      <path d="M4 6h1v4" />
      <path d="M4 10h2" />
      <path d="M6 18H4c0-1 2-2 2-3s-1-1.5-2-1" />
    </IconWrapper>
  ),

  // 详情面板
  Panel: ({ size = 14, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
      <line x1="3" y1="9" x2="21" y2="9" />
      <line x1="9" y1="21" x2="9" y2="9" />
    </IconWrapper>
  ),

  // 节点
  Node: ({ size = 14, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <circle cx="12" cy="12" r="3" />
      <circle cx="19" cy="5" r="2" />
      <circle cx="5" cy="5" r="2" />
      <circle cx="5" cy="19" r="2" />
      <circle cx="19" cy="19" r="2" />
    </IconWrapper>
  ),

  // IR/LLVM
  Cpu: ({ size = 14, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <rect x="4" y="4" width="16" height="16" rx="2" ry="2" />
      <rect x="9" y="9" width="6" height="6" />
      <line x1="9" y1="1" x2="9" y2="4" />
      <line x1="15" y1="1" x2="15" y2="4" />
      <line x1="9" y1="20" x2="9" y2="23" />
      <line x1="15" y1="20" x2="15" y2="23" />
      <line x1="20" y1="9" x2="23" y2="9" />
      <line x1="20" y1="15" x2="23" y2="15" />
      <line x1="1" y1="9" x2="4" y2="9" />
      <line x1="1" y1="15" x2="4" y2="15" />
    </IconWrapper>
  ),

  // 播放/开始
  Play: ({ size = 14, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <polygon points="5 3 19 12 5 21 5 3" />
    </IconWrapper>
  ),

  // 人物/用户
  Users: ({ size = 14, className }: IconProps) => (
    <IconWrapper size={size} className={className}>
      <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2" />
      <circle cx="9" cy="7" r="4" />
      <path d="M23 21v-2a4 4 0 0 0-3-3.87" />
      <path d="M16 3.13a4 4 0 0 1 0 7.75" />
    </IconWrapper>
  ),
}

export default Icons
