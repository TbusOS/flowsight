/**
 * Tauri 接口契约验证器
 * 
 * 用于确保前端调用与后端期望的参数格式一致。
 * 这是测试失效的根本解决方案。
 */

// 后端命令的接口契约定义 (从 Rust commands.rs 提取)
export interface TauriCommandContract {
  name: string;
  description: string;
  parameters: {
    name: string;
    type: string;
    required: boolean;
    naming: 'camelCase' | 'snake_case';
  }[];
  returnType: string;
}

// 从 Rust 后端提取的接口契约
// 重要：参数名必须与 Rust 后端完全一致
export const TAURI_CONTRACTS: TauriCommandContract[] = [
  {
    name: 'open_project',
    description: '打开项目目录',
    parameters: [
      { name: 'path', type: 'string', required: true, naming: 'snake_case' }
    ],
    returnType: 'ProjectInfo'
  },
  {
    name: 'list_directory',
    description: '列出目录内容',
    parameters: [
      { name: 'path', type: 'string', required: true, naming: 'snake_case' }
    ],
    returnType: 'FileNode[]'
  },
  {
    name: 'read_file',
    description: '读取文件内容',
    parameters: [
      { name: 'path', type: 'string', required: true, naming: 'snake_case' }
    ],
    returnType: 'string'
  },
  {
    name: 'write_file',
    description: '写入文件内容',
    parameters: [
      { name: 'path', type: 'string', required: true, naming: 'snake_case' },
      { name: 'content', type: 'string', required: true, naming: 'snake_case' }
    ],
    returnType: 'WriteResult'
  },
  {
    name: 'build_execution_flow',
    description: '构建执行流',
    parameters: [
      // ⚠️ Tauri 2.0 使用 camelCase 参数名进行反序列化
      { name: 'filePath', type: 'string', required: true, naming: 'camelCase' },
      { name: 'entryFunction', type: 'string', required: true, naming: 'camelCase' },
      { name: 'maxDepth', type: 'number', required: false, naming: 'camelCase' },
      { name: 'expandAsync', type: 'boolean', required: false, naming: 'camelCase' }
    ],
    returnType: 'ExecutionFlow'
  },
  {
    name: 'get_entry_points',
    description: '获取入口点',
    parameters: [
      { name: 'file_path', type: 'string', required: true, naming: 'snake_case' }
    ],
    returnType: 'EntryPointInfo[]'
  },
  {
    name: 'get_functions',
    description: '获取函数列表',
    parameters: [
      { name: 'file_path', type: 'string', required: true, naming: 'snake_case' }
    ],
    returnType: 'FunctionInfo[]'
  },
  {
    name: 'get_function_detail_from_file',
    description: '获取函数详情',
    parameters: [
      { name: 'filePath', type: 'string', required: true, naming: 'camelCase' },
      { name: 'functionName', type: 'string', required: true, naming: 'camelCase' }
    ],
    returnType: 'FunctionDetail'
  },
  {
    name: 'index_project_background',
    description: '后台索引项目',
    parameters: [
      { name: 'path', type: 'string', required: true, naming: 'snake_case' }
    ],
    returnType: 'void'
  }
];

// 事件契约定义
export interface TauriEventContract {
  name: string;
  description: string;
  payloadSchema: {
    phase?: string[];  // 允许的 phase 值
    fields: { name: string; type: string; required: boolean }[];
  };
}

export const TAURI_EVENTS: TauriEventContract[] = [
  {
    name: 'index-progress',
    description: '索引进度事件',
    payloadSchema: {
      phase: ['scanning', 'parsing', 'indexing', 'done', 'error'],  // ⚠️ 后端使用 'done' 不是 'complete'
      fields: [
        { name: 'phase', type: 'string', required: true },
        { name: 'current', type: 'number', required: true },
        { name: 'total', type: 'number', required: true },
        { name: 'message', type: 'string', required: true },
        { name: 'files', type: 'number', required: false },
        { name: 'functions', type: 'number', required: false },
        { name: 'structs', type: 'number', required: false }
      ]
    }
  }
];

/**
 * 验证 invoke 调用参数是否符合契约
 */
export function validateInvokeArgs(
  command: string,
  args: Record<string, unknown> | undefined
): { valid: boolean; errors: string[] } {
  const contract = TAURI_CONTRACTS.find(c => c.name === command);
  
  if (!contract) {
    return { valid: true, errors: [] }; // 未知命令，跳过验证
  }
  
  const errors: string[] = [];
  const providedKeys = args ? Object.keys(args) : [];
  
  // 检查必需参数
  for (const param of contract.parameters) {
    if (param.required) {
      if (!args || !(param.name in args)) {
        errors.push(
          `命令 '${command}' 缺少必需参数 '${param.name}'。` +
          `提供的参数: [${providedKeys.join(', ')}]`
        );
      }
    }
  }
  
  // 检查参数名是否正确（不接受别名）
  if (args) {
    const expectedNames = contract.parameters.map(p => p.name);
    for (const key of providedKeys) {
      if (!expectedNames.includes(key)) {
        // 检查是否是命名风格错误
        const snakeVersion = key.replace(/([A-Z])/g, '_$1').toLowerCase();
        const camelVersion = key.replace(/_([a-z])/g, (_, c) => c.toUpperCase());
        
        if (expectedNames.includes(snakeVersion)) {
          errors.push(
            `命令 '${command}' 的参数 '${key}' 应该是 '${snakeVersion}' (snake_case)`
          );
        } else if (expectedNames.includes(camelVersion)) {
          errors.push(
            `命令 '${command}' 的参数 '${key}' 应该是 '${camelVersion}' (camelCase)`
          );
        } else {
          errors.push(
            `命令 '${command}' 收到未知参数 '${key}'。` +
            `期望: [${expectedNames.join(', ')}]`
          );
        }
      }
    }
  }
  
  return {
    valid: errors.length === 0,
    errors
  };
}

/**
 * 验证事件 payload 是否符合契约
 */
export function validateEventPayload(
  eventName: string,
  payload: Record<string, unknown>
): { valid: boolean; errors: string[] } {
  const contract = TAURI_EVENTS.find(e => e.name === eventName);
  
  if (!contract) {
    return { valid: true, errors: [] };
  }
  
  const errors: string[] = [];
  
  // 检查 phase 值是否有效
  if (contract.payloadSchema.phase && 'phase' in payload) {
    const phase = payload.phase as string;
    if (!contract.payloadSchema.phase.includes(phase)) {
      errors.push(
        `事件 '${eventName}' 的 phase '${phase}' 无效。` +
        `有效值: [${contract.payloadSchema.phase.join(', ')}]`
      );
    }
  }
  
  // 检查必需字段
  for (const field of contract.payloadSchema.fields) {
    if (field.required && !(field.name in payload)) {
      errors.push(
        `事件 '${eventName}' 缺少必需字段 '${field.name}'`
      );
    }
  }
  
  return {
    valid: errors.length === 0,
    errors
  };
}

/**
 * 创建带契约验证的严格 Mock
 */
export function createStrictMock(
  originalMock: (cmd: string, args?: Record<string, unknown>) => Promise<unknown>
): (cmd: string, args?: Record<string, unknown>) => Promise<unknown> {
  return async (cmd: string, args?: Record<string, unknown>) => {
    // 验证参数
    const validation = validateInvokeArgs(cmd, args);
    
    if (!validation.valid) {
      const errorMsg = `[契约验证失败] ${validation.errors.join('; ')}`;
      console.error(errorMsg);
      throw new Error(errorMsg);
    }
    
    // 调用原始 mock
    return originalMock(cmd, args);
  };
}

/**
 * 测试报告：记录所有契约违规
 */
export class ContractViolationReport {
  private violations: Array<{
    type: 'invoke' | 'event';
    name: string;
    errors: string[];
    timestamp: Date;
  }> = [];
  
  addViolation(type: 'invoke' | 'event', name: string, errors: string[]) {
    this.violations.push({ type, name, errors, timestamp: new Date() });
  }
  
  hasViolations(): boolean {
    return this.violations.length > 0;
  }
  
  getReport(): string {
    if (this.violations.length === 0) {
      return '✅ 无契约违规';
    }
    
    let report = `❌ 发现 ${this.violations.length} 个契约违规:\n\n`;
    
    for (const v of this.violations) {
      report += `[${v.type.toUpperCase()}] ${v.name}\n`;
      for (const err of v.errors) {
        report += `  - ${err}\n`;
      }
      report += '\n';
    }
    
    return report;
  }
}
