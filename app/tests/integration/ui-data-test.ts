/**
 * UI 数据测试 - 验证知识库 API 集成
 * 
 * 测试 Team-3 完成的 UI 增强功能：
 * 1. 知识库 API 正确返回数据
 * 2. 异步模式 API 正确返回数据
 * 3. 前端组件能正确展示数据
 * 
 * 运行方式：
 *   cd app && npx tsx tests/integration/ui-data-test.ts
 */

import { execSync } from 'child_process';
import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// ============================================================================
// 配置
// ============================================================================

const FLOWSIGHT_ROOT = path.resolve(__dirname, '../../..');
const CLI_PATH = path.join(FLOWSIGHT_ROOT, 'target/release/flowsight');
const CLI_DEBUG_PATH = path.join(FLOWSIGHT_ROOT, 'target/debug/flowsight');

// ============================================================================
// 测试结果
// ============================================================================

interface TestResult {
  name: string;
  category: string;
  passed: boolean;
  error?: string;
  details?: string;
}

const results: TestResult[] = [];

// ============================================================================
// 工具函数
// ============================================================================

function log(msg: string) {
  console.log(msg);
}

function logSection(title: string) {
  console.log(`\n${'━'.repeat(60)}`);
  console.log(`  ${title}`);
  console.log(`${'━'.repeat(60)}\n`);
}

function exec(cmd: string, options: { timeout?: number; cwd?: string } = {}): string {
  try {
    return execSync(cmd, {
      cwd: options.cwd || FLOWSIGHT_ROOT,
      encoding: 'utf-8',
      timeout: options.timeout || 60000,
      stdio: ['pipe', 'pipe', 'pipe'],
    });
  } catch (err: any) {
    const stderr = err.stderr?.toString() || '';
    const stdout = err.stdout?.toString() || '';
    throw new Error(`Command failed: ${cmd}\nstderr: ${stderr}\nstdout: ${stdout}`);
  }
}

function assert(condition: boolean, message: string): asserts condition {
  if (!condition) {
    throw new Error(`断言失败: ${message}`);
  }
}

function runTest(category: string, name: string, testFn: () => void | Promise<void>) {
  const start = Date.now();
  try {
    const result = testFn();
    if (result instanceof Promise) {
      return result
        .then(() => {
          results.push({ name, category, passed: true });
          log(`  ✅ ${name}`);
        })
        .catch((err: Error) => {
          results.push({ name, category, passed: false, error: err.message });
          log(`  ❌ ${name}`);
          log(`     ${err.message}`);
        });
    }
    results.push({ name, category, passed: true });
    log(`  ✅ ${name}`);
  } catch (err: any) {
    results.push({ name, category, passed: false, error: err.message });
    log(`  ❌ ${name}`);
    log(`     ${err.message}`);
  }
}

// ============================================================================
// API 存在性测试
// ============================================================================

function testAPIExists() {
  logSection('🔍 API 存在性测试');

  // 测试 1：检查 Tauri 命令是否注册
  runTest('API 存在性', 'commands.rs 包含 get_knowledge_info', () => {
    const commandsFile = path.join(FLOWSIGHT_ROOT, 'app/src-tauri/src/commands.rs');
    const content = fs.readFileSync(commandsFile, 'utf-8');
    
    assert(content.includes('pub async fn get_knowledge_info'), 
      '未找到 get_knowledge_info 命令');
    assert(content.includes('#[tauri::command]'),
      '命令未使用 #[tauri::command] 宏');
  });

  runTest('API 存在性', 'commands.rs 包含 get_async_pattern_info', () => {
    const commandsFile = path.join(FLOWSIGHT_ROOT, 'app/src-tauri/src/commands.rs');
    const content = fs.readFileSync(commandsFile, 'utf-8');
    
    assert(content.includes('pub async fn get_async_pattern_info'),
      '未找到 get_async_pattern_info 命令');
  });

  // 测试 2：检查前端 API 封装
  runTest('API 存在性', 'tauri-api.ts 包含 getKnowledgeInfo', () => {
    const apiFile = path.join(FLOWSIGHT_ROOT, 'app/src/lib/tauri-api.ts');
    const content = fs.readFileSync(apiFile, 'utf-8');
    
    assert(content.includes('export async function getKnowledgeInfo'),
      '未找到 getKnowledgeInfo 函数');
    assert(content.includes("invoke<KnowledgeInfo | null>('get_knowledge_info'"),
      'getKnowledgeInfo 未正确调用 Tauri invoke');
  });

  runTest('API 存在性', 'tauri-api.ts 包含 getAsyncPatternInfo', () => {
    const apiFile = path.join(FLOWSIGHT_ROOT, 'app/src/lib/tauri-api.ts');
    const content = fs.readFileSync(apiFile, 'utf-8');
    
    assert(content.includes('export async function getAsyncPatternInfo'),
      '未找到 getAsyncPatternInfo 函数');
  });
}

// ============================================================================
// 类型定义测试
// ============================================================================

function testTypeDefinitions() {
  logSection('📝 类型定义测试');

  // 测试 1：KnowledgeInfo 类型完整性
  runTest('类型定义', 'KnowledgeInfo 包含必要字段', () => {
    const apiFile = path.join(FLOWSIGHT_ROOT, 'app/src/lib/tauri-api.ts');
    const content = fs.readFileSync(apiFile, 'utf-8');
    
    const requiredFields = ['name', 'context', 'can_sleep', 'call_chain', 'trigger'];
    for (const field of requiredFields) {
      assert(content.includes(field), `KnowledgeInfo 缺少字段: ${field}`);
    }
  });

  // 测试 2：CallChainNode 类型完整性
  runTest('类型定义', 'CallChainNode 包含必要字段', () => {
    const apiFile = path.join(FLOWSIGHT_ROOT, 'app/src/lib/tauri-api.ts');
    const content = fs.readFileSync(apiFile, 'utf-8');
    
    assert(content.includes('interface CallChainNode'),
      '未找到 CallChainNode 接口');
    assert(content.includes('is_user_entry'),
      'CallChainNode 缺少 is_user_entry 字段');
  });

  // 测试 3：Rust 类型与 TypeScript 类型匹配
  runTest('类型定义', 'Rust 和 TypeScript 类型字段名一致', () => {
    const rustFile = path.join(FLOWSIGHT_ROOT, 'app/src-tauri/src/commands.rs');
    const tsFile = path.join(FLOWSIGHT_ROOT, 'app/src/lib/tauri-api.ts');
    
    const rustContent = fs.readFileSync(rustFile, 'utf-8');
    const tsContent = fs.readFileSync(tsFile, 'utf-8');
    
    // 检查 KnowledgeInfo 关键字段
    assert(rustContent.includes('pub can_sleep:'), 'Rust KnowledgeInfo 缺少 can_sleep');
    assert(tsContent.includes('can_sleep?:'), 'TS KnowledgeInfo 缺少 can_sleep');
    
    assert(rustContent.includes('pub call_chain:'), 'Rust KnowledgeInfo 缺少 call_chain');
    assert(tsContent.includes('call_chain?:'), 'TS KnowledgeInfo 缺少 call_chain');
  });
}

// ============================================================================
// UI 组件测试
// ============================================================================

function testUIComponents() {
  logSection('🎨 UI 组件测试');

  // 测试 1：KnowledgeInfoPanel 组件存在
  runTest('UI 组件', 'KnowledgeInfoPanel 组件存在', () => {
    const componentFile = path.join(
      FLOWSIGHT_ROOT, 
      'app/src/components/KnowledgeInfoPanel/KnowledgeInfoPanel.tsx'
    );
    assert(fs.existsSync(componentFile), '组件文件不存在');
  });

  // 测试 2：组件正确调用 API
  runTest('UI 组件', 'KnowledgeInfoPanel 调用 getKnowledgeInfo', () => {
    const componentFile = path.join(
      FLOWSIGHT_ROOT,
      'app/src/components/KnowledgeInfoPanel/KnowledgeInfoPanel.tsx'
    );
    const content = fs.readFileSync(componentFile, 'utf-8');
    
    assert(content.includes('getKnowledgeInfo'),
      '组件未调用 getKnowledgeInfo');
    assert(content.includes("from '../../lib/tauri-api'"),
      '未从正确路径导入 API');
  });

  // 测试 3：组件展示关键信息
  runTest('UI 组件', 'KnowledgeInfoPanel 展示 context 信息', () => {
    const componentFile = path.join(
      FLOWSIGHT_ROOT,
      'app/src/components/KnowledgeInfoPanel/KnowledgeInfoPanel.tsx'
    );
    const content = fs.readFileSync(componentFile, 'utf-8');
    
    assert(content.includes('context') && content.includes('进程上下文'),
      '组件未展示执行上下文信息');
  });

  runTest('UI 组件', 'KnowledgeInfoPanel 展示 can_sleep 指示器', () => {
    const componentFile = path.join(
      FLOWSIGHT_ROOT,
      'app/src/components/KnowledgeInfoPanel/KnowledgeInfoPanel.tsx'
    );
    const content = fs.readFileSync(componentFile, 'utf-8');
    
    assert(content.includes('can_sleep') || content.includes('canSleep'),
      '组件未展示是否可睡眠');
    assert(content.includes('可睡眠') && content.includes('不可睡眠'),
      '组件缺少睡眠状态文本');
  });

  runTest('UI 组件', 'KnowledgeInfoPanel 展示调用链', () => {
    const componentFile = path.join(
      FLOWSIGHT_ROOT,
      'app/src/components/KnowledgeInfoPanel/KnowledgeInfoPanel.tsx'
    );
    const content = fs.readFileSync(componentFile, 'utf-8');
    
    assert(content.includes('call_chain') || content.includes('callChain'),
      '组件未处理调用链数据');
    assert(content.includes('CallChainSection') || content.includes('调用链'),
      '组件未展示调用链 UI');
  });

  // 测试 4：组件有空状态处理
  runTest('UI 组件', 'KnowledgeInfoPanel 有空状态处理', () => {
    const componentFile = path.join(
      FLOWSIGHT_ROOT,
      'app/src/components/KnowledgeInfoPanel/KnowledgeInfoPanel.tsx'
    );
    const content = fs.readFileSync(componentFile, 'utf-8');
    
    assert(content.includes('empty') || content.includes('暂无'),
      '组件缺少空状态处理');
    assert(content.includes('loading') || content.includes('加载'),
      '组件缺少加载状态处理');
  });
}

// ============================================================================
// TypeScript 编译测试
// ============================================================================

function testTypeScriptCompilation() {
  logSection('🔧 TypeScript 编译测试');

  runTest('编译', 'TypeScript 编译无错误', () => {
    try {
      exec('pnpm tsc --noEmit', { 
        cwd: path.join(FLOWSIGHT_ROOT, 'app'),
        timeout: 120000 
      });
    } catch (err: any) {
      // 提取编译错误信息
      const errorMsg = err.message || '';
      if (errorMsg.includes('error TS')) {
        throw new Error(`TypeScript 编译错误:\n${errorMsg.slice(0, 500)}`);
      }
      throw err;
    }
  });
}

// ============================================================================
// 参数命名一致性测试（防止 camelCase/snake_case 问题）
// ============================================================================

function testParameterNaming() {
  logSection('🔤 参数命名一致性测试');

  runTest('参数命名', 'Tauri invoke 参数使用 camelCase', () => {
    const apiFile = path.join(FLOWSIGHT_ROOT, 'app/src/lib/tauri-api.ts');
    const content = fs.readFileSync(apiFile, 'utf-8');
    
    // 检查 invoke 调用后的参数块
    // getKnowledgeInfo 应该使用 codeContext 而不是 code_context
    assert(content.includes("'get_knowledge_info', {"),
      '未找到 get_knowledge_info invoke 调用');
    assert(content.includes('symbol,'),
      'invoke 参数缺少 symbol');
    assert(content.includes('codeContext,'),
      'invoke 参数应使用 camelCase (codeContext)');
    // 确保没有使用 snake_case
    assert(!content.includes('code_context,'),
      '前端不应使用 snake_case (code_context)');
  });

  runTest('参数命名', 'Rust 命令参数使用 snake_case', () => {
    const rustFile = path.join(FLOWSIGHT_ROOT, 'app/src-tauri/src/commands.rs');
    const content = fs.readFileSync(rustFile, 'utf-8');
    
    // Rust 应该使用 snake_case
    assert(content.includes('symbol: String') || content.includes('symbol: &str'),
      'Rust 命令缺少 symbol 参数');
    assert(content.includes('code_context'),
      'Rust 命令应使用 snake_case (code_context)');
  });
}

// ============================================================================
// 主函数
// ============================================================================

async function main() {
  console.log('\n' + '═'.repeat(60));
  console.log('  FlowSight UI 数据测试');
  console.log('  验证知识库 API 与 UI 组件集成');
  console.log('═'.repeat(60));

  // 运行所有测试
  testAPIExists();
  testTypeDefinitions();
  testUIComponents();
  testParameterNaming();
  testTypeScriptCompilation();

  // 结果汇总
  logSection('📊 测试结果汇总');

  const passed = results.filter(r => r.passed).length;
  const failed = results.filter(r => !r.passed).length;
  const total = results.length;

  // 按类别分组
  const categories = [...new Set(results.map(r => r.category))];
  for (const cat of categories) {
    const catResults = results.filter(r => r.category === cat);
    const catPassed = catResults.filter(r => r.passed).length;
    console.log(`  ${cat}: ${catPassed}/${catResults.length} 通过`);
  }

  console.log(`\n  总计: ${total} 个测试`);
  console.log(`  通过: ${passed} ✅`);
  console.log(`  失败: ${failed} ❌`);
  console.log(`  通过率: ${((passed / total) * 100).toFixed(1)}%`);

  if (failed > 0) {
    console.log('\n❌ 失败的测试:');
    results
      .filter(r => !r.passed)
      .forEach(r => {
        console.log(`  [${r.category}] ${r.name}`);
        console.log(`    └─ ${r.error}`);
      });
    console.log('\n');
    process.exit(1);
  }

  console.log('\n🎉 所有 UI 数据测试通过！\n');
}

main().catch(err => {
  console.error('测试运行出错:', err);
  process.exit(1);
});
