/**
 * NodeDetailPanel Tests
 *
 * 测试节点详情面板的渲染、交互和边界情况
 */

import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { NodeDetailPanel, NodeDetailData } from '../NodeDetailPanel'
import type { AsyncMechanism } from '../../../types'

// 异步机制工厂函数
const createAsyncMechanism = (overrides: Partial<AsyncMechanism> = {}): AsyncMechanism => ({
  WorkQueue: { work_struct: 'my_work', queue: 'system_wq' },
  ...overrides,
} as AsyncMechanism)

// 类型扩展
type TestAsyncCallback = { AsyncCallback: { mechanism: AsyncMechanism } }

// 使用联合类型
type TestNodeType = 'Function' | 'EntryPoint' | 'KernelApi' | 'External' | TestAsyncCallback

interface TestNodeDetailData extends Omit<NodeDetailData, 'nodeType'> {
  nodeType: TestNodeType
}

describe('NodeDetailPanel', () => {
  // 基础测试数据
  const createTestNode = (overrides: Partial<TestNodeDetailData> = {}): TestNodeDetailData => ({
    name: 'test_function',
    nodeType: 'Function',
    location: {
      file: '/kernel/drivers/usb/core/hub.c',
      line: 1234,
      column: 10,
    },
    description: '测试函数描述',
    confidence: {
      level: 'Certain',
      reason: '通过静态分析确定',
    },
    functionSignature: 'int test_function(struct device *dev)',
    params: [
      { name: 'dev', type: 'struct device *' },
      { name: 'id', type: 'int' },
    ],
    returnType: 'int',
    callers: ['caller1', 'caller2'],
    children: [
      {
        id: 'child1',
        name: 'child_function',
        display_name: 'child_function',
        location: { file: '/kernel/drivers/usb/core/hub.c', line: 100, column: 5 },
        node_type: 'Function',
        children: [],
      },
    ],
    llvmIr: [
      'define i32 @test_function(i32 %arg1, i32 %arg2) {',
      'entry:',
      '  %result = add i32 %arg1, %arg2',
      '  ret i32 %result',
      '}',
    ],
    ...overrides,
  })

  describe('渲染状态', () => {
    it('渲染空状态 when node is null', () => {
      render(<NodeDetailPanel node={null} />)

      expect(screen.getByText('未选中节点')).toBeInTheDocument()
      expect(screen.getByText('点击流程图中的节点查看详情')).toBeInTheDocument()
    })

    it('渲染加载状态 when loading is true', () => {
      render(<NodeDetailPanel node={null} loading />)

      expect(screen.getByText('加载节点详情...')).toBeInTheDocument()
    })

    it('渲染节点信息 when node is provided', () => {
      const node = createTestNode()
      render(<NodeDetailPanel node={node} />)

      expect(screen.getByText('test_function()')).toBeInTheDocument()
      expect(screen.getByText('基本信息')).toBeInTheDocument()
    })
  })

  describe('节点类型显示', () => {
    it('显示函数节点类型', () => {
      const node = createTestNode({ nodeType: 'Function' })
      render(<NodeDetailPanel node={node} />)

      // 验证函数类型正确显示（通过面板头部）
      expect(screen.getByText('函数')).toBeInTheDocument()
    })

    it('显示入口点节点类型', () => {
      const node = createTestNode({ nodeType: 'EntryPoint' })
      render(<NodeDetailPanel node={node} />)

      expect(screen.getByText('入口点')).toBeInTheDocument()
    })

    it('显示内核 API 节点类型', () => {
      const node = createTestNode({ nodeType: 'KernelApi' })
      render(<NodeDetailPanel node={node} />)

      expect(screen.getByText('内核 API')).toBeInTheDocument()
    })

    it('显示外部函数节点类型', () => {
      const node = createTestNode({ nodeType: 'External' })
      render(<NodeDetailPanel node={node} />)

      expect(screen.getByText('外部函数')).toBeInTheDocument()
    })

    it('显示异步回调节点类型', () => {
      const node = createTestNode({
        nodeType: { AsyncCallback: { mechanism: createAsyncMechanism() } },
      })
      render(<NodeDetailPanel node={node} />)

      expect(screen.getByText('异步回调')).toBeInTheDocument()
      expect(screen.getByText('工作队列')).toBeInTheDocument()
    })
  })

  describe('异步机制详情', () => {
    it('显示工作队列异步详情', () => {
      const node = createTestNode({
        nodeType: { AsyncCallback: { mechanism: { WorkQueue: { work_struct: 'usb_work', queue: 'usb_wq' } } } },
      })
      render(<NodeDetailPanel node={node} />)

      expect(screen.getByText('工作队列')).toBeInTheDocument()
      expect(screen.getByText('usb_work')).toBeInTheDocument()
    })

    it('显示定时器异步详情', () => {
      const node = createTestNode({
        nodeType: { AsyncCallback: { mechanism: { Timer: { timer_name: 'hrtimer', timer_type: 'HRTIMER_NORMAL' } } } },
      })
      render(<NodeDetailPanel node={node} />)

      expect(screen.getByText('⏱️')).toBeInTheDocument()
      expect(screen.getByText('定时器')).toBeInTheDocument()
      expect(screen.getByText('hrtimer')).toBeInTheDocument()
    })

    it('显示硬中断异步详情', () => {
      const node = createTestNode({
        nodeType: { AsyncCallback: { mechanism: { Irq: { irq_name: 'usb_irq', flags: 'IRQF_SHARED' } } } },
      })
      render(<NodeDetailPanel node={node} />)

      expect(screen.getByText('硬中断')).toBeInTheDocument()
      expect(screen.getByText('usb_irq')).toBeInTheDocument()
    })

    it('显示内核线程异步详情', () => {
      const node = createTestNode({
        nodeType: { AsyncCallback: { mechanism: { Kthread: { kthread_name: 'kworker' } } } },
      })
      render(<NodeDetailPanel node={node} />)

      expect(screen.getByText('内核线程')).toBeInTheDocument()
      expect(screen.getByText('kworker')).toBeInTheDocument()
    })
  })

  describe('展开/折叠功能', () => {
    it('基本信息 section 默认展开', () => {
      const node = createTestNode()
      render(<NodeDetailPanel node={node} />)

      expect(screen.getByText('返回类型')).toBeInTheDocument()
      expect(screen.getByText('位置')).toBeInTheDocument()
    })

    it('可以折叠基本信息 section', () => {
      const node = createTestNode()
      render(<NodeDetailPanel node={node} />)

      const sectionHeader = screen.getByText('基本信息').closest('.section-header') as HTMLElement
      fireEvent.click(sectionHeader)

      expect(screen.queryByText('返回类型')).not.toBeInTheDocument()
    })

    it('可以折叠 LLVM IR section', () => {
      const node = createTestNode()
      render(<NodeDetailPanel node={node} />)

      const sectionHeader = screen.getByText('LLVM IR').closest('.section-header') as HTMLElement
      fireEvent.click(sectionHeader)

      expect(screen.queryByText('define i32')).not.toBeInTheDocument()
    })
  })

  describe('复制功能', () => {
    it('显示复制按钮', () => {
      const node = createTestNode()
      render(<NodeDetailPanel node={node} />)

      const copyBtn = screen.getByTitle('复制')
      expect(copyBtn).toBeInTheDocument()
    })

    it('点击复制按钮调用 clipboard API', async () => {
      const node = createTestNode()
      const writeTextSpy = vi.spyOn(navigator.clipboard, 'writeText')

      render(<NodeDetailPanel node={node} />)

      const copyBtn = screen.getByTitle('复制')
      fireEvent.click(copyBtn)

      await waitFor(() => {
        expect(writeTextSpy).toHaveBeenCalledWith(
          expect.stringContaining('define i32 @test_function')
        )
      })

      writeTextSpy.mockRestore()
    })

    it('复制后显示成功状态', async () => {
      const node = createTestNode()
      render(<NodeDetailPanel node={node} />)

      const copyBtn = screen.getByTitle('复制')
      fireEvent.click(copyBtn)

      await waitFor(() => {
        expect(screen.getByText('✓')).toBeInTheDocument()
      })
    })
  })

  describe('置信度显示', () => {
    it('显示确定置信度', () => {
      const node = createTestNode({
        confidence: { level: 'Certain', reason: '测试原因' },
      })
      render(<NodeDetailPanel node={node} />)

      expect(screen.getByText('✓')).toBeInTheDocument()
      expect(screen.getByText('确定')).toBeInTheDocument()
    })

    it('显示可能置信度', () => {
      const node = createTestNode({
        confidence: { level: 'Possible', reason: '测试原因' },
      })
      render(<NodeDetailPanel node={node} />)

      expect(screen.getByText('?')).toBeInTheDocument()
      expect(screen.getByText('可能')).toBeInTheDocument()
    })

    it('显示未知置信度', () => {
      const node = createTestNode({
        confidence: { level: 'Unknown', reason: '测试原因' },
      })
      render(<NodeDetailPanel node={node} />)

      expect(screen.getByText('!')).toBeInTheDocument()
      expect(screen.getByText('未知')).toBeInTheDocument()
    })
  })

  describe('调用列表', () => {
    it('显示被调用函数列表', () => {
      const node = createTestNode()
      render(<NodeDetailPanel node={node} />)

      expect(screen.getByText('调用 (1)')).toBeInTheDocument()
      expect(screen.getByText('child_function()')).toBeInTheDocument()
    })

    it('点击被调用函数触发 onNodeClick', () => {
      const onNodeClick = vi.fn()
      const node = createTestNode()
      render(<NodeDetailPanel node={node} onNodeClick={onNodeClick} />)

      const callItem = screen.getByText('child_function()').closest('.call-item') as HTMLElement
      fireEvent.click(callItem)

      expect(onNodeClick).toHaveBeenCalledWith('child_function')
    })

    it('显示调用者列表', () => {
      const node = createTestNode()
      render(<NodeDetailPanel node={node} />)

      expect(screen.getByText('被调用 (2)')).toBeInTheDocument()
      expect(screen.getByText('caller1()')).toBeInTheDocument()
      expect(screen.getByText('caller2()')).toBeInTheDocument()
    })
  })

  describe('参数列表', () => {
    it('显示参数信息', () => {
      const node = createTestNode()
      render(<NodeDetailPanel node={node} />)

      expect(screen.getByText('参数 (2)')).toBeInTheDocument()
      expect(screen.getByText('struct device *')).toBeInTheDocument()
      expect(screen.getByText('dev')).toBeInTheDocument()
    })
  })

  describe('位置信息', () => {
    it('显示文件位置', () => {
      const node = createTestNode()
      render(<NodeDetailPanel node={node} />)

      expect(screen.getByText('hub.c')).toBeInTheDocument()
      expect(screen.getByText('1234')).toBeInTheDocument()
    })
  })

  describe('关闭按钮', () => {
    it('显示关闭按钮 when onClose provided', () => {
      const onClose = vi.fn()
      const node = createTestNode()
      render(<NodeDetailPanel node={node} onClose={onClose} />)

      expect(screen.getByTitle('关闭')).toBeInTheDocument()
    })

    it('点击关闭按钮触发 onClose', () => {
      const onClose = vi.fn()
      const node = createTestNode()
      render(<NodeDetailPanel node={node} onClose={onClose} />)

      const closeBtn = screen.getByTitle('关闭')
      fireEvent.click(closeBtn)

      expect(onClose).toHaveBeenCalled()
    })
  })

  describe('LLVM IR 显示', () => {
    it('显示 LLVM IR 代码', () => {
      const node = createTestNode()
      render(<NodeDetailPanel node={node} />)

      expect(screen.getByText('LLVM IR')).toBeInTheDocument()
      expect(screen.getByText('5 行')).toBeInTheDocument()
      expect(screen.getByText('define i32 @test_function')).toBeInTheDocument()
    })

    it('当 showLlvmIr 为 false 时不显示 LLVM IR', () => {
      const node = createTestNode()
      render(<NodeDetailPanel node={node} showLlvmIr={false} />)

      expect(screen.queryByText('LLVM IR')).not.toBeInTheDocument()
    })
  })

  describe('边界情况', () => {
    it('处理无 children 的节点', () => {
      const node = createTestNode({ children: undefined })
      render(<NodeDetailPanel node={node} />)

      expect(screen.queryByText('调用 (0)')).not.toBeInTheDocument()
    })

    it('处理无 callers 的节点', () => {
      const node = createTestNode({ callers: undefined })
      render(<NodeDetailPanel node={node} />)

      expect(screen.queryByText('被调用 (0)')).not.toBeInTheDocument()
    })

    it('处理无参数的函数', () => {
      const node = createTestNode({ params: undefined })
      render(<NodeDetailPanel node={node} />)

      expect(screen.queryByText('参数')).not.toBeInTheDocument()
    })

    it('处理无置信度的节点', () => {
      const node = createTestNode({ confidence: undefined })
      render(<NodeDetailPanel node={node} />)

      expect(screen.queryByText('确定')).not.toBeInTheDocument()
    })

    it('处理空描述的节点', () => {
      const node = createTestNode({ description: undefined })
      render(<NodeDetailPanel node={node} />)

      expect(screen.queryByText('描述')).not.toBeInTheDocument()
    })

    it('处理空 LLVM IR 的节点', () => {
      const node = createTestNode({ llvmIr: [] })
      render(<NodeDetailPanel node={node} />)

      expect(screen.getByText('0 行')).toBeInTheDocument()
    })
  })

  describe('自定义标题', () => {
    it('使用自定义标题', () => {
      const node = createTestNode()
      render(<NodeDetailPanel node={node} title="自定义标题" />)

      expect(screen.getByText('自定义标题')).toBeInTheDocument()
    })
  })

  describe('位置信息显示', () => {
    it('显示节点位置信息', () => {
      const node = createTestNode({
        location: {
          file: '/kernel/drivers/usb/core/hub.c',
          line: 1234,
          column: 10,
        },
      })
      render(<NodeDetailPanel node={node} />)

      expect(screen.getByText('位置')).toBeInTheDocument()
    })
  })
})
