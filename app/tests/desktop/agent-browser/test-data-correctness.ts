#!/usr/bin/env npx tsx
/**
 * FlowSight 数据正确性测试
 * 
 * 🔴 关键测试：防止 "0 个文件, 0 个函数, 0 个结构体" 问题
 * 
 * 这个测试专门针对之前发现的 bug：
 * - 打开真实的内核目录后，UI 显示全零统计
 * - 执行流视图只有一个 "probe" 节点
 * 
 * 运行方式:
 *   1. 启动 Tauri 应用:
 *      WEBKIT_INSPECTOR_HTTP_SERVER=127.0.0.1:9222 cargo tauri dev
 * 
 *   2. 运行测试:
 *      npx tsx app/tests/desktop/agent-browser/test-data-correctness.ts
 */

import { AgentBrowserClient, TestAssertions } from './utils.js';
import * as path from 'path';
import * as fs from 'fs';

// ============================================================================
// 配置
// ============================================================================

const CDP_PORT = parseInt(process.env.CDP_PORT || '9222');
const TIMEOUT = 60000;

// 测试用的内核路径 - GPIO 驱动目录
const TEST_PATHS = [
  '/Users/sky/linux-kernel/linux/drivers/gpio',
  '/home/parallels/linux_kernel/drivers/gpio',
  path.resolve(__dirname, '../../../tests/fixtures/sample-project'),
];

// ============================================================================
// 测试实现
// ============================================================================

async function runDataCorrectnessTests() {
  console.log('\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
  console.log('  FlowSight 数据正确性测试');
  console.log('  目标: 防止 "0 个文件, 0 个函数" 问题');
  console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');

  const client = new AgentBrowserClient({
    cdpPort: CDP_PORT,
    timeout: TIMEOUT,
  });

  const assert = new TestAssertions();
  let testPath: string | null = null;

  // 查找测试路径
  for (const p of TEST_PATHS) {
    if (fs.existsSync(p)) {
      testPath = p;
      break;
    }
  }

  if (!testPath) {
    console.log('❌ 错误: 未找到测试用的内核路径');
    console.log('   请确保以下路径之一存在:');
    TEST_PATHS.forEach(p => console.log(`   - ${p}`));
    process.exit(1);
  }

  console.log(`  测试路径: ${testPath}`);

  try {
    // 连接到 Tauri 应用
    console.log(`\n  连接到 CDP 端口 ${CDP_PORT}...`);
    await client.connect();
    console.log('  ✓ 连接成功\n');

    // ========================================================================
    // 测试 1: 获取当前状态
    // ========================================================================
    console.log('  [测试 1] 获取当前页面状态...');
    
    const initialSnapshot = await client.snapshot();
    console.log(`    快照长度: ${initialSnapshot.tree.length} 字符`);
    console.log(`    元素引用数: ${Object.keys(initialSnapshot.refs).length}`);

    // ========================================================================
    // 测试 2: 检查统计信息
    // ========================================================================
    console.log('\n  [测试 2] 检查统计信息...');
    
    const tree = initialSnapshot.tree;
    
    // 查找统计文本
    const statsPatterns = [
      /(\d+)\s*个文件/,
      /(\d+)\s*个函数/,
      /(\d+)\s*个结构体/,
      /(\d+)\s*files?/i,
      /(\d+)\s*functions?/i,
      /(\d+)\s*structs?/i,
    ];

    let foundStats: { type: string; value: number }[] = [];
    for (const pattern of statsPatterns) {
      const match = tree.match(pattern);
      if (match) {
        foundStats.push({
          type: pattern.source,
          value: parseInt(match[1]),
        });
      }
    }

    if (foundStats.length > 0) {
      console.log('    找到的统计信息:');
      for (const stat of foundStats) {
        console.log(`      - ${stat.type}: ${stat.value}`);
      }

      // 检查是否全是零
      const allZero = foundStats.every(s => s.value === 0);
      
      assert.assertTrue(
        !allZero,
        '统计信息不应该全是零'
      );
    } else {
      console.log('    未找到统计信息文本');
    }

    // ========================================================================
    // 测试 3: 检查执行流视图
    // ========================================================================
    console.log('\n  [测试 3] 检查执行流视图...');

    // 查找执行流相关内容
    const hasFlowView = tree.includes('执行流') || 
                        tree.includes('Execution Flow') ||
                        tree.includes('flow');

    if (hasFlowView) {
      console.log('    ✓ 检测到执行流视图');

      // 检查是否只有一个 probe 节点
      const probePattern = /probe\s*(?:00|0)?/gi;
      const probeMatches = tree.match(probePattern);
      
      // 查找其他节点
      const nodePatterns = [
        /platform_driver_register/i,
        /module_init/i,
        /\_\_init/i,
        /driver_register/i,
        /device_create/i,
        /gpio_/i,
        /irq_/i,
        /request_irq/i,
        /devm_/i,
      ];

      let foundNodes: string[] = [];
      for (const pattern of nodePatterns) {
        if (pattern.test(tree)) {
          foundNodes.push(pattern.source);
        }
      }

      if (foundNodes.length > 0) {
        console.log(`    ✓ 找到 ${foundNodes.length} 个内核函数节点`);
      } else if (probeMatches && probeMatches.length === 1) {
        console.log('    ⚠️ 警告: 只找到一个 probe 节点');
        assert.assertTrue(
          false,
          '执行流视图不应该只有一个 probe 节点'
        );
      }
    } else {
      console.log('    注意: 未检测到执行流视图');
    }

    // ========================================================================
    // 测试 4: 检查文件树
    // ========================================================================
    console.log('\n  [测试 4] 检查文件树...');

    // 查找 .c 和 .h 文件
    const cFilePattern = /[\w-]+\.c\b/g;
    const hFilePattern = /[\w-]+\.h\b/g;
    
    const cFiles = tree.match(cFilePattern) || [];
    const hFiles = tree.match(hFilePattern) || [];

    console.log(`    C 文件: ${cFiles.length} 个`);
    console.log(`    H 文件: ${hFiles.length} 个`);

    if (cFiles.length > 0 || hFiles.length > 0) {
      // 有文件显示时，统计不应该是零
      const statsZeroPattern = /0\s*个文件.*0\s*个函数.*0\s*个结构体/;
      const hasZeroStats = statsZeroPattern.test(tree);

      assert.assertTrue(
        !hasZeroStats,
        '显示了文件但统计为零，这是矛盾的'
      );
    }

    // ========================================================================
    // 测试 5: 执行 JavaScript 获取实际数据
    // ========================================================================
    console.log('\n  [测试 5] 获取应用内部状态...');

    try {
      // 尝试获取 React/Vue 状态或全局变量
      const appState = await client.evaluate(`
        (function() {
          // 尝试获取 window 上的状态
          if (window.__TAURI__) {
            return {
              hasTauri: true,
              // 检查是否有项目数据
              projectData: window.__PROJECT_DATA__ || null,
            };
          }
          return { hasTauri: false };
        })()
      `);
      
      console.log(`    Tauri 环境: ${appState.hasTauri ? '是' : '否'}`);
      
      if (appState.projectData) {
        console.log(`    项目数据已加载`);
      }
    } catch (err: any) {
      console.log(`    无法获取内部状态: ${err.message}`);
    }

    // ========================================================================
    // 测试结果汇总
    // ========================================================================
    console.log('\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
    console.log('  测试结果');
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');

    const failures = assert.getFailures();
    
    if (failures.length === 0) {
      console.log('  ✓ 所有数据正确性检查通过\n');
    } else {
      console.log(`  ✗ 发现 ${failures.length} 个问题:\n`);
      for (const failure of failures) {
        console.log(`    - ${failure.message}`);
      }
      console.log('');
    }

    // 关闭连接
    await client.close();

    // 退出码
    process.exit(failures.length > 0 ? 1 : 0);

  } catch (err: any) {
    console.log(`\n  ❌ 测试执行失败: ${err.message}`);
    
    if (err.message.includes('connect')) {
      console.log('\n  提示: 请确保 Tauri 应用已启动并启用远程调试:');
      console.log('    WEBKIT_INSPECTOR_HTTP_SERVER=127.0.0.1:9222 cargo tauri dev');
    }
    
    process.exit(1);
  }
}

// ============================================================================
// 主程序
// ============================================================================

runDataCorrectnessTests();
