/**
 * Agent Browser 测试工具库
 * 
 * 封装 agent-browser CLI 调用，提供 TypeScript 友好的 API
 */

import { execSync, spawn, ChildProcess } from 'child_process';

// ============================================================================
// 类型定义
// ============================================================================

export interface SnapshotRef {
  role: string;
  name?: string;
}

export interface SnapshotResult {
  tree: string;
  refs: Record<string, SnapshotRef>;
}

export interface TabInfo {
  index: number;
  url: string;
  title: string;
  active: boolean;
}

export interface AgentBrowserConfig {
  cdpPort?: number;
  cdpUrl?: string;
  headed?: boolean;
  timeout?: number;
  session?: string;
}

export interface TestAssertion {
  passed: boolean;
  message: string;
  actual?: any;
  expected?: any;
}

// ============================================================================
// Agent Browser 客户端
// ============================================================================

export class AgentBrowserClient {
  private config: Required<AgentBrowserConfig>;
  private connected: boolean = false;

  constructor(config: AgentBrowserConfig = {}) {
    this.config = {
      cdpPort: config.cdpPort ?? 9222,
      cdpUrl: config.cdpUrl ?? '',
      headed: config.headed ?? false,
      timeout: config.timeout ?? 30000,
      session: config.session ?? 'default',
    };
  }

  // --------------------------------------------------------------------------
  // 核心命令执行
  // --------------------------------------------------------------------------

  private exec(args: string[], options: { json?: boolean; timeout?: number } = {}): string {
    const sessionArg = `--session ${this.config.session}`;
    const jsonArg = options.json ? '--json' : '';
    const cmd = `agent-browser ${sessionArg} ${jsonArg} ${args.join(' ')}`.trim();
    
    try {
      const result = execSync(cmd, {
        encoding: 'utf-8',
        timeout: options.timeout ?? this.config.timeout,
        stdio: ['pipe', 'pipe', 'pipe'],
      });
      return result.trim();
    } catch (err: any) {
      const stderr = err.stderr?.toString() || '';
      const stdout = err.stdout?.toString() || '';
      throw new Error(`Agent Browser 命令失败: ${cmd}\nstderr: ${stderr}\nstdout: ${stdout}`);
    }
  }

  private execJson<T>(args: string[]): T {
    const result = this.exec(args, { json: true });
    try {
      const parsed = JSON.parse(result);
      if (!parsed.success) {
        throw new Error(parsed.error || 'Unknown error');
      }
      return parsed.data as T;
    } catch (err: any) {
      if (err.message.includes('Agent Browser 命令失败')) {
        throw err;
      }
      throw new Error(`JSON 解析失败: ${result}`);
    }
  }

  // --------------------------------------------------------------------------
  // 连接管理
  // --------------------------------------------------------------------------

  /**
   * 连接到 Tauri 应用的 WebView（通过 CDP）
   */
  async connect(): Promise<void> {
    const endpoint = this.config.cdpUrl || String(this.config.cdpPort);
    this.exec(['connect', endpoint]);
    this.connected = true;
  }

  /**
   * 检查是否已连接
   */
  isConnected(): boolean {
    return this.connected;
  }

  /**
   * 关闭连接
   */
  async close(): Promise<void> {
    if (this.connected) {
      try {
        this.exec(['close']);
      } catch {
        // 忽略关闭错误
      }
      this.connected = false;
    }
  }

  // --------------------------------------------------------------------------
  // 快照与元素定位
  // --------------------------------------------------------------------------

  /**
   * 获取页面快照（可访问性树 + 元素引用）
   */
  async snapshot(options: { interactive?: boolean; compact?: boolean; depth?: number } = {}): Promise<SnapshotResult> {
    const args = ['snapshot'];
    if (options.interactive) args.push('-i');
    if (options.compact) args.push('-c');
    if (options.depth) args.push('-d', String(options.depth));

    const result = this.execJson<{ snapshot: string; refs?: Record<string, SnapshotRef> }>(args);
    return {
      tree: result.snapshot,
      refs: result.refs || {},
    };
  }

  /**
   * 根据文本或角色查找元素引用
   */
  async findRef(options: { text?: string; role?: string; name?: string }): Promise<string | null> {
    const snapshot = await this.snapshot({ interactive: true });
    
    for (const [ref, data] of Object.entries(snapshot.refs)) {
      if (options.role && data.role !== options.role) continue;
      if (options.name && data.name !== options.name) continue;
      if (options.text && !data.name?.includes(options.text)) continue;
      return ref;
    }
    
    return null;
  }

  // --------------------------------------------------------------------------
  // 交互操作
  // --------------------------------------------------------------------------

  /**
   * 点击元素（支持 ref 和 CSS 选择器）
   */
  async click(selector: string): Promise<void> {
    this.exec(['click', selector]);
  }

  /**
   * 双击元素
   */
  async dblclick(selector: string): Promise<void> {
    this.exec(['dblclick', selector]);
  }

  /**
   * 填充输入框
   */
  async fill(selector: string, value: string): Promise<void> {
    this.exec(['fill', selector, `"${value}"`]);
  }

  /**
   * 输入文本（逐字符）
   */
  async type(selector: string, text: string): Promise<void> {
    this.exec(['type', selector, `"${text}"`]);
  }

  /**
   * 按键
   */
  async press(key: string): Promise<void> {
    this.exec(['press', key]);
  }

  /**
   * 悬停
   */
  async hover(selector: string): Promise<void> {
    this.exec(['hover', selector]);
  }

  /**
   * 滚动
   */
  async scroll(direction: 'up' | 'down' | 'left' | 'right', amount?: number): Promise<void> {
    const args = ['scroll', direction];
    if (amount) args.push(String(amount));
    this.exec(args);
  }

  // --------------------------------------------------------------------------
  // 等待
  // --------------------------------------------------------------------------

  /**
   * 等待元素出现
   */
  async waitFor(selector: string, timeout?: number): Promise<void> {
    const args = ['wait', selector];
    if (timeout) args.push('--timeout', String(timeout));
    this.exec(args, { timeout: timeout ?? this.config.timeout });
  }

  /**
   * 等待文本出现
   */
  async waitForText(text: string, timeout?: number): Promise<void> {
    this.exec(['wait', '--text', `"${text}"`], { timeout: timeout ?? this.config.timeout });
  }

  /**
   * 等待 URL 匹配
   */
  async waitForUrl(pattern: string, timeout?: number): Promise<void> {
    this.exec(['wait', '--url', `"${pattern}"`], { timeout: timeout ?? this.config.timeout });
  }

  /**
   * 等待指定毫秒
   */
  async waitMs(ms: number): Promise<void> {
    this.exec(['wait', String(ms)]);
  }

  // --------------------------------------------------------------------------
  // 获取信息
  // --------------------------------------------------------------------------

  /**
   * 获取元素文本
   */
  async getText(selector: string): Promise<string> {
    const result = this.execJson<{ text: string }>(['get', 'text', selector]);
    return result.text || '';
  }

  /**
   * 获取元素数量
   */
  async getCount(selector: string): Promise<number> {
    const result = this.execJson<{ count: number }>(['get', 'count', selector]);
    return result.count;
  }

  /**
   * 检查元素是否可见
   */
  async isVisible(selector: string): Promise<boolean> {
    const result = this.execJson<{ visible: boolean }>(['is', 'visible', selector]);
    return result.visible;
  }

  /**
   * 获取当前 URL
   */
  async getUrl(): Promise<string> {
    const result = this.execJson<{ url: string }>(['get', 'url']);
    return result.url;
  }

  /**
   * 获取页面标题
   */
  async getTitle(): Promise<string> {
    const result = this.execJson<{ title: string }>(['get', 'title']);
    return result.title;
  }

  // --------------------------------------------------------------------------
  // 截图与录制
  // --------------------------------------------------------------------------

  /**
   * 截图
   */
  async screenshot(path?: string, options: { fullPage?: boolean } = {}): Promise<string> {
    const args = ['screenshot'];
    if (path) args.push(path);
    if (options.fullPage) args.push('--full');
    
    const result = this.execJson<{ path: string }>(args);
    return result.path;
  }

  /**
   * 开始录制视频
   */
  async startRecording(path: string): Promise<void> {
    this.exec(['record', 'start', path]);
  }

  /**
   * 停止录制视频
   */
  async stopRecording(): Promise<{ path: string }> {
    const result = this.execJson<{ path: string }>(['record', 'stop']);
    return result;
  }

  // --------------------------------------------------------------------------
  // 标签页管理
  // --------------------------------------------------------------------------

  /**
   * 列出所有标签页
   */
  async listTabs(): Promise<TabInfo[]> {
    const result = this.execJson<{ tabs: TabInfo[] }>(['tab']);
    return result.tabs;
  }

  /**
   * 切换到指定标签页
   */
  async switchTab(index: number): Promise<void> {
    this.exec(['tab', String(index)]);
  }

  // --------------------------------------------------------------------------
  // 执行 JavaScript
  // --------------------------------------------------------------------------

  /**
   * 在页面中执行 JavaScript
   */
  async evaluate<T = any>(script: string): Promise<T> {
    const result = this.execJson<{ result: T }>(['eval', `"${script.replace(/"/g, '\\"')}"`]);
    return result.result;
  }

  // --------------------------------------------------------------------------
  // 控制台与错误
  // --------------------------------------------------------------------------

  /**
   * 获取控制台消息
   */
  async getConsoleMessages(): Promise<Array<{ type: string; text: string }>> {
    const result = this.execJson<{ messages: Array<{ type: string; text: string }> }>(['console']);
    return result.messages;
  }

  /**
   * 获取页面错误
   */
  async getPageErrors(): Promise<Array<{ message: string }>> {
    const result = this.execJson<{ errors: Array<{ message: string }> }>(['errors']);
    return result.errors;
  }
}

// ============================================================================
// 断言工具
// ============================================================================

export class TestAssertions {
  private failures: TestAssertion[] = [];

  /**
   * 断言条件为真
   */
  assertTrue(condition: boolean, message: string): TestAssertion {
    const assertion: TestAssertion = {
      passed: condition,
      message,
      actual: condition,
      expected: true,
    };
    if (!assertion.passed) this.failures.push(assertion);
    return assertion;
  }

  /**
   * 断言值相等
   */
  assertEqual<T>(actual: T, expected: T, message: string): TestAssertion {
    const passed = actual === expected;
    const assertion: TestAssertion = {
      passed,
      message,
      actual,
      expected,
    };
    if (!assertion.passed) this.failures.push(assertion);
    return assertion;
  }

  /**
   * 断言值大于
   */
  assertGreaterThan(actual: number, expected: number, message: string): TestAssertion {
    const passed = actual > expected;
    const assertion: TestAssertion = {
      passed,
      message: `${message} (期望 > ${expected}, 实际: ${actual})`,
      actual,
      expected,
    };
    if (!assertion.passed) this.failures.push(assertion);
    return assertion;
  }

  /**
   * 断言值大于等于
   */
  assertGreaterOrEqual(actual: number, expected: number, message: string): TestAssertion {
    const passed = actual >= expected;
    const assertion: TestAssertion = {
      passed,
      message: `${message} (期望 >= ${expected}, 实际: ${actual})`,
      actual,
      expected,
    };
    if (!assertion.passed) this.failures.push(assertion);
    return assertion;
  }

  /**
   * 断言包含
   */
  assertContains(haystack: string, needle: string, message: string): TestAssertion {
    const passed = haystack.includes(needle);
    const assertion: TestAssertion = {
      passed,
      message: `${message} (在 "${haystack.slice(0, 50)}..." 中查找 "${needle}")`,
      actual: haystack,
      expected: `包含 "${needle}"`,
    };
    if (!assertion.passed) this.failures.push(assertion);
    return assertion;
  }

  /**
   * 断言元素存在于快照中
   */
  assertRefExists(snapshot: SnapshotResult, refOrName: string, message: string): TestAssertion {
    let found = false;
    
    // 检查是否是 ref
    if (snapshot.refs[refOrName]) {
      found = true;
    } else {
      // 检查是否有匹配的 name
      for (const ref of Object.values(snapshot.refs)) {
        if (ref.name?.includes(refOrName)) {
          found = true;
          break;
        }
      }
    }

    const assertion: TestAssertion = {
      passed: found,
      message,
      actual: found ? '找到' : '未找到',
      expected: refOrName,
    };
    if (!assertion.passed) this.failures.push(assertion);
    return assertion;
  }

  /**
   * 获取所有失败的断言
   */
  getFailures(): TestAssertion[] {
    return this.failures;
  }

  /**
   * 检查是否全部通过
   */
  allPassed(): boolean {
    return this.failures.length === 0;
  }

  /**
   * 重置
   */
  reset(): void {
    this.failures = [];
  }
}

// ============================================================================
// 测试运行器
// ============================================================================

export interface TestCase {
  name: string;
  category?: string;
  fn: (client: AgentBrowserClient, assert: TestAssertions) => Promise<void>;
}

export interface TestRunnerOptions {
  cdpPort?: number;
  timeout?: number;
  stopOnFailure?: boolean;
  videoDir?: string;
}

export class TestRunner {
  private client: AgentBrowserClient;
  private options: TestRunnerOptions;
  private results: Array<{
    test: TestCase;
    passed: boolean;
    error?: string;
    duration: number;
    assertions: TestAssertion[];
  }> = [];

  constructor(options: TestRunnerOptions = {}) {
    this.options = {
      cdpPort: options.cdpPort ?? 9222,
      timeout: options.timeout ?? 30000,
      stopOnFailure: options.stopOnFailure ?? false,
      videoDir: options.videoDir,
    };
    this.client = new AgentBrowserClient({
      cdpPort: this.options.cdpPort,
      timeout: this.options.timeout,
    });
  }

  /**
   * 运行单个测试
   */
  async runTest(test: TestCase): Promise<boolean> {
    const assertions = new TestAssertions();
    const startTime = Date.now();
    let passed = false;
    let error: string | undefined;

    console.log(`\n  [TEST] ${test.name}`);

    // 开始录制（如果配置了视频目录）
    let videoPath: string | undefined;
    if (this.options.videoDir) {
      const sanitizedName = test.name.replace(/[^a-zA-Z0-9]/g, '_');
      videoPath = `${this.options.videoDir}/${sanitizedName}.webm`;
      try {
        await this.client.startRecording(videoPath);
      } catch {
        // 忽略录制启动错误
      }
    }

    try {
      await test.fn(this.client, assertions);
      passed = assertions.allPassed();
      
      if (!passed) {
        const failures = assertions.getFailures();
        error = failures.map(f => `    - ${f.message}`).join('\n');
      }
    } catch (err: any) {
      passed = false;
      error = err.message;
    }

    // 停止录制
    if (videoPath) {
      try {
        await this.client.stopRecording();
      } catch {
        // 忽略录制停止错误
      }
    }

    const duration = Date.now() - startTime;
    
    this.results.push({
      test,
      passed,
      error,
      duration,
      assertions: assertions.getFailures(),
    });

    const status = passed ? '✓ PASS' : '✗ FAIL';
    console.log(`  ${status} (${duration}ms)`);
    if (error) {
      console.log(`  ${error}`);
    }

    return passed;
  }

  /**
   * 运行所有测试
   */
  async runAll(tests: TestCase[]): Promise<void> {
    console.log('\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
    console.log('  Agent Browser 桌面应用测试');
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');

    // 连接到应用
    console.log(`  连接到 CDP 端口 ${this.options.cdpPort}...`);
    try {
      await this.client.connect();
      console.log('  ✓ 连接成功\n');
    } catch (err: any) {
      console.log(`  ✗ 连接失败: ${err.message}`);
      console.log('  请确保 Tauri 应用已启动并启用了远程调试。');
      process.exit(1);
    }

    // 运行测试
    for (const test of tests) {
      const passed = await this.runTest(test);
      
      if (!passed && this.options.stopOnFailure) {
        console.log('\n  停止执行: stopOnFailure 已启用');
        break;
      }
    }

    // 关闭连接
    await this.client.close();

    // 打印汇总
    this.printSummary();
  }

  /**
   * 打印测试汇总
   */
  private printSummary(): void {
    const passed = this.results.filter(r => r.passed).length;
    const failed = this.results.filter(r => !r.passed).length;
    const total = this.results.length;
    const totalDuration = this.results.reduce((sum, r) => sum + r.duration, 0);

    console.log('\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
    console.log('  测试结果汇总');
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');
    
    console.log(`  总计: ${total} 个测试`);
    console.log(`  通过: ${passed} ✓`);
    console.log(`  失败: ${failed} ✗`);
    console.log(`  耗时: ${totalDuration}ms`);

    if (failed > 0) {
      console.log('\n  失败的测试:');
      for (const result of this.results.filter(r => !r.passed)) {
        console.log(`    - ${result.test.name}`);
        if (result.error) {
          console.log(`      ${result.error}`);
        }
      }
    }

    console.log('\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');

    // 设置退出码
    if (failed > 0) {
      process.exit(1);
    }
  }

  /**
   * 获取测试结果
   */
  getResults() {
    return this.results;
  }
}

// ============================================================================
// 导出
// ============================================================================

export default {
  AgentBrowserClient,
  TestAssertions,
  TestRunner,
};
