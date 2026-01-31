/**
 * 真实后端集成测试
 * 
 * 这个测试直接调用 Rust CLI，验证真实的解析和分析结果。
 * 不使用任何 Mock，测试真实的数据流。
 * 
 * 使用方法:
 *   npx tsx tests/integration/real-backend-test.ts
 */

import { execSync } from 'child_process';
import * as path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// 测试配置
const FLOWSIGHT_ROOT = path.resolve(__dirname, '../../..');
const CLI_PATH = path.join(FLOWSIGHT_ROOT, 'target/debug/flowsight');
const TEST_KERNEL_PATH = '/Users/sky/linux-kernel/linux';
const TEST_FILE = path.join(TEST_KERNEL_PATH, 'drivers/gpio/gpio-dwapb.c');

interface TestResult {
  name: string;
  passed: boolean;
  error?: string;
  details?: string;
}

const results: TestResult[] = [];

function runTest(name: string, testFn: () => void) {
  try {
    testFn();
    results.push({ name, passed: true });
    console.log(`✅ ${name}`);
  } catch (err) {
    const errorMsg = err instanceof Error ? err.message : String(err);
    results.push({ name, passed: false, error: errorMsg });
    console.log(`❌ ${name}`);
    console.log(`   Error: ${errorMsg}`);
  }
}

function exec(cmd: string): string {
  try {
    return execSync(cmd, { 
      cwd: FLOWSIGHT_ROOT, 
      encoding: 'utf-8',
      timeout: 30000 
    });
  } catch (err: any) {
    throw new Error(`Command failed: ${cmd}\n${err.stderr || err.message}`);
  }
}

function assert(condition: boolean, message: string) {
  if (!condition) {
    throw new Error(message);
  }
}

// ============================================================================
// 测试用例
// ============================================================================

console.log('\n🔬 FlowSight 真实后端集成测试\n');
console.log(`CLI: ${CLI_PATH}`);
console.log(`测试文件: ${TEST_FILE}\n`);

// 确保 CLI 已编译
console.log('📦 编译 CLI...');
try {
  exec('cargo build --package flowsight-cli 2>&1');
  console.log('   编译成功\n');
} catch (err) {
  console.error('❌ CLI 编译失败，请先运行: cargo build --package flowsight-cli');
  process.exit(1);
}

// 测试 1: 解析器能正确提取函数
runTest('解析器提取函数数量 >= 20', () => {
  const output = exec(`${CLI_PATH} analyze "${TEST_FILE}" 2>&1`);
  const match = output.match(/Found (\d+) functions/);
  assert(match !== null, '无法解析函数数量');
  const count = parseInt(match[1]);
  assert(count >= 20, `期望 >= 20 个函数，实际: ${count}`);
});

// 测试 2: 解析器能正确提取结构体
runTest('解析器提取结构体数量 >= 1', () => {
  const output = exec(`${CLI_PATH} analyze "${TEST_FILE}" 2>&1`);
  const match = output.match(/(\d+) structs/);
  assert(match !== null, '无法解析结构体数量');
  const count = parseInt(match[1]);
  assert(count >= 1, `期望 >= 1 个结构体，实际: ${count}`);
});

// 测试 3: 函数调用关系被正确提取
runTest('dwapb_gpio_probe 调用了 dwapb_gpio_add_port', () => {
  const output = exec(`${CLI_PATH} callees "${TEST_FILE}" dwapb_gpio_probe 2>&1`);
  assert(output.includes('dwapb_gpio_add_port'), 
    'dwapb_gpio_probe 应该调用 dwapb_gpio_add_port');
});

// 测试 4: 执行流能正确展开
runTest('执行流包含多个节点（不只是入口函数）', () => {
  const output = exec(`${CLI_PATH} flow "${TEST_FILE}" dwapb_gpio_probe 2>&1`);
  // 计算缩进行数（代表调用深度）
  const lines = output.split('\n').filter(l => l.trim().length > 0);
  assert(lines.length >= 5, `期望 >= 5 行输出，实际: ${lines.length}`);
  
  // 应该包含子函数
  assert(output.includes('dwapb_gpio_add_port') || output.includes('devm_'), 
    '执行流应该展开子函数调用');
});

// 测试 5: 外部函数被正确标记
runTest('外部函数被标记为 [External]', () => {
  const output = exec(`${CLI_PATH} callees "${TEST_FILE}" dwapb_gpio_probe 2>&1`);
  assert(output.includes('[External]'), '外部函数应该被标记为 [External]');
  assert(output.includes('devm_') || output.includes('platform_'), 
    '应该包含 devm_* 或 platform_* 外部函数');
});

// 测试 6: 回调函数识别命令可用
runTest('callbacks 命令可以执行', () => {
  // 这个命令应该能正常运行，即使没有识别到回调
  const output = exec(`${CLI_PATH} callbacks "${TEST_FILE}" 2>&1`);
  assert(output.includes('Callbacks in'), 'callbacks 命令应该正常输出');
});

// 测试 7: 验证 probe 函数的调用深度
runTest('probe 函数的调用链深度 >= 2', () => {
  const output = exec(`${CLI_PATH} flow "${TEST_FILE}" dwapb_gpio_probe 2>&1`);
  // 检查是否有多层缩进
  const hasDepth2 = output.split('\n').some(line => line.startsWith('    ') || line.includes('├──'));
  assert(hasDepth2, '执行流应该有 >= 2 层深度');
});

// ============================================================================
// 测试结果汇总
// ============================================================================

console.log('\n' + '='.repeat(60));
console.log('测试结果汇总');
console.log('='.repeat(60));

const passed = results.filter(r => r.passed).length;
const failed = results.filter(r => !r.passed).length;

console.log(`\n总计: ${results.length} 个测试`);
console.log(`通过: ${passed} ✅`);
console.log(`失败: ${failed} ❌`);

if (failed > 0) {
  console.log('\n失败的测试:');
  results.filter(r => !r.passed).forEach(r => {
    console.log(`  - ${r.name}`);
    console.log(`    ${r.error}`);
  });
  process.exit(1);
}

console.log('\n🎉 所有测试通过！');
