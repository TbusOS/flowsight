#!/usr/bin/env npx tsx
/**
 * 测试质量检查脚本
 * 
 * 🔴 这是自动化测试质量门禁的核心
 * 
 * 功能：
 * 1. 检查测试文件是否包含正确性断言（不只是存在性）
 * 2. 检查 Mock 是否正确验证参数
 * 3. 统计测试覆盖范围
 * 4. 生成质量报告
 * 
 * 运行方式：
 *   npx tsx tests/scripts/quality-check.ts
 */

import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// ============================================================================
// 配置
// ============================================================================

const TEST_DIR = path.resolve(__dirname, '..');
const RULES = {
  // 禁止的弱断言模式
  WEAK_ASSERTIONS: [
    /expect\([^)]+\)\.toBeVisible\(\)/g,
    /expect\([^)]+\)\.toBeDefined\(\)/g,
    /expect\([^)]+\)\.toBeTruthy\(\)/g,
    /expect\([^)]+\)\.count\(\)\)\.toBeGreaterThan\(0\)/g, // 只检查 > 0 也算弱
  ],
  // 必须有的强断言模式
  STRONG_ASSERTIONS: [
    /expect\([^)]+\)\.toBeGreaterThan\([1-9]\d*\)/g, // > 1 以上
    /expect\([^)]+\)\.toContain\([^)]+\)/g,
    /expect\([^)]+\)\.toEqual\([^)]+\)/g,
    /expect\([^)]+\)\.toHaveText\([^)]+\)/g,
    /expect\([^)]+\)\.toHaveAttribute\([^)]+\)/g,
  ],
  // Mock 必须验证参数
  MOCK_VALIDATION: [
    /validateContract/g,
    /args\.filePath/g,
    /args\.entryFunction/g,
  ],
  // 禁止的测试模式
  FORBIDDEN_PATTERNS: [
    /test\.skip/g, // 跳过的测试
    /\.only\(/g,   // 只运行单个测试
  ],
};

// 质量门禁阈值
const THRESHOLDS = {
  MIN_STRONG_ASSERTIONS_PER_TEST: 1,
  MAX_WEAK_ASSERTIONS_RATIO: 0.5, // 弱断言不能超过 50%
  MIN_TEST_FILES: 5,
};

// ============================================================================
// 类型定义
// ============================================================================

interface TestFileAnalysis {
  file: string;
  testCount: number;
  weakAssertions: number;
  strongAssertions: number;
  hasMockValidation: boolean;
  forbiddenPatterns: string[];
  issues: string[];
}

interface QualityReport {
  totalFiles: number;
  totalTests: number;
  totalWeakAssertions: number;
  totalStrongAssertions: number;
  filesWithIssues: TestFileAnalysis[];
  passed: boolean;
  summary: string[];
}

// ============================================================================
// 分析函数
// ============================================================================

function analyzeTestFile(filePath: string): TestFileAnalysis {
  const content = fs.readFileSync(filePath, 'utf-8');
  const fileName = path.relative(TEST_DIR, filePath);

  const result: TestFileAnalysis = {
    file: fileName,
    testCount: 0,
    weakAssertions: 0,
    strongAssertions: 0,
    hasMockValidation: false,
    forbiddenPatterns: [],
    issues: [],
  };

  // 统计测试数量
  const testMatches = content.match(/\btest\s*\(/g) || [];
  result.testCount = testMatches.length;

  // 统计弱断言
  for (const pattern of RULES.WEAK_ASSERTIONS) {
    const matches = content.match(pattern) || [];
    result.weakAssertions += matches.length;
  }

  // 统计强断言
  for (const pattern of RULES.STRONG_ASSERTIONS) {
    const matches = content.match(pattern) || [];
    result.strongAssertions += matches.length;
  }

  // 检查 Mock 验证
  for (const pattern of RULES.MOCK_VALIDATION) {
    if (pattern.test(content)) {
      result.hasMockValidation = true;
      break;
    }
  }

  // 检查禁止的模式
  for (const pattern of RULES.FORBIDDEN_PATTERNS) {
    const matches = content.match(pattern);
    if (matches) {
      result.forbiddenPatterns.push(pattern.source);
    }
  }

  // 生成问题列表
  if (result.testCount > 0) {
    const ratio = result.weakAssertions / (result.weakAssertions + result.strongAssertions || 1);
    
    if (ratio > THRESHOLDS.MAX_WEAK_ASSERTIONS_RATIO) {
      result.issues.push(
        `弱断言比例过高: ${(ratio * 100).toFixed(0)}% (阈值: ${THRESHOLDS.MAX_WEAK_ASSERTIONS_RATIO * 100}%)`
      );
    }

    if (result.strongAssertions < result.testCount * THRESHOLDS.MIN_STRONG_ASSERTIONS_PER_TEST) {
      result.issues.push(
        `强断言不足: ${result.strongAssertions} (最少需要: ${result.testCount * THRESHOLDS.MIN_STRONG_ASSERTIONS_PER_TEST})`
      );
    }

    if (result.forbiddenPatterns.length > 0) {
      result.issues.push(`包含禁止的模式: ${result.forbiddenPatterns.join(', ')}`);
    }

    // 检查是否使用了 Mock 但没有参数验证
    const hasMock = content.includes('__TAURI__') || content.includes('Mock');
    if (hasMock && !result.hasMockValidation) {
      result.issues.push('使用了 Mock 但没有参数验证');
    }
  }

  return result;
}

function findTestFiles(dir: string): string[] {
  const files: string[] = [];

  function walk(currentDir: string) {
    const entries = fs.readdirSync(currentDir, { withFileTypes: true });
    for (const entry of entries) {
      const fullPath = path.join(currentDir, entry.name);
      if (entry.isDirectory() && !entry.name.includes('node_modules')) {
        walk(fullPath);
      } else if (
        entry.isFile() &&
        (entry.name.endsWith('.spec.ts') || entry.name.endsWith('.test.ts'))
      ) {
        files.push(fullPath);
      }
    }
  }

  walk(dir);
  return files;
}

function generateReport(analyses: TestFileAnalysis[]): QualityReport {
  const report: QualityReport = {
    totalFiles: analyses.length,
    totalTests: analyses.reduce((sum, a) => sum + a.testCount, 0),
    totalWeakAssertions: analyses.reduce((sum, a) => sum + a.weakAssertions, 0),
    totalStrongAssertions: analyses.reduce((sum, a) => sum + a.strongAssertions, 0),
    filesWithIssues: analyses.filter(a => a.issues.length > 0),
    passed: true,
    summary: [],
  };

  // 检查门禁条件
  if (report.totalFiles < THRESHOLDS.MIN_TEST_FILES) {
    report.passed = false;
    report.summary.push(`❌ 测试文件数量不足: ${report.totalFiles} (最少需要: ${THRESHOLDS.MIN_TEST_FILES})`);
  }

  if (report.filesWithIssues.length > 0) {
    report.passed = false;
    report.summary.push(`❌ ${report.filesWithIssues.length} 个测试文件存在质量问题`);
  }

  const overallRatio = report.totalWeakAssertions / 
    (report.totalWeakAssertions + report.totalStrongAssertions || 1);
  if (overallRatio > THRESHOLDS.MAX_WEAK_ASSERTIONS_RATIO) {
    report.passed = false;
    report.summary.push(
      `❌ 整体弱断言比例过高: ${(overallRatio * 100).toFixed(0)}%`
    );
  }

  if (report.passed) {
    report.summary.push('✅ 所有质量门禁检查通过');
  }

  return report;
}

// ============================================================================
// 主函数
// ============================================================================

function main() {
  console.log('\n' + '═'.repeat(60));
  console.log('  FlowSight 测试质量检查');
  console.log('═'.repeat(60) + '\n');

  // 查找测试文件
  const testFiles = findTestFiles(TEST_DIR);
  console.log(`📁 找到 ${testFiles.length} 个测试文件\n`);

  // 分析每个文件
  const analyses: TestFileAnalysis[] = [];
  for (const file of testFiles) {
    const analysis = analyzeTestFile(file);
    analyses.push(analysis);
    
    const status = analysis.issues.length === 0 ? '✅' : '⚠️';
    console.log(`${status} ${analysis.file}`);
    console.log(`   测试: ${analysis.testCount} | 强断言: ${analysis.strongAssertions} | 弱断言: ${analysis.weakAssertions}`);
    
    if (analysis.issues.length > 0) {
      for (const issue of analysis.issues) {
        console.log(`   ❌ ${issue}`);
      }
    }
    console.log();
  }

  // 生成报告
  const report = generateReport(analyses);

  // 输出汇总
  console.log('━'.repeat(60));
  console.log('  质量报告汇总');
  console.log('━'.repeat(60) + '\n');

  console.log(`总计测试文件: ${report.totalFiles}`);
  console.log(`总计测试用例: ${report.totalTests}`);
  console.log(`强断言总数: ${report.totalStrongAssertions}`);
  console.log(`弱断言总数: ${report.totalWeakAssertions}`);
  console.log(`问题文件数: ${report.filesWithIssues.length}\n`);

  for (const line of report.summary) {
    console.log(line);
  }

  // 详细问题列表
  if (report.filesWithIssues.length > 0) {
    console.log('\n问题详情:');
    for (const file of report.filesWithIssues) {
      console.log(`\n  ${file.file}:`);
      for (const issue of file.issues) {
        console.log(`    - ${issue}`);
      }
    }
  }

  console.log('\n');

  // 退出码
  if (!report.passed) {
    console.log('❌ 质量门禁检查失败，请修复上述问题\n');
    process.exit(1);
  }

  console.log('🎉 质量门禁检查通过！\n');
}

main();
