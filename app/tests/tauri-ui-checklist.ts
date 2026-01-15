/**
 * FlowSight Tauri桌面应用 UI测试
 * 使用 Playwright MCP 工具进行测试
 *
 * 测试前先启动: pnpm tauri dev
 * 确保应用运行在 http://localhost:5173
 */

const TEST_URL = 'http://localhost:5173';

/**
 * 测试清单 - 需要手动或通过MCP工具执行的测试
 */
const testChecklist = {
  app: [
    { name: '应用标题显示', check: 'header h1 包含 "FlowSight"' },
    { name: '状态栏显示', check: '底部状态栏显示文件路径和状态' },
  ],
  sidebar: [
    { name: '项目面板显示', check: '左侧面板显示项目名称' },
    { name: '文件树加载', check: '显示文件树结构' },
    { name: '索引进度条', check: '打开项目时显示索引进度' },
  ],
  editor: [
    { name: '代码编辑器', check: 'Monaco编辑器正常显示' },
    { name: '标签栏', check: '支持多标签页切换' },
    { name: '面包屑导航', check: '显示文件路径和当前函数' },
  ],
  flowView: [
    { name: '执行流视图', check: 'FlowView组件正常显示' },
    { name: '节点展开/折叠', check: '点击箭头展开/折叠子节点' },
    { name: '节点点击跳转', check: '点击节点跳转到对应代码行' },
    { name: '悬停显示tooltip', check: '鼠标悬停显示详细信息' },
    { name: '搜索功能', check: '输入函数名过滤节点' },
    { name: 'Kernel过滤', check: '隐藏Kernel API节点功能' },
  ],
  outline: [
    { name: '大纲面板', check: '右侧显示代码大纲' },
    { name: '搜索过滤', check: '按 / 键搜索函数' },
    { name: '键盘导航', check: '上下箭头选择, Enter跳转' },
  ],
  contextMenu: [
    { name: '右键菜单', check: '右键显示上下文菜单' },
    { name: '键盘导航', check: '上下箭头导航, Enter执行' },
    { name: 'ESC关闭', check: '按ESC关闭菜单' },
  ],
  interactions: [
    { name: '代码-图联动', check: '点击函数高亮执行流节点' },
    { name: '导航按钮', check: '后退/前进按钮可用' },
    { name: '视图切换', check: '代码/分屏/执行流视图切换' },
    { name: '设置面板', check: '可打开设置主题等' },
  ],
};

/**
 * 生成测试报告模板
 */
function generateReport() {
  console.log('\n' + '='.repeat(60));
  console.log('FlowSight Tauri 桌面应用 UI测试报告');
  console.log('='.repeat(60));
  console.log(`测试时间: ${new Date().toLocaleString()}`);
  console.log(`测试URL: ${TEST_URL}`);
  console.log('\n');

  for (const [category, tests] of Object.entries(testChecklist)) {
    console.log(`【${category.toUpperCase()}】`);
    tests.forEach(t => {
      console.log(`  [ ] ${t.name}`);
      console.log(`      → ${t.check}`);
    });
    console.log('');
  }

  console.log('='.repeat(60));
  console.log('使用 MCP 工具执行测试:');
  console.log('  1. browser_navigate - 导航到应用');
  console.log('  2. browser_snapshot - 获取页面快照');
  console.log('  3. browser_click - 点击元素');
  console.log('  4. browser_take_screenshot - 截图验证');
  console.log('  5. browser_verify_element_visible - 验证元素');
  console.log('='.repeat(60));
}

generateReport();
