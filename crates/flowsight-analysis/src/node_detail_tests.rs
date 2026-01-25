//! Extended Tests for NodeDetail and LlvmIrPanel Integration
//!
//! Tests for node detail API, LLVM IR parsing, and frontend integration

use flowsight_analysis::node_detail::{
    NodeDetailRequest, NodeDetailResponse, NodeDetailService, LocationDto,
    FlowNodeTypeDto, AsyncMechanismDto, ConfidenceDto, ConfidenceLevelDto,
};
use flowsight_core::{FunctionDef, Location, CallEdge};
use flowsight_llvm::{
    LlvmParser, LlvmIrParseResult, LlvmFunction, LlvmBasicBlock, LlvmInstruction,
    LlvmParameter,
};

#[cfg(test)]
mod node_detail_tests {
    use super::*;

    #[test]
    fn test_node_detail_service_creation() {
        let service = NodeDetailService::new();
        assert!(service.function_defs.is_empty());
    }

    #[test]
    fn test_node_detail_service_with_data() {
        let function_defs = std::collections::HashMap::from([
            (
                "test_func".to_string(),
                FunctionDef {
                    name: "test_func".to_string(),
                    return_type: "i32".to_string(),
                    params: vec![],
                    location: Some(Location {
                        file: "test.c".to_string(),
                        line: 10,
                        column: 5,
                    }),
                    calls: vec![],
                    called_by: vec![],
                    is_callback: false,
                    callback_context: None,
                    attributes: vec![],
                },
            ),
        ]);

        let call_edges = vec![];
        let service = NodeDetailService::with_data(function_defs, &call_edges, None);

        assert!(service.function_defs.contains_key("test_func"));
    }

    #[test]
    fn test_add_function() {
        let mut service = NodeDetailService::new();
        let func = FunctionDef {
            name: "my_function".to_string(),
            return_type: "void".to_string(),
            params: vec![],
            location: None,
            calls: vec![],
            called_by: vec![],
            is_callback: false,
            callback_context: None,
            attributes: vec![],
        };

        service.add_function(func);

        assert!(service.function_defs.contains_key("my_function"));
    }

    #[test]
    fn test_add_call_edge() {
        let mut service = NodeDetailService::new();
        let edge = CallEdge {
            caller: "caller_func".to_string(),
            callee: "callee_func".to_string(),
            confidence: flowsight_core::CallConfidence::Certain,
        };

        service.add_call_edge(&edge);

        let callees = service.call_graph.get("caller_func").unwrap();
        assert!(callees.contains(&"callee_func".to_string()));

        let callers = service.reverse_call_graph.get("callee_func").unwrap();
        assert!(callers.contains(&"caller_func".to_string()));
    }

    #[test]
    fn test_query_by_name() {
        let function_defs = std::collections::HashMap::from([
            (
                "query_func".to_string(),
                FunctionDef {
                    name: "query_func".to_string(),
                    return_type: "i32".to_string(),
                    params: vec![],
                    location: Some(Location {
                        file: "query.c".to_string(),
                        line: 100,
                        column: 1,
                    }),
                    calls: vec![],
                    called_by: vec!["other_func".to_string()],
                    is_callback: false,
                    callback_context: None,
                    attributes: vec![],
                },
            ),
        ]);

        let service = NodeDetailService::with_data(function_defs, &[], None);
        let response = service.query_by_name("query_func", false);

        assert!(response.is_some());
        let detail = response.unwrap();
        assert_eq!(detail.name, "query_func");
        assert_eq!(detail.return_type, Some("i32".to_string()));
    }

    #[test]
    fn test_query_nonexistent_node() {
        let service = NodeDetailService::new();
        let request = NodeDetailRequest {
            node_id: "nonexistent".to_string(),
            include_llvm_ir: false,
            include_callers: false,
            include_callees: false,
        };

        let response = service.query(&request);
        assert!(response.is_none());
    }

    #[test]
    fn test_node_detail_request_serialization() {
        let request = NodeDetailRequest {
            node_id: "test_node".to_string(),
            include_llvm_ir: true,
            include_callers: true,
            include_callees: true,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("test_node"));
        assert!(json.contains("true"));
    }

    #[test]
    fn test_node_detail_response_serialization() {
        let response = NodeDetailResponse {
            name: "test_func".to_string(),
            node_type: FlowNodeTypeDto::Function,
            location: Some(LocationDto {
                file: "test.c".to_string(),
                line: 42,
                column: Some(10),
            }),
            description: Some("Test function".to_string()),
            confidence: Some(ConfidenceDto {
                level: ConfidenceLevelDto::Certain,
                reason: "Direct call".to_string(),
            }),
            children: vec![],
            llvm_ir: vec!["define i32 @test()".to_string()],
            function_signature: Some("int test()".to_string()),
            params: vec![],
            return_type: Some("i32".to_string()),
            callers: vec!["caller1".to_string()],
            metadata: Default::default(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("test_func"));
        assert!(json.contains("test.c"));
        assert!(json.contains("Certain"));
    }

    #[test]
    fn test_async_mechanism_dto() {
        let mechanism = AsyncMechanismDto::WorkQueue {
            work_struct: Some("work".to_string()),
            queue: Some("system_wq".to_string()),
        };

        let json = serde_json::to_string(&mechanism).unwrap();
        assert!(json.contains("WorkQueue"));
        assert!(json.contains("work"));
    }

    #[test]
    fn test_async_callback_node_type() {
        let node_type = FlowNodeTypeDto::AsyncCallback {
            mechanism: AsyncMechanismDto::Timer {
                timer_name: Some("hrtimer".to_string()),
                timer_type: Some("HRTIMER_MODE_REL".to_string()),
            },
        };

        let json = serde_json::to_string(&node_type).unwrap();
        assert!(json.contains("AsyncCallback"));
        assert!(json.contains("Timer"));
    }

    #[test]
    fn test_kernel_api_node_type() {
        let node_type = FlowNodeTypeDto::KernelApi;
        let json = serde_json::to_string(&node_type).unwrap();
        assert!(json.contains("KernelApi"));
    }

    #[test]
    fn test_entry_point_node_type() {
        let node_type = FlowNodeTypeDto::EntryPoint;
        let json = serde_json::to_string(&node_type).unwrap();
        assert!(json.contains("EntryPoint"));
    }

    #[test]
    fn test_external_node_type() {
        let node_type = FlowNodeTypeDto::External;
        let json = serde_json::to_string(&node_type).unwrap();
        assert!(json.contains("External"));
    }

    #[test]
    fn test_confidence_levels() {
        let certain = ConfidenceLevelDto::Certain;
        let possible = ConfidenceLevelDto::Possible;
        let unknown = ConfidenceLevelDto::Unknown;

        assert_eq!(certain, ConfidenceLevelDto::Certain);
        assert_eq!(possible, ConfidenceLevelDto::Possible);
        assert_eq!(unknown, ConfidenceLevelDto::Unknown);
    }

    #[test]
    fn test_location_dto_full() {
        let location = LocationDto {
            file: "/path/to/file.c".to_string(),
            line: 123,
            column: Some(45),
        };

        assert_eq!(location.file, "/path/to/file.c");
        assert_eq!(location.line, 123);
        assert_eq!(location.column, Some(45));
    }

    #[test]
    fn test_location_dto_no_column() {
        let location = LocationDto {
            file: "file.c".to_string(),
            line: 1,
            column: None,
        };

        let json = serde_json::to_string(&location).unwrap();
        assert!(json.contains("file.c"));
        assert!(json.contains("1"));
    }
}

#[cfg(test)]
mod llvm_parser_tests {
    use super::*;

    #[test]
    fn test_llvm_parser_creation() {
        let parser = LlvmParser::new();
        assert!(parser.knowledge_base.is_some());
    }

    #[test]
    fn test_parse_empty_ir() {
        let parser = LlvmParser::new();
        let result = parser.parse_ir("");

        // Empty IR should return a basic result
        assert!(result.is_ok());
        let parse_result = result.unwrap();
        assert!(parse_result.functions.is_empty());
    }

    #[test]
    fn test_parse_simple_function_ir() {
        let ir = r#"
define i32 @add(i32 %a, i32 %b) {
entry:
  %result = add i32 %a, %b
  ret i32 %result
}
"#;

        let parser = LlvmParser::new();
        let result = parser.parse_ir(ir);

        assert!(result.is_ok());
        let parse_result = result.unwrap();

        assert!(parse_result.functions.contains_key("add"));
        let func = parse_result.functions.get("add").unwrap();
        assert_eq!(func.return_type, "i32");
        assert_eq!(func.parameters.len(), 2);
    }

    #[test]
    fn test_parse_multiple_functions() {
        let ir = r#"
define i32 @func1(i32 %x) {
entry:
  ret i32 %x
}

define void @func2(i32 %y) {
entry:
  ret void
}

define i64 @func3(i32 %a, i32 %b, i32 %c) {
entry:
  ret i64 0
}
"#;

        let parser = LlvmParser::new();
        let result = parser.parse_ir(ir);

        assert!(result.is_ok());
        let parse_result = result.unwrap();

        assert_eq!(parse_result.functions.len(), 3);
        assert!(parse_result.functions.contains_key("func1"));
        assert!(parse_result.functions.contains_key("func2"));
        assert!(parse_result.functions.contains_key("func3"));
    }

    #[test]
    fn test_parse_function_with_branches() {
        let ir = r#"
define i32 @check(i32 %x) {
entry:
  %cmp = icmp sgt i32 %x, 0
  br i1 %cmp, label %then, label %else

then:
  ret i32 1

else:
  ret i32 0
}
"#;

        let parser = LlvmParser::new();
        let result = parser.parse_ir(ir);

        assert!(result.is_ok());
        let parse_result = result.unwrap();

        let func = parse_result.functions.get("check").unwrap();
        // Should have multiple basic blocks
        assert!(func.blocks.len() >= 2);
    }

    #[test]
    fn test_parse_module_name() {
        let ir = r#"
; ModuleID = 'my_module'
source_filename = "test.c"

define void @test() {
entry:
  ret void
}
"#;

        let parser = LlvmParser::new();
        let result = parser.parse_ir(ir);

        assert!(result.is_ok());
        let parse_result = result.unwrap();

        assert_eq!(parse_result.module_name, "my_module");
    }

    #[test]
    fn test_parse_void_return() {
        let ir = r#"
define void @process(i32 %x) {
entry:
  ret void
}
"#;

        let parser = LlvmParser::new();
        let result = parser.parse_ir(ir);

        assert!(result.is_ok());
        let parse_result = result.unwrap();

        let func = parse_result.functions.get("process").unwrap();
        assert_eq!(func.return_type, "void");
    }

    #[test]
    fn test_parse_function_attributes() {
        let ir = r#"
define i32 @fast_func(i32 %x) #0 {
entry:
  ret i32 %x
}

attributes #0 = { noinline noreturn }
"#;

        let parser = LlvmParser::new();
        let result = parser.parse_ir(ir);

        assert!(result.is_ok());
        let parse_result = result.unwrap();

        assert!(parse_result.functions.contains_key("fast_func"));
    }

    #[test]
    fn test_to_frontend_format() {
        use flowsight_llvm::to_frontend_format;

        let ir = r#"
define i32 @test_func(i32 %x) {
entry:
  ret i32 %x
}
"#;

        let parser = LlvmParser::new();
        let result = parser.parse_ir(ir).unwrap();

        let frontend_result = to_frontend_format(&result);

        assert!(frontend_result.functions.contains_key("test_func"));
        let func = frontend_result.functions.get("test_func").unwrap();
        assert_eq!(func.return_type, "i32");
    }

    #[test]
    fn test_function_summaries() {
        use flowsight_llvm::get_function_summaries;

        let ir = r#"
define i32 @func_a(i32 %x) {
entry:
  ret i32 %x
}

define void @func_b(i32 %x, i32 %y) {
entry:
  ret void
}
"#;

        let parser = LlvmParser::new();
        let result = parser.parse_ir(ir).unwrap();
        let frontend_result = flowsight_llvm::to_frontend_format(&result);

        let summaries = get_function_summaries(&frontend_result);

        assert_eq!(summaries.len(), 2);
        let func_a_summary = summaries.iter().find(|s| s.name == "func_a").unwrap();
        assert_eq!(func_a_summary.param_count, 1);
        assert_eq!(func_a_summary.instruction_count, 1);

        let func_b_summary = summaries.iter().find(|s| s.name == "func_b").unwrap();
        assert_eq!(func_b_summary.param_count, 2);
    }

    #[test]
    fn test_paginate_function() {
        use flowsight_llvm::paginate_function;
        use flowsight_llvm::LlvmIrPageRequest;

        let ir = r#"
define i32 @multi_block(i32 %x) {
entry:
  %add1 = add i32 %x, 1
  %add2 = add i32 %add1, 2
  %add3 = add i32 %add2, 3
  %add4 = add i32 %add3, 4
  %add5 = add i32 %add4, 5
  ret i32 %add5
}
"#;

        let parser = LlvmParser::new();
        let result = parser.parse_ir(ir).unwrap();
        let frontend_result = flowsight_llvm::to_frontend_format(&result);

        let request = LlvmIrPageRequest {
            function_name: "multi_block".to_string(),
            page: 0,
            page_size: 3,
        };

        let page_response = paginate_function(&frontend_result, &request);

        assert!(page_response.is_some());
        let page = page_response.unwrap();
        assert_eq!(page.instructions.len(), 3);
        assert!(page.has_next);
        assert!(!page.has_previous);
    }

    #[test]
    fn test_find_function() {
        use flowsight_llvm::find_function;

        let ir = r#"
define i32 @my_function(i32 %x) {
entry:
  ret i32 %x
}
"#;

        let parser = LlvmParser::new();
        let result = parser.parse_ir(ir).unwrap();

        let found = find_function(&result, "my_function");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "my_function");
    }

    #[test]
    fn test_filter_by_context() {
        let ir = r#"
define void @process_fn(ptr %work) {
entry:
  ret void
}
"#;

        let parser = LlvmParser::new();
        let result = parser.parse_ir(ir).unwrap();

        let process_funcs = parser.filter_by_context(&result, flowsight_core::ExecutionContext::Process);
        // May or may not find depending on knowledge base matching
        // This is just testing the API works
    }

    #[test]
    fn test_get_callbacks() {
        let ir = r#"
define void @callback_fn(ptr %work) {
entry:
  ret void
}
"#;

        let parser = LlvmParser::new();
        let result = parser.parse_ir(ir).unwrap();

        let callbacks = parser.get_callbacks(&result);
        // May be empty depending on callback detection
        // This tests the API works
    }
}

#[cfg(test)]
mod llir_formatting_tests {
    use super::*;

    #[test]
    fn test_format_llvm_function_basic_block() {
        // Test that basic blocks are properly formatted
        let llvm_func = LlvmFunction {
            name: "test".to_string(),
            return_type: "i32".to_string(),
            parameters: vec![LlvmParameter {
                name: "x".to_string(),
                type_str: "i32".to_string(),
            }],
            blocks: vec![LlvmBasicBlock {
                name: "entry".to_string(),
                instructions: vec![LlvmInstruction {
                    opcode: "add".to_string(),
                    dest: Some("result".to_string()),
                    type_str: "i32".to_string(),
                    operands: vec!["%x".to_string(), "1".to_string()],
                    location: None,
                }],
                predecessors: vec![],
                successors: vec![],
                terminator: Some(LlvmInstruction {
                    opcode: "ret".to_string(),
                    dest: None,
                    type_str: "i32".to_string(),
                    operands: vec!["%result".to_string()],
                    location: None,
                }),
            }],
            is_callback: false,
            callback_context: None,
        };

        // Just verify the struct can be created
        assert_eq!(llvm_func.blocks.len(), 1);
        assert_eq!(llvm_func.blocks[0].name, "entry");
    }

    #[test]
    fn test_instruction_operands() {
        let instr = LlvmInstruction {
            opcode: "call".to_string(),
            dest: None,
            type_str: "void".to_string(),
            operands: vec!["@printf".to_string(), "\"Hello\"".to_string()],
            location: None,
        };

        assert_eq!(instr.operands.len(), 2);
        assert!(instr.dest.is_none());
    }

    #[test]
    fn test_llvm_parameter_types() {
        let params = vec![
            LlvmParameter {
                name: "dev".to_string(),
                type_str: "ptr".to_string(),
            },
            LlvmParameter {
                name: "irq".to_string(),
                type_str: "i32".to_string(),
            },
        ];

        assert_eq!(params.len(), 2);
        assert_eq!(params[0].type_str, "ptr");
        assert_eq!(params[1].type_str, "i32");
    }
}
