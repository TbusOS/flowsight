/**
 * LlvmIrPanel Tests
 *
 * 测试 LLVM IR 可视化面板的渲染、语法高亮、搜索、展开/折叠等功能
 */

import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import {
  LlvmIrPanel,
  LlvmIrParseResult,
  LlvmFunction,
  LlvmBasicBlock,
  LlvmInstruction,
} from '../LlvmIrPanel'

// 测试数据工厂函数
const createTestInstruction = (overrides: Partial<LlvmInstruction> = {}): LlvmInstruction => ({
  opcode: 'add',
  dest: 'result',
  typeStr: 'i32',
  operands: ['%a', '%b'],
  ...overrides,
})

const createTestBlock = (name: string, instructionCount: number = 3): LlvmBasicBlock => ({
  name,
  instructions: Array.from({ length: instructionCount }, (_, i) =>
    createTestInstruction({ opcode: ['add', 'sub', 'mul'][i % 3] })
  ),
  predecessors: [],
  successors: [],
  terminator: createTestInstruction({ opcode: 'ret' }),
})

const createTestFunction = (name: string, blockCount: number = 2): LlvmFunction => ({
  name,
  returnType: 'i32',
  parameters: [
    { name: 'arg1', typeStr: 'i32' },
    { name: 'arg2', typeStr: 'i32' },
  ],
  blocks: Array.from({ length: blockCount }, (_, i) => createTestBlock(`${name}.bb${i}`)),
  isCallback: false,
})

const createTestParseResult = (): LlvmIrParseResult => ({
  moduleName: 'test_module',
  functions: {
    'test_func': createTestFunction('test_func'),
    'helper_func': createTestFunction('helper_func'),
  },
})

describe('LlvmIrPanel', () => {
  describe('渲染状态', () => {
    it('渲染空状态 when parseResult is null', () => {
      render(<LlvmIrPanel parseResult={null} />)

      expect(screen.getByText('暂无 LLVM IR 数据')).toBeInTheDocument()
      expect(screen.getByText('请先加载 LLVM IR 文件')).toBeInTheDocument()
    })

    it('渲染加载状态 when loading is true', () => {
      render(<LlvmIrPanel parseResult={null} loading />)

      expect(screen.getByText('加载 LLVM IR 数据...')).toBeInTheDocument()
    })

    it('渲染错误状态 when error is provided', () => {
      render(<LlvmIrPanel parseResult={null} error="解析失败" />)

      expect(screen.getByText('解析失败')).toBeInTheDocument()
    })

    it('渲染空搜索结果 when search has no matches', () => {
      const parseResult = createTestParseResult()
      render(
        <LlvmIrPanel
          parseResult={parseResult}
          selectedFunction="test_func"
        />
      )

      // 在搜索框中输入不存在的查询
      const searchInput = screen.getByPlaceholderText('搜索指令、寄存器...')
      fireEvent.change(searchInput, { target: { value: 'nonexistent' } })

      expect(screen.getByText('未找到匹配 "nonexistent" 的基本块')).toBeInTheDocument()
    })
  })

  describe('函数选择器', () => {
    it('显示函数选择器下拉框', () => {
      const parseResult = createTestParseResult()
      render(<LlvmIrPanel parseResult={parseResult} />)

      expect(screen.getByRole('combobox')).toBeInTheDocument()
    })

    it('显示所有可用函数', () => {
      const parseResult = createTestParseResult()
      render(<LlvmIrPanel parseResult={parseResult} />)

      expect(screen.getByText('test_func')).toBeInTheDocument()
      expect(screen.getByText('helper_func')).toBeInTheDocument()
    })

    it('回调函数显示闪电图标', () => {
      const parseResult: LlvmIrParseResult = {
        moduleName: 'test_module',
        functions: {
          'callback_func': {
            ...createTestFunction('callback_func'),
            isCallback: true,
            callbackContext: 'WorkQueue',
          },
        },
      }
      render(<LlvmIrPanel parseResult={parseResult} />)

      // 闪电图标在选择器选项中显示
      expect(screen.getByRole('option', { name: /callback_func.*⚡/ })).toBeInTheDocument()
    })

    it('选择函数触发 onFunctionSelect', () => {
      const onFunctionSelect = vi.fn()
      const parseResult = createTestParseResult()
      render(
        <LlvmIrPanel
          parseResult={parseResult}
          onFunctionSelect={onFunctionSelect}
        />
      )

      const select = screen.getByRole('combobox')
      fireEvent.change(select, { target: { value: 'test_func' } })

      expect(onFunctionSelect).toHaveBeenCalledWith('test_func')
    })
  })

  describe('搜索功能', () => {
    it('显示搜索框', () => {
      const parseResult = createTestParseResult()
      render(<LlvmIrPanel parseResult={parseResult} />)

      expect(screen.getByPlaceholderText('搜索指令、寄存器...')).toBeInTheDocument()
    })

    it('搜索匹配基本块名称', () => {
      const parseResult = createTestParseResult()
      render(
        <LlvmIrPanel
          parseResult={parseResult}
          selectedFunction="test_func"
        />
      )

      const searchInput = screen.getByPlaceholderText('搜索指令、寄存器...')
      fireEvent.change(searchInput, { target: { value: 'bb0' } })

      // bb0 块应该显示
      expect(screen.getByText('test_func.bb0:')).toBeInTheDocument()
    })

    it('搜索匹配指令操作码', () => {
      const parseResult = createTestParseResult()
      render(
        <LlvmIrPanel
          parseResult={parseResult}
          selectedFunction="test_func"
        />
      )

      const searchInput = screen.getByPlaceholderText('搜索指令、寄存器...')
      fireEvent.change(searchInput, { target: { value: 'add' } })

      // 包含 add 指令的块应该显示
      expect(screen.getByText('test_func.bb0:')).toBeInTheDocument()
    })

    it('搜索不区分大小写', () => {
      const parseResult = createTestParseResult()
      render(
        <LlvmIrPanel
          parseResult={parseResult}
          selectedFunction="test_func"
        />
      )

      const searchInput = screen.getByPlaceholderText('搜索指令、寄存器...')
      fireEvent.change(searchInput, { target: { value: 'ADD' } })

      expect(screen.getByText('test_func.bb0:')).toBeInTheDocument()
    })

    it('清空搜索显示所有块', () => {
      const parseResult = createTestParseResult()
      render(
        <LlvmIrPanel
          parseResult={parseResult}
          selectedFunction="test_func"
        />
      )

      const searchInput = screen.getByPlaceholderText('搜索指令、寄存器...')
      fireEvent.change(searchInput, { target: { value: 'test' } })
      fireEvent.change(searchInput, { target: { value: '' } })

      expect(screen.getByText('test_func.bb0:')).toBeInTheDocument()
      expect(screen.getByText('test_func.bb1:')).toBeInTheDocument()
    })
  })

  describe('基本块展开/折叠', () => {
    it('基本块默认展开', () => {
      const parseResult = createTestParseResult()
      render(
        <LlvmIrPanel
          parseResult={parseResult}
          selectedFunction="test_func"
        />
      )

      expect(screen.getByText('test_func.bb0:')).toBeInTheDocument()
      expect(screen.getByText('3 条指令')).toBeInTheDocument()
    })

    it('点击块头部切换展开/折叠', () => {
      const parseResult = createTestParseResult()
      render(
        <LlvmIrPanel
          parseResult={parseResult}
          selectedFunction="test_func"
        />
      )

      const blockHeader = screen.getByText('test_func.bb0:').closest('.block-header') as HTMLElement
      fireEvent.click(blockHeader)

      expect(screen.getByText('3 条指令 (点击展开)')).toBeInTheDocument()
    })

    it('折叠后不显示指令内容', () => {
      const parseResult = createTestParseResult()
      render(
        <LlvmIrPanel
          parseResult={parseResult}
          selectedFunction="test_func"
        />
      )

      const blockHeader = screen.getByText('test_func.bb0:').closest('.block-header') as HTMLElement
      fireEvent.click(blockHeader)

      expect(screen.queryByText('%result = add i32')).not.toBeInTheDocument()
    })

    it('点击块触发 onBlockClick', () => {
      const onBlockClick = vi.fn()
      const parseResult = createTestParseResult()
      render(
        <LlvmIrPanel
          parseResult={parseResult}
          selectedFunction="test_func"
          onBlockClick={onBlockClick}
        />
      )

      const blockHeader = screen.getByText('test_func.bb0:').closest('.block-header') as HTMLElement
      fireEvent.click(blockHeader)

      expect(onBlockClick).toHaveBeenCalledWith('test_func.bb0')
    })
  })

  describe('语法高亮', () => {
    it('高亮显示关键字 define', () => {
      const parseResult: LlvmIrParseResult = {
        moduleName: 'test_module',
        functions: {
          'test_func': {
            ...createTestFunction('test_func'),
            blocks: [],
          },
        },
      }
      render(
        <LlvmIrPanel
          parseResult={parseResult}
          selectedFunction="test_func"
        />
      )

      expect(screen.getByText('define')).toBeInTheDocument()
    })

    it('高亮显示标签行', () => {
      const parseResult = createTestParseResult()
      render(
        <LlvmIrPanel
          parseResult={parseResult}
          selectedFunction="test_func"
        />
      )

      expect(screen.getByText('test_func.bb0:')).toBeInTheDocument()
    })

    it('高亮显示寄存器', () => {
      const parseResult = createTestParseResult()
      render(
        <LlvmIrPanel
          parseResult={parseResult}
          selectedFunction="test_func"
        />
      )

      expect(screen.getByText('%result')).toBeInTheDocument()
    })

    it('高亮显示数字常量', () => {
      const parseResult: LlvmIrParseResult = {
        moduleName: 'test_module',
        functions: {
          'test_func': {
            ...createTestFunction('test_func'),
            blocks: [
              {
                name: 'entry',
                instructions: [
                  { opcode: 'add', dest: 'x', typeStr: 'i32', operands: ['1', '2'] },
                ],
                predecessors: [],
                successors: [],
              },
            ],
            isCallback: false,
          },
        },
      }
      render(
        <LlvmIrPanel
          parseResult={parseResult}
          selectedFunction="test_func"
        />
      )

      expect(screen.getByText('1')).toBeInTheDocument()
      expect(screen.getByText('2')).toBeInTheDocument()
    })

    it('高亮显示注释', () => {
      const parseResult: LlvmIrParseResult = {
        moduleName: 'test_module',
        functions: {
          'test_func': {
            ...createTestFunction('test_func'),
            blocks: [
              {
                name: 'entry',
                instructions: [
                  { opcode: 'add', typeStr: 'i32', operands: ['%a', '%b'] },
                ],
                predecessors: [],
                successors: [],
              },
            ],
            isCallback: false,
          },
        },
      }

      // 手动测试带注释的行
      const block = parseResult.functions['test_func'].blocks[0]
      block.instructions = [
        { opcode: 'add', typeStr: 'i32', operands: ['%a', '%b'] },
      ]

      render(
        <LlvmIrPanel
          parseResult={parseResult}
          selectedFunction="test_func"
        />
      )

      expect(screen.getByText('entry:')).toBeInTheDocument()
    })
  })

  describe('关键指令标注', () => {
    it('call 指令显示 CALL 标签', () => {
      const parseResult: LlvmIrParseResult = {
        moduleName: 'test_module',
        functions: {
          'test_func': {
            ...createTestFunction('test_func'),
            blocks: [
              {
                name: 'entry',
                instructions: [
                  { opcode: 'call', typeStr: 'void', operands: ['@printf'] },
                ],
                predecessors: [],
                successors: [],
                terminator: createTestInstruction({ opcode: 'ret', typeStr: 'void' }),
              },
            ],
            isCallback: false,
          },
        },
      }
      render(
        <LlvmIrPanel
          parseResult={parseResult}
          selectedFunction="test_func"
        />
      )

      expect(screen.getByText('CALL')).toBeInTheDocument()
    })

    it('ret 指令显示 RET 标签', () => {
      const parseResult = createTestParseResult()
      render(
        <LlvmIrPanel
          parseResult={parseResult}
          selectedFunction="test_func"
        />
      )

      expect(screen.getByText('RET')).toBeInTheDocument()
    })

    it('br 指令显示 BR 标签', () => {
      const parseResult: LlvmIrParseResult = {
        moduleName: 'test_module',
        functions: {
          'test_func': {
            ...createTestFunction('test_func'),
            blocks: [
              {
                name: 'entry',
                instructions: [],
                predecessors: [],
                successors: ['then', 'else'],
                terminator: { opcode: 'br', typeStr: 'label', operands: ['%cond'] },
              },
            ],
            isCallback: false,
          },
        },
      }
      render(
        <LlvmIrPanel
          parseResult={parseResult}
          selectedFunction="test_func"
        />
      )

      expect(screen.getByText('BR')).toBeInTheDocument()
    })

    it('store/load 指令显示对应标签', () => {
      const parseResult: LlvmIrParseResult = {
        moduleName: 'test_module',
        functions: {
          'test_func': {
            ...createTestFunction('test_func'),
            blocks: [
              {
                name: 'entry',
                instructions: [
                  { opcode: 'store', typeStr: 'i32', operands: ['%val', '%ptr'] },
                  { opcode: 'load', dest: 'result', typeStr: 'i32', operands: ['%ptr'] },
                ],
                predecessors: [],
                successors: [],
              },
            ],
            isCallback: false,
          },
        },
      }
      render(
        <LlvmIrPanel
          parseResult={parseResult}
          selectedFunction="test_func"
        />
      )

      expect(screen.getByText('STORE')).toBeInTheDocument()
      expect(screen.getByText('LOAD')).toBeInTheDocument()
    })
  })

  describe('指令点击回调', () => {
    it('点击指令触发 onInstructionClick', () => {
      const onInstructionClick = vi.fn()
      const parseResult = createTestParseResult()
      render(
        <LlvmIrPanel
          parseResult={parseResult}
          selectedFunction="test_func"
          onInstructionClick={onInstructionClick}
        />
      )

      const instructionLine = screen.getByText('add').closest('.llvm-code-line') as HTMLElement
      fireEvent.click(instructionLine)

      expect(onInstructionClick).toHaveBeenCalled()
    })
  })

  describe('主题模式', () => {
    it('默认使用暗色主题', () => {
      const parseResult = createTestParseResult()
      render(<LlvmIrPanel parseResult={parseResult} />)

      const panel = screen.getByText('LLVM IR 可视化').closest('.llvm-ir-panel') as HTMLElement
      expect(panel).toBeInTheDocument()
    })

    it('支持浅色主题', () => {
      const parseResult = createTestParseResult()
      render(<LlvmIrPanel parseResult={parseResult} theme="light" />)

      const panel = screen.getByText('LLVM IR 可视化').closest('.llvm-ir-panel') as HTMLElement
      expect(panel).toHaveClass('light')
    })
  })

  describe('函数信息显示', () => {
    it('显示函数签名', () => {
      const parseResult = createTestParseResult()
      render(
        <LlvmIrPanel
          parseResult={parseResult}
          selectedFunction="test_func"
        />
      )

      expect(screen.getByText('define')).toBeInTheDocument()
      expect(screen.getByText('i32')).toBeInTheDocument()
      expect(screen.getByText('@test_func')).toBeInTheDocument()
    })

    it('回调函数显示回调标记', () => {
      const parseResult: LlvmIrParseResult = {
        moduleName: 'test_module',
        functions: {
          'callback_func': {
            ...createTestFunction('callback_func'),
            isCallback: true,
            callbackContext: 'WorkQueue',
          },
        },
      }
      render(
        <LlvmIrPanel
          parseResult={parseResult}
          selectedFunction="callback_func"
        />
      )

      expect(screen.getByText('回调函数')).toBeInTheDocument()
    })
  })

  describe('基本块前驱后继', () => {
    it('显示前驱块信息', () => {
      const parseResult: LlvmIrParseResult = {
        moduleName: 'test_module',
        functions: {
          'test_func': {
            ...createTestFunction('test_func'),
            blocks: [
              {
                name: 'then',
                instructions: [],
                predecessors: ['entry'],
                successors: [],
              },
            ],
            isCallback: false,
          },
        },
      }
      render(
        <LlvmIrPanel
          parseResult={parseResult}
          selectedFunction="test_func"
        />
      )

      expect(screen.getByText('← entry')).toBeInTheDocument()
    })

    it('显示后继块信息', () => {
      const parseResult: LlvmIrParseResult = {
        moduleName: 'test_module',
        functions: {
          'test_func': {
            ...createTestFunction('test_func'),
            blocks: [
              {
                name: 'entry',
                instructions: [],
                predecessors: [],
                successors: ['then', 'else'],
              },
            ],
            isCallback: false,
          },
        },
      }
      render(
        <LlvmIrPanel
          parseResult={parseResult}
          selectedFunction="test_func"
        />
      )

      expect(screen.getByText('→ then, else')).toBeInTheDocument()
    })
  })

  describe('关键指令图例', () => {
    it('显示指令图例', () => {
      const parseResult = createTestParseResult()
      render(
        <LlvmIrPanel
          parseResult={parseResult}
          selectedFunction="test_func"
        />
      )

      expect(screen.getByText('关键指令:')).toBeInTheDocument()
      expect(screen.getByText('CALL')).toBeInTheDocument()
      expect(screen.getByText('RET')).toBeInTheDocument()
      expect(screen.getByText('BR')).toBeInTheDocument()
    })
  })

  describe('面板信息', () => {
    it('显示模块名称', () => {
      const parseResult = createTestParseResult()
      render(
        <LlvmIrPanel
          parseResult={parseResult}
          selectedFunction="test_func"
        />
      )

      expect(screen.getByText('模块: test_module')).toBeInTheDocument()
    })

    it('显示函数计数', () => {
      const parseResult = createTestParseResult()
      render(<LlvmIrPanel parseResult={parseResult} />)

      expect(screen.getByText('2 个函数')).toBeInTheDocument()
    })
  })

  describe('边界情况', () => {
    it('处理无块的基本块', () => {
      const parseResult: LlvmIrParseResult = {
        moduleName: 'test_module',
        functions: {
          'test_func': {
            ...createTestFunction('test_func'),
            blocks: [],
          },
        },
      }
      render(
        <LlvmIrPanel
          parseResult={parseResult}
          selectedFunction="test_func"
        />
      )

      expect(screen.queryByText('基本块')).not.toBeInTheDocument()
    })

    it('处理无参数函数', () => {
      const parseResult: LlvmIrParseResult = {
        moduleName: 'test_module',
        functions: {
          'test_func': {
            ...createTestFunction('test_func'),
            parameters: [],
          },
        },
      }
      render(
        <LlvmIrPanel
          parseResult={parseResult}
          selectedFunction="test_func"
        />
      )

      expect(screen.getByText('()')).toBeInTheDocument()
    })
  })

  describe('行号显示', () => {
    it('显示行号', () => {
      const parseResult = createTestParseResult()
      render(
        <LlvmIrPanel
          parseResult={parseResult}
          selectedFunction="test_func"
        />
      )

      expect(screen.getByText('1')).toBeInTheDocument()
    })
  })
})
