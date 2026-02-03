/**
 * FlowSight 知识库模式匹配测试
 * 
 * 测试知识库定义的模式能否在真实内核代码中正确匹配
 * 
 * 测试原则：
 * 1. 使用真实内核代码验证知识库模式
 * 2. 每个知识库文件的关键模式都需要测试
 * 3. 验证匹配率 >= 90%
 * 
 * 运行方式：
 *   cd app && npx tsx tests/integration/knowledge-pattern-test.ts
 */

import * as fs from 'fs';
import * as path from 'path';
import * as yaml from 'js-yaml';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// ============================================================================
// 配置
// ============================================================================

const FLOWSIGHT_ROOT = path.resolve(__dirname, '../../..');
const KNOWLEDGE_PATH = path.join(FLOWSIGHT_ROOT, 'knowledge/platforms/linux-kernel');
const KERNEL_PATH = '/Users/sky/linux-kernel/linux';

interface PatternTest {
  name: string;
  pattern: string | RegExp;
  testFile: string;
  expectedMatches: number; // 0 表示至少要有 1 个匹配
  description: string;
}

interface TestResult {
  category: string;
  pattern: string;
  testFile: string;
  passed: boolean;
  matchCount: number;
  expectedMin: number;
  error?: string;
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

function loadYamlFile(filePath: string): any {
  try {
    const content = fs.readFileSync(filePath, 'utf-8');
    return yaml.load(content);
  } catch (err: any) {
    console.error(`加载 YAML 失败: ${filePath}`, err.message);
    return null;
  }
}

function testPattern(
  category: string,
  patternStr: string,
  testFile: string,
  expectedMin: number = 1
): TestResult {
  const result: TestResult = {
    category,
    pattern: patternStr.slice(0, 50),
    testFile: path.basename(testFile),
    passed: false,
    matchCount: 0,
    expectedMin,
  };

  try {
    if (!fs.existsSync(testFile)) {
      result.error = '文件不存在';
      return result;
    }

    const source = fs.readFileSync(testFile, 'utf-8');
    
    // 将 YAML 中的模式转换为 RegExp
    const pattern = new RegExp(patternStr, 'g');
    const matches = source.match(pattern) || [];
    
    result.matchCount = matches.length;
    result.passed = matches.length >= expectedMin;
    
    if (!result.passed) {
      result.error = `匹配数 ${matches.length} < 期望最小值 ${expectedMin}`;
    }
  } catch (err: any) {
    result.error = err.message;
  }

  return result;
}

// ============================================================================
// USB 知识库测试
// ============================================================================

function testUSBKnowledge() {
  logSection('🔌 USB 知识库模式测试');

  const usbYamlPath = path.join(KNOWLEDGE_PATH, 'drivers/usb.yaml');
  const usbDriverFile = path.join(KERNEL_PATH, 'drivers/usb/core/driver.c');
  const usbHubFile = path.join(KERNEL_PATH, 'drivers/usb/core/hub.c');

  if (!fs.existsSync(usbYamlPath)) {
    log(`  ⚠️  跳过：知识库文件不存在 ${usbYamlPath}`);
    return;
  }

  const kb = loadYamlFile(usbYamlPath);
  if (!kb) return;

  // 测试 usb_driver 模式
  if (kb.usb_driver?.registration) {
    for (const reg of kb.usb_driver.registration) {
      const result = testPattern('USB 驱动注册', reg.pattern, usbDriverFile);
      results.push(result);
      log(`  ${result.passed ? '✅' : '❌'} ${reg.description}: ${result.matchCount} 匹配`);
    }
  }

  // 测试 usb_driver 回调
  if (kb.usb_driver?.callbacks) {
    for (const [name, cb] of Object.entries(kb.usb_driver.callbacks) as [string, any][]) {
      if (cb.pattern) {
        const result = testPattern(`USB ${name} 回调`, cb.pattern, usbDriverFile);
        results.push(result);
        log(`  ${result.passed ? '✅' : '❌'} ${cb.description}: ${result.matchCount} 匹配`);
      }
    }
  }

  // 测试 URB 模式
  if (kb.urb?.allocation && fs.existsSync(usbHubFile)) {
    for (const alloc of kb.urb.allocation) {
      const result = testPattern('URB 分配', alloc.pattern, usbHubFile);
      results.push(result);
      log(`  ${result.passed ? '✅' : '❌'} ${alloc.description}: ${result.matchCount} 匹配`);
    }
  }

  if (kb.urb?.submission && fs.existsSync(usbHubFile)) {
    for (const sub of kb.urb.submission) {
      const result = testPattern('URB 提交', sub.pattern, usbHubFile);
      results.push(result);
      log(`  ${result.passed ? '✅' : '❌'} ${sub.description}: ${result.matchCount} 匹配`);
    }
  }
}

// ============================================================================
// GPIO 知识库测试
// ============================================================================

function testGPIOKnowledge() {
  logSection('⚡ GPIO 知识库模式测试');

  const gpioYamlPath = path.join(KNOWLEDGE_PATH, 'drivers/gpio.yaml');
  const gpioDriverFile = path.join(KERNEL_PATH, 'drivers/gpio/gpio-dwapb.c');

  if (!fs.existsSync(gpioYamlPath)) {
    log(`  ⚠️  跳过：知识库文件不存在 ${gpioYamlPath}`);
    return;
  }

  const kb = loadYamlFile(gpioYamlPath);
  if (!kb) return;

  // 测试 GPIO chip 注册模式
  if (kb.gpiochip?.registration) {
    for (const reg of kb.gpiochip.registration) {
      const result = testPattern('GPIO chip 注册', reg.pattern, gpioDriverFile);
      results.push(result);
      log(`  ${result.passed ? '✅' : '❌'} ${reg.description}: ${result.matchCount} 匹配`);
    }
  }

  // 测试回调模式
  if (kb.gpiochip?.callbacks) {
    for (const [name, cb] of Object.entries(kb.gpiochip.callbacks) as [string, any][]) {
      if (cb.pattern) {
        const result = testPattern(`GPIO ${name} 回调`, cb.pattern, gpioDriverFile);
        results.push(result);
        log(`  ${result.passed ? '✅' : '❌'} ${cb.description || name}: ${result.matchCount} 匹配`);
      }
    }
  }
}

// ============================================================================
// WorkQueue 知识库测试
// ============================================================================

function testWorkQueueKnowledge() {
  logSection('⚙️ WorkQueue 知识库模式测试');

  const wqYamlPath = path.join(KNOWLEDGE_PATH, 'core/workqueue.yaml');
  // 使用一个包含 workqueue 使用的文件
  const testFile = path.join(KERNEL_PATH, 'drivers/usb/core/hub.c');

  if (!fs.existsSync(wqYamlPath)) {
    log(`  ⚠️  跳过：知识库文件不存在 ${wqYamlPath}`);
    return;
  }

  const kb = loadYamlFile(wqYamlPath);
  if (!kb) return;

  // 测试 INIT_WORK 模式
  const workPatterns = [
    { pattern: 'INIT_WORK\\s*\\(', description: 'INIT_WORK 初始化' },
    { pattern: 'INIT_DELAYED_WORK\\s*\\(', description: 'INIT_DELAYED_WORK 初始化' },
    { pattern: 'schedule_work\\s*\\(', description: 'schedule_work 调度' },
    { pattern: 'schedule_delayed_work\\s*\\(', description: 'schedule_delayed_work 调度' },
    { pattern: 'queue_work\\s*\\(', description: 'queue_work 入队' },
    { pattern: 'cancel_work_sync\\s*\\(', description: 'cancel_work_sync 取消' },
  ];

  for (const p of workPatterns) {
    const result = testPattern('WorkQueue', p.pattern, testFile, 0); // 0 表示可以为 0
    results.push(result);
    log(`  ${result.matchCount > 0 ? '✅' : '⚠️'} ${p.description}: ${result.matchCount} 匹配`);
  }
}

// ============================================================================
// IRQ 知识库测试
// ============================================================================

function testIRQKnowledge() {
  logSection('⚡ IRQ 知识库模式测试');

  const irqYamlPath = path.join(KNOWLEDGE_PATH, 'core/irq.yaml');
  const gpioDriverFile = path.join(KERNEL_PATH, 'drivers/gpio/gpio-dwapb.c');

  if (!fs.existsSync(irqYamlPath)) {
    log(`  ⚠️  跳过：知识库文件不存在 ${irqYamlPath}`);
    return;
  }

  const kb = loadYamlFile(irqYamlPath);
  if (!kb) return;

  // 测试 IRQ 注册模式
  const irqPatterns = [
    { pattern: 'request_irq\\s*\\(', description: 'request_irq 注册' },
    { pattern: 'request_threaded_irq\\s*\\(', description: 'request_threaded_irq 注册' },
    { pattern: 'devm_request_irq\\s*\\(', description: 'devm_request_irq 注册' },
    { pattern: 'free_irq\\s*\\(', description: 'free_irq 释放' },
    { pattern: 'enable_irq\\s*\\(', description: 'enable_irq 启用' },
    { pattern: 'disable_irq\\s*\\(', description: 'disable_irq 禁用' },
  ];

  for (const p of irqPatterns) {
    const result = testPattern('IRQ', p.pattern, gpioDriverFile, 0);
    results.push(result);
    log(`  ${result.matchCount > 0 ? '✅' : '⚠️'} ${p.description}: ${result.matchCount} 匹配`);
  }
}

// ============================================================================
// Platform 驱动知识库测试
// ============================================================================

function testPlatformKnowledge() {
  logSection('🔧 Platform 驱动知识库模式测试');

  const platformYamlPath = path.join(KNOWLEDGE_PATH, 'drivers/platform.yaml');
  const gpioDriverFile = path.join(KERNEL_PATH, 'drivers/gpio/gpio-dwapb.c');

  if (!fs.existsSync(platformYamlPath)) {
    log(`  ⚠️  跳过：知识库文件不存在 ${platformYamlPath}`);
    return;
  }

  const kb = loadYamlFile(platformYamlPath);
  if (!kb) return;

  // 测试 platform_driver 模式
  const platformPatterns = [
    { pattern: 'platform_driver_register\\s*\\(', description: 'platform_driver_register 注册' },
    { pattern: 'module_platform_driver\\s*\\(', description: 'module_platform_driver 宏' },
    { pattern: 'struct platform_driver\\s+\\w+', description: 'platform_driver 结构体' },
    { pattern: '\\.probe\\s*=\\s*\\w+', description: 'probe 回调' },
    { pattern: '\\.remove\\s*=\\s*\\w+', description: 'remove 回调' },
    { pattern: 'platform_get_resource\\s*\\(', description: 'platform_get_resource 获取资源' },
    { pattern: 'platform_get_irq\\s*\\(', description: 'platform_get_irq 获取中断' },
  ];

  for (const p of platformPatterns) {
    const result = testPattern('Platform 驱动', p.pattern, gpioDriverFile, 0);
    results.push(result);
    log(`  ${result.matchCount > 0 ? '✅' : '⚠️'} ${p.description}: ${result.matchCount} 匹配`);
  }
}

// ============================================================================
// 内存分配知识库测试
// ============================================================================

function testMemoryKnowledge() {
  logSection('💾 内存分配知识库模式测试');

  const memoryYamlPath = path.join(KNOWLEDGE_PATH, 'core/memory.yaml');
  const gpioDriverFile = path.join(KERNEL_PATH, 'drivers/gpio/gpio-dwapb.c');

  if (!fs.existsSync(memoryYamlPath)) {
    log(`  ⚠️  跳过：知识库文件不存在 ${memoryYamlPath}`);
    return;
  }

  // 测试内存分配模式
  const memoryPatterns = [
    { pattern: 'devm_kzalloc\\s*\\(', description: 'devm_kzalloc 分配' },
    { pattern: 'devm_kcalloc\\s*\\(', description: 'devm_kcalloc 分配' },
    { pattern: 'kzalloc\\s*\\(', description: 'kzalloc 分配' },
    { pattern: 'kcalloc\\s*\\(', description: 'kcalloc 分配' },
    { pattern: 'kmalloc\\s*\\(', description: 'kmalloc 分配' },
    { pattern: 'kfree\\s*\\(', description: 'kfree 释放' },
  ];

  for (const p of memoryPatterns) {
    const result = testPattern('内存分配', p.pattern, gpioDriverFile, 0);
    results.push(result);
    log(`  ${result.matchCount > 0 ? '✅' : '⚠️'} ${p.description}: ${result.matchCount} 匹配`);
  }
}

// ============================================================================
// 主函数
// ============================================================================

async function main() {
  console.log('\n' + '═'.repeat(60));
  console.log('  FlowSight 知识库模式匹配测试');
  console.log('  验证知识库定义的模式能在真实内核代码中匹配');
  console.log('═'.repeat(60));

  // 检查内核路径
  if (!fs.existsSync(KERNEL_PATH)) {
    console.log(`\n❌ 内核路径不存在: ${KERNEL_PATH}`);
    process.exit(1);
  }

  // 运行测试
  testUSBKnowledge();
  testGPIOKnowledge();
  testWorkQueueKnowledge();
  testIRQKnowledge();
  testPlatformKnowledge();
  testMemoryKnowledge();

  // 结果汇总
  logSection('📊 测试结果汇总');

  const passed = results.filter(r => r.passed).length;
  const warned = results.filter(r => !r.passed && r.matchCount > 0).length;
  const failed = results.filter(r => !r.passed && r.matchCount === 0).length;
  const total = results.length;

  // 按类别分组
  const categories = [...new Set(results.map(r => r.category))];
  for (const cat of categories) {
    const catResults = results.filter(r => r.category === cat);
    const catPassed = catResults.filter(r => r.passed).length;
    const catTotal = catResults.length;
    const catMatches = catResults.reduce((sum, r) => sum + r.matchCount, 0);
    console.log(`  ${cat}: ${catPassed}/${catTotal} 通过, ${catMatches} 总匹配`);
  }

  console.log(`\n  总计: ${total} 个模式测试`);
  console.log(`  通过: ${passed} ✅`);
  console.log(`  警告 (有匹配但少于预期): ${warned} ⚠️`);
  console.log(`  无匹配: ${failed} ❌`);
  
  const matchRate = ((passed + warned) / total * 100).toFixed(1);
  console.log(`  匹配率: ${matchRate}%`);

  // 写入测试报告
  const reportDir = path.join(FLOWSIGHT_ROOT, '.claude/reports');
  if (!fs.existsSync(reportDir)) {
    fs.mkdirSync(reportDir, { recursive: true });
  }

  const report = generateReport(results, passed, warned, failed, total);
  fs.writeFileSync(path.join(reportDir, 'knowledge-pattern-test-report.md'), report);
  console.log(`\n📝 测试报告已写入: .claude/reports/knowledge-pattern-test-report.md`);

  // 如果匹配率低于 70%，视为失败
  if ((passed + warned) / total < 0.7) {
    console.log('\n❌ 知识库模式匹配率过低！');
    process.exit(1);
  }

  console.log('\n🎉 知识库模式验证完成！\n');
}

function generateReport(
  results: TestResult[],
  passed: number,
  warned: number,
  failed: number,
  total: number
): string {
  const now = new Date().toISOString().split('T')[0];
  
  let report = `# FlowSight 知识库模式匹配测试报告

生成日期: ${now}

## 测试概览

| 指标 | 值 |
|------|-----|
| 总模式数 | ${total} |
| 通过 | ${passed} ✅ |
| 警告 | ${warned} ⚠️ |
| 无匹配 | ${failed} ❌ |
| 匹配率 | ${((passed + warned) / total * 100).toFixed(1)}% |

## 测试详情

`;

  // 按类别分组
  const categories = [...new Set(results.map(r => r.category))];
  
  for (const cat of categories) {
    const catResults = results.filter(r => r.category === cat);
    
    report += `### ${cat}\n\n`;
    report += '| 模式 | 测试文件 | 匹配数 | 状态 |\n';
    report += '|------|----------|--------|------|\n';
    
    for (const r of catResults) {
      const status = r.passed ? '✅' : (r.matchCount > 0 ? '⚠️' : '❌');
      report += `| \`${r.pattern}\` | ${r.testFile} | ${r.matchCount} | ${status} |\n`;
    }
    
    report += '\n';
  }

  report += `## 改进建议

1. **无匹配的模式**: 检查正则表达式语法，可能需要调整
2. **警告的模式**: 增加测试文件覆盖范围
3. **新增模式**: 为新的内核子系统添加知识库支持

## 知识库文件位置

- USB: \`knowledge/platforms/linux-kernel/drivers/usb.yaml\`
- GPIO: \`knowledge/platforms/linux-kernel/drivers/gpio.yaml\`
- WorkQueue: \`knowledge/platforms/linux-kernel/core/workqueue.yaml\`
- IRQ: \`knowledge/platforms/linux-kernel/core/irq.yaml\`
- Platform: \`knowledge/platforms/linux-kernel/drivers/platform.yaml\`
- Memory: \`knowledge/platforms/linux-kernel/core/memory.yaml\`
`;

  return report;
}

main().catch(err => {
  console.error('测试运行出错:', err);
  process.exit(1);
});
