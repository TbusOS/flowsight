/**
 * FlowSight Display Components - Index
 * All IDE visualization panels for execution flow display
 */

// Main execution flow views
export { FlowTextView } from './FlowView/FlowTextView'
export type { FlowTextViewProps, TextLine } from './FlowView/FlowTextView'

// Node detail panel (new version)
export { NodeDetailPanel } from './panels/node-detail-panel'

// AI-assisted translation and explanation panels
export { ConditionTranslationPanel } from './ConditionTranslationPanel/ConditionTranslationPanel'
export type { ConditionTranslation, ConditionTranslationPanelProps } from './ConditionTranslationPanel/ConditionTranslationPanel'

export { BusinessSemanticsPanel } from './BusinessSemanticsPanel/BusinessSemanticsPanel'
export type { BusinessExplanation, BusinessSemanticsPanelProps } from './BusinessSemanticsPanel/BusinessSemanticsPanel'

export { ExecutionContextPanel } from './ExecutionContextPanel/ExecutionContextPanel'
export type { ExecutionContextAnnotation, ExecutionContextPanelProps } from './ExecutionContextPanel/ExecutionContextPanel'

// LLVM IR 可视化面板
export { LlvmIrPanel } from './LlvmIrPanel'
export type {
  LlvmBasicBlock,
  LlvmInstruction,
  LlvmFunction,
  LlvmParameter,
  LlvmIrParseResult,
  LlvmIrPanelProps,
} from './LlvmIrPanel'
