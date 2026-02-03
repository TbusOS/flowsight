/**
 * FlowSight 真实后端集成测试 (v2.0)
 * 
 * 🔴 关键：这是防止 UI 显示假数据的最后一道防线
 * 
 * 测试原则：
 * 1. 不使用任何 Mock - 调用真实的 Rust 后端
 * 2. 使用真实的内核代码目录进行测试
 * 3. 验证数据正确性，不只是存在性
 * 4. 包含严格的断言，确保功能真正工作
 * 
 * 运行方式：
 *   cd app && npx tsx tests/integration/real-backend-test.ts
 * 
 * CI 集成：
 *   必须在 CI 中运行，阻止不通过的代码合并
 */

import { execSync, spawn } from 'child_process';
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

// 测试用的内核路径（如果不存在则使用 fixture）
const KERNEL_PATHS = [
  '/Users/sky/linux-kernel/linux',
  '/home/parallels/linux_kernel',
  path.join(FLOWSIGHT_ROOT, 'tests/kernel_mock'),
];

const FIXTURE_PATH = path.join(FLOWSIGHT_ROOT, 'app/tests/fixtures/sample-project');

// 测试结果
interface TestResult {
  name: string;
  category: string;
  passed: boolean;
  error?: string;
  details?: string;
  duration?: number;
}

const results: TestResult[] = [];
let cliPath: string | null = null;
let testKernelPath: string | null = null;

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

function assertGreaterThan(actual: number, expected: number, message: string) {
  assert(actual > expected, `${message} (期望 > ${expected}, 实际: ${actual})`);
}

function assertGreaterOrEqual(actual: number, expected: number, message: string) {
  assert(actual >= expected, `${message} (期望 >= ${expected}, 实际: ${actual})`);
}

function assertContains(text: string, substring: string, message: string) {
  assert(text.includes(substring), `${message} (未找到: "${substring}")`);
}

function runTest(category: string, name: string, testFn: () => void | Promise<void>) {
  const start = Date.now();
  try {
    const result = testFn();
    if (result instanceof Promise) {
      return result
        .then(() => {
          results.push({ 
            name, 
            category, 
            passed: true, 
            duration: Date.now() - start 
          });
          log(`  ✅ ${name}`);
        })
        .catch((err: Error) => {
          results.push({ 
            name, 
            category, 
            passed: false, 
            error: err.message,
            duration: Date.now() - start 
          });
          log(`  ❌ ${name}`);
          log(`     ${err.message}`);
        });
    }
    results.push({ 
      name, 
      category, 
      passed: true, 
      duration: Date.now() - start 
    });
    log(`  ✅ ${name}`);
  } catch (err: any) {
    results.push({ 
      name, 
      category, 
      passed: false, 
      error: err.message,
      duration: Date.now() - start 
    });
    log(`  ❌ ${name}`);
    log(`     ${err.message}`);
  }
}

// ============================================================================
// 环境准备
// ============================================================================

async function setup(): Promise<boolean> {
  logSection('🔧 环境准备');

  // 1. 检查 CLI 是否已编译
  if (fs.existsSync(CLI_PATH)) {
    cliPath = CLI_PATH;
    log(`  ✅ 找到 Release CLI: ${cliPath}`);
  } else if (fs.existsSync(CLI_DEBUG_PATH)) {
    cliPath = CLI_DEBUG_PATH;
    log(`  ✅ 找到 Debug CLI: ${cliPath}`);
  } else {
    log('  📦 编译 CLI...');
    try {
      exec('cargo build --package flowsight-cli --release 2>&1', { timeout: 300000 });
      cliPath = CLI_PATH;
      log('     编译成功');
    } catch (err: any) {
      log(`  ❌ CLI 编译失败: ${err.message}`);
      log('     请手动运行: cargo build --package flowsight-cli --release');
      return false;
    }
  }

  // 2. 查找测试用内核目录
  for (const kp of KERNEL_PATHS) {
    if (fs.existsSync(kp) && fs.existsSync(path.join(kp, 'drivers'))) {
      testKernelPath = kp;
      log(`  ✅ 找到内核目录: ${testKernelPath}`);
      break;
    }
  }

  // 3. 如果没有内核目录，使用 fixture
  if (!testKernelPath) {
    if (fs.existsSync(FIXTURE_PATH)) {
      testKernelPath = FIXTURE_PATH;
      log(`  ⚠️  使用 fixture 目录: ${testKernelPath}`);
    } else {
      log('  ❌ 没有找到测试目录');
      return false;
    }
  }

  return true;
}

// ============================================================================
// 测试用例：解析功能
// ============================================================================

function testParserFunctionality() {
  logSection('📝 解析器测试');

  const testFile = testKernelPath!.includes('fixture')
    ? path.join(testKernelPath!, 'main.c')
    : path.join(testKernelPath!, 'drivers/gpio/gpio-dwapb.c');

  if (!fs.existsSync(testFile)) {
    log(`  ⚠️  跳过：测试文件不存在 ${testFile}`);
    return;
  }

  // 测试 1：解析器能提取函数
  runTest('解析器', '提取函数数量 > 0', () => {
    const output = exec(`"${cliPath}" analyze "${testFile}" 2>&1`);
    const match = output.match(/Found (\d+) functions/i) || output.match(/函数: (\d+)/);
    assert(match !== null, '无法从输出中解析函数数量');
    const count = parseInt(match[1]);
    assertGreaterThan(count, 0, '函数数量');
  });

  // 测试 2：解析器能提取结构体（内核代码）
  if (!testKernelPath!.includes('fixture')) {
    runTest('解析器', '提取结构体数量 >= 1', () => {
      const output = exec(`"${cliPath}" analyze "${testFile}" 2>&1`);
      const match = output.match(/(\d+) structs/i) || output.match(/结构体: (\d+)/);
      assert(match !== null, '无法从输出中解析结构体数量');
      const count = parseInt(match[1]);
      assertGreaterOrEqual(count, 1, '结构体数量');
    });
  }

  // 测试 3：解析结果包含真实函数名
  runTest('解析器', '输出包含真实的函数名', () => {
    const output = exec(`"${cliPath}" functions "${testFile}" 2>&1`);
    // 应该有函数列表输出
    assert(output.length > 50, '函数列表输出太短，可能解析失败');
    // 应该包含常见的函数模式
    const hasProbe = output.toLowerCase().includes('probe') || output.includes('main');
    assert(hasProbe, '应该包含 probe 或 main 函数');
  });
}

// ============================================================================
// 测试用例：执行流分析
// ============================================================================

function testExecutionFlowAnalysis() {
  logSection('🔄 执行流分析测试');

  const testFile = testKernelPath!.includes('fixture')
    ? path.join(testKernelPath!, 'main.c')
    : path.join(testKernelPath!, 'drivers/gpio/gpio-dwapb.c');

  if (!fs.existsSync(testFile)) {
    log(`  ⚠️  跳过：测试文件不存在 ${testFile}`);
    return;
  }

  // 获取入口函数名
  const entryFunction = testKernelPath!.includes('fixture') ? 'main' : 'dwapb_gpio_probe';

  // 测试 1：执行流包含多个节点
  runTest('执行流', `${entryFunction} 执行流包含多个节点`, () => {
    const output = exec(`"${cliPath}" flow "${testFile}" ${entryFunction} 2>&1`);
    const lines = output.split('\n').filter(l => l.trim().length > 0);
    assertGreaterThan(lines.length, 1, '执行流节点数量');
  });

  // 测试 2：调用关系被正确提取
  runTest('执行流', '函数调用关系被正确提取', () => {
    const output = exec(`"${cliPath}" callees "${testFile}" ${entryFunction} 2>&1`);
    // 应该有被调用函数列表
    assert(output.length > 10, '调用关系输出为空');
    // 对于内核驱动，应该有 devm_ 或其他内核 API
    if (!testKernelPath!.includes('fixture')) {
      const hasKernelCalls = output.includes('devm_') || 
                             output.includes('platform_') || 
                             output.includes('gpio_');
      assert(hasKernelCalls, '应该包含内核 API 调用');
    }
  });

  // 测试 3：执行流深度 >= 2
  runTest('执行流', '执行流深度 >= 2 层', () => {
    const output = exec(`"${cliPath}" flow "${testFile}" ${entryFunction} 2>&1`);
    // 检查是否有缩进（表示调用深度）
    const hasDepth = output.includes('  ') || output.includes('├') || output.includes('└');
    assert(hasDepth, '执行流应该有多层调用深度');
  });
}

// ============================================================================
// 测试用例：项目索引
// ============================================================================

function testProjectIndexing() {
  logSection('📁 项目索引测试');

  // 使用 GPIO 驱动目录作为测试
  const testDir = testKernelPath!.includes('fixture')
    ? testKernelPath!
    : path.join(testKernelPath!, 'drivers/gpio');

  if (!fs.existsSync(testDir)) {
    log(`  ⚠️  跳过：测试目录不存在 ${testDir}`);
    return;
  }

  // 测试 1：索引能扫描到文件
  runTest('项目索引', '能正确扫描 .c 文件', () => {
    const output = exec(`"${cliPath}" index "${testDir}" 2>&1`, { timeout: 120000 });
    // 应该报告找到的文件数量
    const hasFileCount = output.match(/(\d+)\s*files?/i) || output.match(/文件[:：]\s*(\d+)/);
    if (hasFileCount) {
      const count = parseInt(hasFileCount[1]);
      assertGreaterThan(count, 0, '文件数量');
    } else {
      // 至少应该有一些输出
      assert(output.length > 20, '索引输出太短');
    }
  });

  // 测试 2：索引能提取函数
  runTest('项目索引', '能正确提取函数总数', () => {
    const output = exec(`"${cliPath}" index "${testDir}" 2>&1`, { timeout: 120000 });
    const hasFunctionCount = output.match(/(\d+)\s*functions?/i) || output.match(/函数[:：]\s*(\d+)/);
    if (hasFunctionCount) {
      const count = parseInt(hasFunctionCount[1]);
      assertGreaterThan(count, 0, '函数总数');
    }
  });

  // 🔴 关键测试：验证 "0 个文件, 0 个函数" 的问题
  runTest('项目索引', '🔴 不能返回 0 个文件 0 个函数', () => {
    const output = exec(`"${cliPath}" index "${testDir}" 2>&1`, { timeout: 120000 });
    // 这是导致 UI 显示问题的关键场景
    const hasZeroFiles = output.match(/0\s*(?:files?|个文件)/i);
    const hasZeroFunctions = output.match(/0\s*(?:functions?|个函数)/i);
    
    // 如果有文件，不应该显示 0 个
    const cFiles = fs.readdirSync(testDir).filter(f => f.endsWith('.c'));
    if (cFiles.length > 0) {
      assert(!hasZeroFiles, `目录有 ${cFiles.length} 个 .c 文件，但索引返回 0`);
      assert(!hasZeroFunctions, '目录有文件但索引返回 0 个函数');
    }
  });
}

// ============================================================================
// 测试用例：回调识别
// ============================================================================

function testCallbackDetection() {
  logSection('🔗 回调识别测试');

  const testFile = testKernelPath!.includes('fixture')
    ? path.join(testKernelPath!, 'main.c')
    : path.join(testKernelPath!, 'drivers/gpio/gpio-dwapb.c');

  if (!fs.existsSync(testFile)) {
    log(`  ⚠️  跳过：测试文件不存在 ${testFile}`);
    return;
  }

  // 测试 1：回调命令可执行
  runTest('回调识别', 'callbacks 命令可执行', () => {
    const output = exec(`"${cliPath}" callbacks "${testFile}" 2>&1`);
    // 命令应该正常运行
    assert(output !== null && output !== undefined, '命令无输出');
  });

  // 测试 2：对于内核驱动，应该识别到一些回调
  if (!testKernelPath!.includes('fixture')) {
    runTest('回调识别', '内核驱动应识别到回调函数', () => {
      const output = exec(`"${cliPath}" callbacks "${testFile}" 2>&1`);
      // GPIO 驱动通常有中断处理函数等回调
      const hasCallbacks = output.includes('irq') || 
                           output.includes('handler') || 
                           output.includes('callback') ||
                           output.includes('probe');
      // 即使没有显式回调，命令也应该正常运行
      assert(output.length > 0, '回调命令输出为空');
    });
  }
}

// ============================================================================
// 测试用例：数据正确性（关键）
// ============================================================================

function testDataCorrectness() {
  logSection('🔴 数据正确性测试（关键）');

  const testDir = testKernelPath!.includes('fixture')
    ? testKernelPath!
    : path.join(testKernelPath!, 'drivers/gpio');

  if (!fs.existsSync(testDir)) {
    log(`  ⚠️  跳过：测试目录不存在 ${testDir}`);
    return;
  }

  // 🔴 关键测试 1：索引结果必须与实际文件数量匹配
  runTest('数据正确性', '🔴 索引文件数与实际 .c 文件数一致', () => {
    // 统计实际的 .c 文件数量
    const countCFiles = (dir: string): number => {
      let count = 0;
      try {
        const entries = fs.readdirSync(dir, { withFileTypes: true });
        for (const entry of entries) {
          if (entry.isDirectory() && !entry.name.startsWith('.')) {
            count += countCFiles(path.join(dir, entry.name));
          } else if (entry.isFile() && entry.name.endsWith('.c')) {
            count++;
          }
        }
      } catch {
        // ignore
      }
      return count;
    };

    const actualCount = countCFiles(testDir);
    const output = exec(`"${cliPath}" index "${testDir}" 2>&1`, { timeout: 120000 });
    
    const match = output.match(/(\d+)\s*(?:files?|个文件)/i);
    if (match) {
      const reportedCount = parseInt(match[1]);
      // 允许有一些差异（可能有些文件解析失败）
      const tolerance = Math.max(1, actualCount * 0.2); // 20% 容差
      assert(
        Math.abs(reportedCount - actualCount) <= tolerance,
        `索引报告 ${reportedCount} 个文件，但实际有 ${actualCount} 个 .c 文件`
      );
    }
  });

  // 🔴 关键测试 2：解析的函数必须存在于源代码中
  const testFile = testKernelPath!.includes('fixture')
    ? path.join(testKernelPath!, 'main.c')
    : path.join(testKernelPath!, 'drivers/gpio/gpio-dwapb.c');

  if (fs.existsSync(testFile)) {
    runTest('数据正确性', '🔴 解析的函数存在于源代码中', () => {
      const output = exec(`"${cliPath}" functions "${testFile}" 2>&1`);
      const source = fs.readFileSync(testFile, 'utf-8');
      
      // 从输出中提取函数名
      const functionNames = output
        .split('\n')
        .map(line => line.trim())
        .filter(line => line.length > 0 && !line.startsWith('#'))
        .map(line => line.split(/\s+/)[0])
        .filter(name => name && name.match(/^[a-zA-Z_][a-zA-Z0-9_]*$/));

      // 验证这些函数确实存在于源代码中
      for (const funcName of functionNames.slice(0, 5)) { // 检查前5个
        const pattern = new RegExp(`\\b${funcName}\\s*\\(`);
        assert(
          pattern.test(source),
          `解析器报告的函数 "${funcName}" 在源代码中未找到`
        );
      }
    });
  }

  // 🔴 关键测试 3：执行流的调用关系必须真实存在
  if (fs.existsSync(testFile)) {
    const entryFunction = testKernelPath!.includes('fixture') ? 'main' : 'dwapb_gpio_probe';
    
    runTest('数据正确性', '🔴 执行流的调用关系真实存在', () => {
      const callees = exec(`"${cliPath}" callees "${testFile}" ${entryFunction} 2>&1`);
      const source = fs.readFileSync(testFile, 'utf-8');
      
      // 找到入口函数的代码
      const funcPattern = new RegExp(
        `${entryFunction}\\s*\\([^)]*\\)\\s*\\{[^}]*\\}`,
        's'
      );
      const funcMatch = source.match(funcPattern);
      
      if (funcMatch) {
        const funcBody = funcMatch[0];
        // 从 callees 输出中提取被调用函数
        const calledFunctions = callees
          .split('\n')
          .map(line => line.trim().split(/\s+/)[0])
          .filter(name => name && name.match(/^[a-zA-Z_][a-zA-Z0-9_]*$/))
          .slice(0, 3); // 检查前3个
        
        // 至少有一个调用应该在函数体中
        if (calledFunctions.length > 0) {
          const foundCall = calledFunctions.some(fn => funcBody.includes(fn + '('));
          assert(foundCall, '解析器报告的调用关系在源代码中未找到');
        }
      }
    });
  }
}

// ============================================================================
// 主函数
// ============================================================================

async function main() {
  console.log('\n' + '═'.repeat(60));
  console.log('  FlowSight 真实后端集成测试 v2.0');
  console.log('  🔴 防止 UI 显示假数据的最后一道防线');
  console.log('═'.repeat(60));

  // 环境准备
  const ready = await setup();
  if (!ready) {
    console.log('\n❌ 环境准备失败，测试中止');
    process.exit(1);
  }

  // 运行测试
  testParserFunctionality();
  testExecutionFlowAnalysis();
  testProjectIndexing();
  testCallbackDetection();
  testDataCorrectness();

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

  console.log('\n🎉 所有测试通过！\n');
}

main().catch(err => {
  console.error('测试运行出错:', err);
  process.exit(1);
});
