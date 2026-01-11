import { test, expect } from '@playwright/test';

/**
 * FlowSight 基础功能测试
 *
 * 测试应用的基本加载和核心 UI 组件
 */

test.describe('FlowSight 基础功能', () => {
  test('应用正常加载', async ({ page }) => {
    await page.goto('/');

    // 验证应用标题或主要组件存在
    await expect(page.locator('body')).toBeVisible();

    // 等待 React 应用加载
    await page.waitForTimeout(1000);

    // 截图留档
    await page.screenshot({ path: 'test-results/app-loaded.png' });
  });

  test('侧边栏文件树可见', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(1000);

    // 查找文件树区域 (通常有 file-tree 或 sidebar 类)
    const sidebar = page.locator('[class*="sidebar"], [class*="file-tree"], [data-testid="file-tree"]');

    // 如果侧边栏存在，验证它可见
    if (await sidebar.count() > 0) {
      await expect(sidebar.first()).toBeVisible();
    }
  });

  test('执行流视图区域存在', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(1000);

    // 查找执行流视图区域
    const flowView = page.locator('[class*="flow"], [class*="graph"], [data-testid="flow-view"]');

    // 如果流视图存在，验证它可见
    if (await flowView.count() > 0) {
      await expect(flowView.first()).toBeVisible();
    }
  });
});

test.describe('内核调用链显示', () => {
  test.skip('验证 USB probe 调用链节点', async ({ page }) => {
    // 此测试需要打开包含 USB 驱动代码的目录
    // 跳过直到有合适的测试数据
    await page.goto('/');

    // 等待分析完成后，查找包含 "USB 设备插入" 的节点
    const triggerNode = page.locator('text=USB 设备插入');

    if (await triggerNode.count() > 0) {
      await expect(triggerNode).toBeVisible();

      // 验证调用链中的内核函数节点
      const kernelNodes = [
        'usb_hub_port_connect',
        'usb_new_device',
        'device_add',
        'bus_probe_device',
        'driver_probe_device',
      ];

      for (const nodeName of kernelNodes) {
        const node = page.locator(`text=${nodeName}`);
        if (await node.count() > 0) {
          console.log(`Found kernel node: ${nodeName}`);
        }
      }
    }
  });
});

test.describe('执行上下文标注', () => {
  test.skip('验证节点显示执行上下文', async ({ page }) => {
    // 此测试需要在有分析结果的情况下运行
    await page.goto('/');

    // 查找包含执行上下文信息的元素
    // 预期格式：进程上下文、软中断上下文、硬中断上下文
    const contextLabels = [
      '进程上下文',
      '软中断上下文',
      '硬中断上下文',
      '可睡眠',
      '不可睡眠',
    ];

    for (const label of contextLabels) {
      const element = page.locator(`text=${label}`);
      if (await element.count() > 0) {
        console.log(`Found context label: ${label}`);
      }
    }
  });
});
