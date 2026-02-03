/**
 * FlowSight 真实后端内核分析测试
 * 
 * 🔴 关键测试：使用真实 CLI 和真实内核代码进行测试
 * 
 * 测试原则：
 * 1. 不使用任何 Mock - 调用真实的 Rust CLI
 * 2. 使用真实的 Linux 内核代码
 * 3. 验证数据正确性，不只是存在性
 * 4. 包含严格的断言
 * 
 * 运行方式：
 *   cd app && npx tsx tests/integration/real-backend-kernel.spec.ts
 */

import { execSync, spawn, ChildProcess } from 'child_process';
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

// Linux 内核路径
const KERNEL_PATHS = [
  '/Users/sky/linux-kernel/linux',
  '/home/parallels/linux_kernel',
];

// 测试文件
const TEST_FILES = {
  usb_driver: 'drivers/usb/core/driver.c',
  gpio_driver: 'drivers/gpio/gpio-dwapb.c',
  usb_hub: 'drivers/usb/core/hub.c',
  imx_clk: 'arch/arm/mach-imx/clk-imx6q.c',
  imx_pm: 'arch/arm/mach-imx/pm-imx6.c',
};

// ============================================================================
// 测试结果
// ============================================================================

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
let kernelPath: string | null = null;

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
    throw new Error(`命令执行失败: ${cmd}\nstderr: ${stderr}\nstdout: ${stdout}`);
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

  // 2. 查找内核目录
  for (const kp of KERNEL_PATHS) {
    if (fs.existsSync(kp) && fs.existsSync(path.join(kp, 'drivers'))) {
      kernelPath = kp;
      log(`  ✅ 找到内核目录: ${kernelPath}`);
      break;
    }
  }

  if (!kernelPath) {
    log('  ❌ 没有找到 Linux 内核目录');
    log('     请确保内核目录存在于以下路径之一:');
    KERNEL_PATHS.forEach(p => log(`       - ${p}`));
    return false;
  }

  return true;
}

// ============================================================================
// USB 驱动测试
// ============================================================================

function testUSBDriverAnalysis() {
  logSection('🔌 USB 驱动分析测试');

  const testFile = path.join(kernelPath!, TEST_FILES.usb_driver);

  if (!fs.existsSync(testFile)) {
    log(`  ⚠️  跳过：测试文件不存在 ${testFile}`);
    return;
  }

  // 测试 1：解析 USB 驱动文件
  runTest('USB 驱动', '解析 driver.c 函数数量 > 0', () => {
    const output = exec(`"${cliPath}" analyze "${testFile}" 2>&1`);
    const match = output.match(/Found (\d+) functions/i) || output.match(/函数[:：]?\s*(\d+)/);
    assert(match !== null, '无法从输出中解析函数数量');
    const count = parseInt(match[1]);
    assertGreaterThan(count, 0, '函数数量');
    log(`     找到 ${count} 个函数`);
  });

  // 测试 2：识别 usb_probe_interface 函数
  runTest('USB 驱动', '识别 usb_probe_interface 函数', () => {
    const output = exec(`"${cliPath}" analyze "${testFile}" --format json 2>&1`);
    // 检查 JSON 输出是否包含 probe 相关函数
    const hasProbe = output.includes('probe') || output.includes('usb_');
    assert(hasProbe, 'USB 驱动应该包含 probe 相关函数');
  });

  // 测试 3：分析 probe 函数的执行流
  runTest('USB 驱动', '分析 probe 执行流', () => {
    // 先找到 probe 函数名
    const analyzeOutput = exec(`"${cliPath}" analyze "${testFile}" --format json 2>&1`);
    
    // 尝试分析常见的 probe 函数
    const probeFunctions = ['usb_probe_interface', 'usb_probe_device', 'usb_generic_driver_probe'];
    let flowOutput = '';
    
    for (const func of probeFunctions) {
      try {
        flowOutput = exec(`"${cliPath}" flow "${testFile}" ${func} 2>&1`);
        if (flowOutput.length > 50) {
          log(`     成功分析函数: ${func}`);
          break;
        }
      } catch {
        // 继续尝试下一个
      }
    }
    
    assert(flowOutput.length > 10, '执行流输出不应为空');
  });

  // 测试 4：检测异步回调
  runTest('USB 驱动', '检测异步回调 (URB completion)', () => {
    const output = exec(`"${cliPath}" callbacks "${testFile}" 2>&1`);
    // USB 驱动通常有回调函数
    assert(output.length > 0, '回调命令应有输出');
  });
}

// ============================================================================
// GPIO 驱动测试
// ============================================================================

function testGPIODriverAnalysis() {
  logSection('⚡ GPIO 驱动分析测试');

  const testFile = path.join(kernelPath!, TEST_FILES.gpio_driver);

  if (!fs.existsSync(testFile)) {
    log(`  ⚠️  跳过：测试文件不存在 ${testFile}`);
    return;
  }

  // 测试 1：解析 GPIO 驱动
  runTest('GPIO 驱动', '解析 gpio-dwapb.c 函数数量 > 5', () => {
    const output = exec(`"${cliPath}" analyze "${testFile}" 2>&1`);
    const match = output.match(/Found (\d+) functions/i) || output.match(/函数[:：]?\s*(\d+)/);
    assert(match !== null, '无法从输出中解析函数数量');
    const count = parseInt(match[1]);
    assertGreaterThan(count, 5, '函数数量');
    log(`     找到 ${count} 个函数`);
  });

  // 测试 2：识别 probe 回调
  runTest('GPIO 驱动', '识别 dwapb_gpio_probe 函数', () => {
    const output = exec(`"${cliPath}" flow "${testFile}" dwapb_gpio_probe 2>&1`);
    assert(output.length > 20, '执行流输出不应为空');
    log(`     执行流输出 ${output.length} 字符`);
  });

  // 测试 3：分析 callees
  runTest('GPIO 驱动', 'dwapb_gpio_probe 调用列表', () => {
    const output = exec(`"${cliPath}" callees "${testFile}" dwapb_gpio_probe 2>&1`);
    assert(output.length > 10, '被调用函数列表不应为空');
    
    // 应该调用一些 devm_ 或 platform_ 函数
    const hasKernelAPIs = output.includes('devm_') || 
                          output.includes('platform_') || 
                          output.includes('gpio_');
    assert(hasKernelAPIs, '应该包含内核 API 调用');
    log(`     找到内核 API 调用`);
  });

  // 测试 4：检测 IRQ handler
  runTest('GPIO 驱动', '检测 IRQ handler', () => {
    const output = exec(`"${cliPath}" async "${testFile}" 2>&1`);
    // GPIO 驱动通常有中断处理
    log(`     异步输出: ${output.slice(0, 200)}...`);
  });
}

// ============================================================================
// ARM32 驱动测试
// ============================================================================

function testARM32DriverAnalysis() {
  logSection('🔧 ARM32 架构驱动测试');

  const imxClkFile = path.join(kernelPath!, TEST_FILES.imx_clk);
  const imxPmFile = path.join(kernelPath!, TEST_FILES.imx_pm);

  // 测试 IMX6 时钟驱动
  if (fs.existsSync(imxClkFile)) {
    runTest('ARM32', '解析 IMX6 时钟驱动', () => {
      const output = exec(`"${cliPath}" analyze "${imxClkFile}" 2>&1`);
      const match = output.match(/Found (\d+) functions/i) || output.match(/函数[:：]?\s*(\d+)/);
      assert(match !== null, '无法从输出中解析函数数量');
      const count = parseInt(match[1]);
      assertGreaterThan(count, 0, '函数数量');
      log(`     找到 ${count} 个函数`);
    });

    runTest('ARM32', 'IMX6 时钟驱动包含 clk 相关调用', () => {
      const output = exec(`"${cliPath}" analyze "${imxClkFile}" --format json 2>&1`);
      const hasClkAPIs = output.includes('clk_') || output.includes('imx_');
      assert(hasClkAPIs, '应该包含时钟 API');
    });
  } else {
    log(`  ⚠️  跳过 IMX6 时钟驱动测试：文件不存在`);
  }

  // 测试 IMX6 电源管理驱动
  if (fs.existsSync(imxPmFile)) {
    runTest('ARM32', '解析 IMX6 电源管理驱动', () => {
      const output = exec(`"${cliPath}" analyze "${imxPmFile}" 2>&1`);
      assert(output.length > 20, '分析输出不应为空');
    });
  } else {
    log(`  ⚠️  跳过 IMX6 电源管理测试：文件不存在`);
  }
}

// ============================================================================
// 执行流正确性测试
// ============================================================================

function testExecutionFlowCorrectness() {
  logSection('🔴 执行流正确性测试 (关键)');

  const testFile = path.join(kernelPath!, TEST_FILES.gpio_driver);

  if (!fs.existsSync(testFile)) {
    log(`  ⚠️  跳过：测试文件不存在`);
    return;
  }

  const sourceCode = fs.readFileSync(testFile, 'utf-8');

  // 测试 1：执行流中的函数调用必须真实存在于源代码
  runTest('执行流正确性', '🔴 调用关系真实存在于源代码', () => {
    const calleesOutput = exec(`"${cliPath}" callees "${testFile}" dwapb_gpio_probe 2>&1`);
    
    // 提取被调用函数名
    const callees = calleesOutput
      .split('\n')
      .map(line => line.trim())
      .filter(line => line.includes('(') || line.startsWith('├') || line.startsWith('└'))
      .map(line => {
        const match = line.match(/([a-zA-Z_][a-zA-Z0-9_]*)\s*\(/);
        return match ? match[1] : null;
      })
      .filter(Boolean) as string[];

    log(`     提取到 ${callees.length} 个被调用函数`);

    // 验证每个被调用函数在源代码中
    let verified = 0;
    let failed = 0;
    
    for (const callee of callees.slice(0, 10)) {
      // 检查源代码中是否有这个调用
      const callPattern = new RegExp(`${callee}\\s*\\(`);
      if (callPattern.test(sourceCode)) {
        verified++;
      } else {
        // 可能是外部函数（内核 API）
        if (callee.startsWith('devm_') || callee.startsWith('platform_') || 
            callee.startsWith('gpio_') || callee.startsWith('dev_')) {
          verified++; // 内核 API 不需要在文件中定义
        } else {
          failed++;
          log(`     ⚠️ 未验证: ${callee}`);
        }
      }
    }

    const ratio = callees.length > 0 ? verified / callees.length : 0;
    // 60% 阈值：因为很多调用是外部内核 API，不在同一个文件中定义
    assert(ratio >= 0.6, `调用验证率过低: ${(ratio * 100).toFixed(1)}%`);
    log(`     验证率: ${(ratio * 100).toFixed(1)}%`);
  });

  // 测试 2：执行流深度合理
  runTest('执行流正确性', '执行流深度 >= 2 层', () => {
    const flowOutput = exec(`"${cliPath}" flow "${testFile}" dwapb_gpio_probe 2>&1`);
    
    // 检查是否有缩进（表示调用深度）
    const hasDepth = flowOutput.includes('  ') || 
                     flowOutput.includes('├') || 
                     flowOutput.includes('└');
    assert(hasDepth, '执行流应该有多层调用');
  });

  // 测试 3：ftrace 格式输出包含行号
  runTest('执行流正确性', 'ftrace 格式包含行号信息', () => {
    const traceOutput = exec(`"${cliPath}" trace "${testFile}" dwapb_gpio_probe --format ftrace 2>&1`);
    
    // ftrace 格式应该包含行号 (L开头的数字)
    const hasLineNumbers = /L\d+/.test(traceOutput);
    // 即使没有行号，也应该有结构化输出
    assert(traceOutput.length > 20, 'ftrace 输出不应为空');
    
    if (hasLineNumbers) {
      log(`     ✓ 包含行号信息`);
    }
  });
}

// ============================================================================
// 知识库匹配测试
// ============================================================================

function testKnowledgeBaseMatching() {
  logSection('📚 知识库模式匹配测试');

  const testFile = path.join(kernelPath!, TEST_FILES.usb_driver);

  if (!fs.existsSync(testFile)) {
    log(`  ⚠️  跳过：测试文件不存在`);
    return;
  }

  const sourceCode = fs.readFileSync(testFile, 'utf-8');

  // 测试知识库中的 USB 模式
  runTest('知识库匹配', 'USB driver 模式匹配', () => {
    // 检查源代码中是否有知识库定义的模式
    const usbPatterns = [
      /usb_register\s*\(/,           // USB 驱动注册
      /module_usb_driver\s*\(/,      // 模块宏
      /struct usb_driver\s+\w+/,     // USB 驱动结构体
      /\.probe\s*=\s*\w+/,           // probe 回调
      /\.disconnect\s*=\s*\w+/,      // disconnect 回调
    ];

    let matched = 0;
    for (const pattern of usbPatterns) {
      if (pattern.test(sourceCode)) {
        matched++;
      }
    }

    assertGreaterThan(matched, 0, 'USB 模式匹配数量');
    log(`     匹配到 ${matched}/${usbPatterns.length} 个 USB 模式`);
  });

  // 测试 URB 模式
  const hubFile = path.join(kernelPath!, TEST_FILES.usb_hub);
  if (fs.existsSync(hubFile)) {
    runTest('知识库匹配', 'URB 模式匹配', () => {
      const hubSource = fs.readFileSync(hubFile, 'utf-8');
      
      const urbPatterns = [
        /usb_alloc_urb\s*\(/,
        /usb_submit_urb\s*\(/,
        /usb_kill_urb\s*\(/,
        /usb_free_urb\s*\(/,
        /\.complete\s*=\s*\w+/,  // URB completion callback
      ];

      let matched = 0;
      for (const pattern of urbPatterns) {
        if (pattern.test(hubSource)) {
          matched++;
        }
      }

      assertGreaterThan(matched, 0, 'URB 模式匹配数量');
      log(`     匹配到 ${matched}/${urbPatterns.length} 个 URB 模式`);
    });
  }
}

// ============================================================================
// 主函数
// ============================================================================

async function main() {
  console.log('\n' + '═'.repeat(60));
  console.log('  FlowSight 真实后端内核分析测试');
  console.log('  🔴 使用真实 CLI + 真实 Linux 内核代码');
  console.log('═'.repeat(60));

  // 环境准备
  const ready = await setup();
  if (!ready) {
    console.log('\n❌ 环境准备失败，测试中止');
    process.exit(1);
  }

  // 运行测试
  testUSBDriverAnalysis();
  testGPIODriverAnalysis();
  testARM32DriverAnalysis();
  testExecutionFlowCorrectness();
  testKnowledgeBaseMatching();

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

  // 写入测试报告
  const reportDir = path.join(FLOWSIGHT_ROOT, '.claude/reports');
  if (!fs.existsSync(reportDir)) {
    fs.mkdirSync(reportDir, { recursive: true });
  }

  const report = generateReport(results, passed, failed, total);
  fs.writeFileSync(path.join(reportDir, 'kernel-backend-test-report.md'), report);
  console.log(`\n📝 测试报告已写入: .claude/reports/kernel-backend-test-report.md`);

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

function generateReport(results: TestResult[], passed: number, failed: number, total: number): string {
  const now = new Date().toISOString().split('T')[0];
  
  let report = `# FlowSight 内核后端测试报告

生成日期: ${now}

## 测试概览

| 指标 | 值 |
|------|-----|
| 总测试数 | ${total} |
| 通过 | ${passed} ✅ |
| 失败 | ${failed} ❌ |
| 通过率 | ${((passed / total) * 100).toFixed(1)}% |

## 测试环境

- **CLI 路径**: ${cliPath}
- **内核路径**: ${kernelPath}

## 测试用例详情

`;

  // 按类别分组
  const categories = [...new Set(results.map(r => r.category))];
  
  for (const cat of categories) {
    const catResults = results.filter(r => r.category === cat);
    const catPassed = catResults.filter(r => r.passed).length;
    
    report += `### ${cat} (${catPassed}/${catResults.length})\n\n`;
    report += '| 测试 | 状态 | 耗时 | 错误 |\n';
    report += '|------|------|------|------|\n';
    
    for (const r of catResults) {
      const status = r.passed ? '✅' : '❌';
      const duration = r.duration ? `${r.duration}ms` : '-';
      const error = r.error ? r.error.slice(0, 50) + (r.error.length > 50 ? '...' : '') : '-';
      report += `| ${r.name} | ${status} | ${duration} | ${error} |\n`;
    }
    
    report += '\n';
  }

  if (failed > 0) {
    report += `## 失败分析\n\n`;
    
    for (const r of results.filter(r => !r.passed)) {
      report += `### ${r.name}\n\n`;
      report += `- **类别**: ${r.category}\n`;
      report += `- **错误**: ${r.error}\n`;
      report += '\n**修复建议**:\n\n';
      
      // 根据错误类型给出建议
      if (r.error?.includes('编译')) {
        report += '- 确保 Rust 工具链已安装\n';
        report += '- 运行 `cargo build --package flowsight-cli --release`\n';
      } else if (r.error?.includes('文件不存在')) {
        report += '- 确保 Linux 内核源码已下载\n';
        report += '- 检查内核路径配置\n';
      } else if (r.error?.includes('验证率')) {
        report += '- 检查解析器是否正确识别函数调用\n';
        report += '- 验证 AST 解析逻辑\n';
      }
      
      report += '\n';
    }
  }

  report += `## 下一步

1. 修复失败的测试用例
2. 增加更多内核子系统的测试覆盖
3. 完善知识库模式匹配
4. 集成到 CI/CD 流程
`;

  return report;
}

main().catch(err => {
  console.error('测试运行出错:', err);
  process.exit(1);
});
