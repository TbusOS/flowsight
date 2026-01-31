/**
 * Tauri API 包装器
 * 
 * 在真实 Tauri 环境中使用原生 API，在测试环境中使用 Mock。
 * 这允许 Playwright 测试在浏览器环境中运行。
 */

// 类型定义
export interface FileNode {
  name: string;
  path: string;
  is_dir: boolean;
  extension?: string;
  children?: FileNode[];
}

export interface ProjectInfo {
  path: string;
  files_count: number;
  functions_count: number;
  structs_count: number;
  indexed: boolean;
}

export interface FunctionInfo {
  name: string;
  return_type: string;
  line: number;
  is_callback: boolean;
  callback_context?: string;
}

export interface DialogOptions {
  directory?: boolean;
  multiple?: boolean;
  title?: string;
  filters?: Array<{ name: string; extensions: string[] }>;
}

// 检测是否在 Tauri 环境中
export function isTauriEnv(): boolean {
  return typeof window !== 'undefined' && 
         (window as any).__TAURI_INTERNALS__ !== undefined;
}

// 检测是否在 Mock 环境中
export function isMockEnv(): boolean {
  return typeof window !== 'undefined' && 
         (window as any).__TAURI__ !== undefined &&
         !(window as any).__TAURI_INTERNALS__?.ipc;
}

/**
 * 调用 Tauri 命令
 */
export async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  // 优先使用 Mock（测试环境）
  if (typeof window !== 'undefined' && (window as any).__TAURI__?.core?.invoke) {
    console.log('[TauriAPI] Using mock invoke:', cmd);
    return (window as any).__TAURI__.core.invoke(cmd, args);
  }
  
  // 使用真实 Tauri API
  if (typeof window !== 'undefined' && (window as any).__TAURI_INTERNALS__) {
    console.log('[TauriAPI] Using real invoke:', cmd);
    const { invoke: tauriInvoke } = await import('@tauri-apps/api/core');
    return tauriInvoke<T>(cmd, args);
  }
  
  // 非 Tauri 环境
  console.warn('[TauriAPI] Not in Tauri environment, command not executed:', cmd);
  throw new Error(`Not in Tauri environment: ${cmd}`);
}

/**
 * 打开文件/目录选择对话框
 */
export async function openDialog(options?: DialogOptions): Promise<string | string[] | null> {
  // 优先使用 Mock（测试环境）
  if (typeof window !== 'undefined' && (window as any).__TAURI__?.dialog?.open) {
    console.log('[TauriAPI] Using mock dialog.open');
    return (window as any).__TAURI__.dialog.open(options);
  }
  
  // 使用真实 Tauri API
  if (typeof window !== 'undefined' && (window as any).__TAURI_INTERNALS__) {
    console.log('[TauriAPI] Using real dialog.open');
    const { open } = await import('@tauri-apps/plugin-dialog');
    return open(options);
  }
  
  // 非 Tauri 环境
  console.warn('[TauriAPI] Not in Tauri environment, dialog not available');
  return null;
}

/**
 * 保存文件对话框
 */
export async function saveDialog(options?: {
  title?: string;
  defaultPath?: string;
  filters?: Array<{ name: string; extensions: string[] }>;
}): Promise<string | null> {
  // 优先使用 Mock（测试环境）
  if (typeof window !== 'undefined' && (window as any).__TAURI__?.dialog?.save) {
    console.log('[TauriAPI] Using mock dialog.save');
    return (window as any).__TAURI__.dialog.save(options);
  }
  
  // 使用真实 Tauri API
  if (typeof window !== 'undefined' && (window as any).__TAURI_INTERNALS__) {
    console.log('[TauriAPI] Using real dialog.save');
    const { save } = await import('@tauri-apps/plugin-dialog');
    return save(options);
  }
  
  // 非 Tauri 环境
  console.warn('[TauriAPI] Not in Tauri environment, dialog not available');
  return null;
}

/**
 * Tauri 事件结构
 */
export interface TauriEvent<T> {
  payload: T;
  event: string;
  id: number;
}

/**
 * 监听 Tauri 事件
 */
export async function listen<T>(
  event: string,
  handler: (event: TauriEvent<T>) => void
): Promise<() => void> {
  // 优先使用 Mock（测试环境）
  if (typeof window !== 'undefined' && (window as any).__TAURI__?.event?.listen) {
    console.log('[TauriAPI] Using mock event.listen:', event);
    // Mock 环境下包装 handler 以匹配 Tauri 事件结构
    return (window as any).__TAURI__.event.listen(event, (payload: T) => {
      handler({ payload, event, id: Date.now() });
    });
  }
  
  // 使用真实 Tauri API
  if (typeof window !== 'undefined' && (window as any).__TAURI_INTERNALS__) {
    console.log('[TauriAPI] Using real event.listen:', event);
    const { listen: tauriListen } = await import('@tauri-apps/api/event');
    return tauriListen(event, handler as any);
  }
  
  // 非 Tauri 环境
  console.warn('[TauriAPI] Not in Tauri environment, events not available');
  return () => {};
}

/**
 * 发送 Tauri 事件
 */
export async function emit(event: string, payload?: unknown): Promise<void> {
  // 优先使用 Mock（测试环境）
  if (typeof window !== 'undefined' && (window as any).__TAURI__?.event?.emit) {
    console.log('[TauriAPI] Using mock event.emit:', event);
    return (window as any).__TAURI__.event.emit(event, payload);
  }
  
  // 使用真实 Tauri API
  if (typeof window !== 'undefined' && (window as any).__TAURI_INTERNALS__) {
    console.log('[TauriAPI] Using real event.emit:', event);
    const { emit: tauriEmit } = await import('@tauri-apps/api/event');
    return tauriEmit(event, payload);
  }
  
  // 非 Tauri 环境
  console.warn('[TauriAPI] Not in Tauri environment, events not available');
}
