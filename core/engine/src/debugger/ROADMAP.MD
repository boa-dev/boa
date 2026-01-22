# Boa Debugger - Development Roadmap

**Goal**: Achieve full SpiderMonkey-level debugger parity and complete DAP protocol support for professional IDE debugging experience.

**Philosophy**: Gradual, incremental implementation. Each DAP message is tackled individually with all required engine enhancements to make it fully functional.

## Current Status: Foundation Complete (~15%)

### ✅ What Works Today

**Core Infrastructure:**
- Debugger struct with state management (pause/resume)
- Thread-safe design with Arc<Mutex<>> and condition variables
- DAP protocol message parsing (all 30+ message types)
- DAP server with TCP transport
- Basic request/response routing

**Limitations:**
- Only pause/resume works
- Breakpoints stored but not checked by VM
- No VM hook integration
- No reflection/introspection
- No line-to-PC mapping

---

## Phase 1: VM Hook Integration

**Why First**: Without VM hooks, the debugger can't observe execution. This is the foundation for everything else.

### 1.1 Implement on_step Hook Calling

**Current Problem**: VM doesn't call `on_step()` before each instruction.

**Engine Work Required:**
```rust
// In core/engine/src/vm/mod.rs - execution loop
loop {
    // BEFORE executing instruction
    if let Some(hooks) = &context.host_hooks {
        if hooks.on_step(context, current_frame, pc)? {
            // Hook requested pause - wait on condition variable
            wait_while_paused(&debugger, &condvar);
        }
    }
    
    // Execute instruction
    let opcode = read_opcode(current_frame.code_block, pc);
    execute_instruction(opcode, context)?;
}
```

**Tasks:**
- [ ] Add hook call point in VM loop (before instruction execution)
- [ ] Handle pause request from hook
- [ ] Add condition variable wait mechanism
- [ ] Test with simple step tracing
- [ ] Measure performance impact (<5% target)

**Enables**: Step-through debugging, breakpoint checking, instruction tracing

---

### 1.2 Implement on_enter_frame / on_exit_frame Hooks

**Current Problem**: Frame lifecycle hooks defined but never called.

**Engine Work Required:**
```rust
// In frame push locations (8 call sites):
// core/engine/src/script.rs:214
// core/engine/src/builtins/function/mod.rs:1024, 1145
// core/engine/src/builtins/generator/mod.rs:99
// core/engine/src/builtins/json/mod.rs:142
// core/engine/src/builtins/eval/mod.rs:333
// core/engine/src/object/builtins/jspromise.rs:1231, 1288

// Before push_frame():
if let Some(hooks) = &context.host_hooks {
    if hooks.on_enter_frame(context, &new_frame)? {
        wait_while_paused(&debugger, &condvar);
    }
}
context.vm.push_frame(new_frame);

// Before pop_frame():
let frame = context.vm.current_frame();
if let Some(hooks) = &context.host_hooks {
    hooks.on_exit_frame(context, frame)?;
}
context.vm.pop_frame();
```

**Borrowing Challenge**: `push_frame()` borrows `context.vm` mutably, but we need to call hook with `context`.

**Solution Approaches:**
1. **Extract frame data first**: Get all needed info, then call hook
2. **Refactor push_frame signature**: `push_frame(frame, host_hooks: Option<&HostHooks>)`
3. **Add Context wrapper methods**: `context.push_frame_with_hooks(frame)`

**Tasks:**
- [ ] Choose and implement borrowing solution
- [ ] Update all 8 call sites
- [ ] Implement step-over logic using frame depth
- [ ] Implement step-out logic using frame depth
- [ ] Test with recursive functions

**Enables**: Step-over, step-out, call stack inspection

---

### 1.3 Implement on_debugger_statement Hook

**Current Problem**: `debugger;` statement in JavaScript doesn't trigger pause.

**Engine Work Required:**
```rust
// In core/engine/src/vm/opcode/control_flow.rs
// Handle Debugger opcode
Opcode::Debugger => {
    if let Some(hooks) = &context.host_hooks {
        if hooks.on_debugger_statement(context, current_frame)? {
            wait_while_paused(&debugger, &condvar);
        }
    }
}
```

**Tasks:**
- [ ] Add hook call in Debugger opcode handler
- [ ] Test with JavaScript `debugger;` statement
- [ ] Ensure DAP "stopped" event sent with reason="debugger statement"

**Enables**: `debugger;` statement support in JavaScript code

---

## Phase 2: Breakpoint System

**Why Second**: With hooks in place, we can now check breakpoints during execution.

### 2.1 PC-Based Breakpoint Checking

**Current State**: Breakpoints stored but never checked.

**Engine Work Required:**
```rust
// In on_step hook implementation (DebuggerHostHooks)
fn on_step(&self, context: &mut Context, frame: &CallFrame, pc: u32) -> JsResult<bool> {
    let debugger = self.debugger.lock().unwrap();
    let script_id = frame.code_block.script_id;
    
    // Check if breakpoint exists at this location
    if debugger.has_breakpoint(script_id, pc) {
        drop(debugger); // Release lock before calling user hook
        
        // Call user's breakpoint handler
        if let Some(hooks) = debugger.hooks() {
            if hooks.on_breakpoint(context, frame, breakpoint_id)? {
                return Ok(true); // Request pause
            }
        }
        
        return Ok(true); // Default: pause on breakpoint
    }
    
    // Check stepping mode
    if debugger.should_pause_for_step(frame_depth) {
        return Ok(true);
    }
    
    Ok(false)
}
```

**Tasks:**
- [ ] Implement breakpoint checking in on_step
- [ ] Ensure script_id is set in all CodeBlocks during compilation
- [ ] Test breakpoint hit detection
- [ ] Add breakpoint hit notification to DAP client

**Enables**: Basic PC-based breakpoints

---

### 2.2 Script Registry & ScriptId Management

**Current Problem**: No mapping from source files to ScriptId.

**Design:**
```rust
pub struct ScriptRegistry {
    scripts: HashMap<ScriptId, ScriptInfo>,
    next_id: AtomicU32,
    url_to_script: HashMap<String, ScriptId>,
}

pub struct ScriptInfo {
    id: ScriptId,
    source: String,
    url: Option<String>,  // File path or URL
    code_block: Gc<CodeBlock>,
    line_mapping: SourceMapping,
}

impl ScriptRegistry {
    pub fn register(&mut self, source: String, url: Option<String>, code_block: Gc<CodeBlock>) -> ScriptId;
    pub fn get_by_id(&self, id: ScriptId) -> Option<&ScriptInfo>;
    pub fn get_by_url(&self, url: &str) -> Option<&ScriptInfo>;
    pub fn all_scripts(&self) -> Vec<ScriptId>;
}
```

**Integration Points:**
```rust
// In bytecompiler when creating CodeBlock
let script_id = context.script_registry.register(
    source.to_string(),
    source_url,
    code_block_gc
);
code_block.script_id = script_id;

// Call on_new_script hook
if let Some(hooks) = &context.host_hooks {
    hooks.on_new_script(context, script_id)?;
}
```

**Tasks:**
- [ ] Design and implement ScriptRegistry
- [ ] Add registry to Context
- [ ] Integrate with bytecompiler
- [ ] Assign ScriptId during compilation
- [ ] Store CodeBlock reference
- [ ] Implement on_new_script hook calling
- [ ] Add GC marking for script references

**Enables**: Source-based breakpoint lookup

---

### 2.3 Line-to-PC Mapping

**Current Problem**: No way to map source line numbers to bytecode PC offsets.

**Design:**
```rust
pub struct SourceMapping {
    // Dense vector: index = PC, value = location
    pc_to_location: Vec<SourceLocation>,
    
    // Sparse map: line → [PCs on that line]
    line_to_pcs: HashMap<u32, Vec<u32>>,
}

#[derive(Clone, Copy)]
pub struct SourceLocation {
    pub line: u32,
    pub column: u32,
}

impl SourceMapping {
    pub fn add_mapping(&mut self, pc: u32, line: u32, column: u32);
    pub fn pc_to_line(&self, pc: u32) -> Option<u32>;
    pub fn line_to_pc(&self, line: u32) -> Option<u32>;  // First PC on line
    pub fn get_line_pcs(&self, line: u32) -> &[u32];     // All PCs on line
}
```

**Bytecompiler Enhancement:**
```rust
// In bytecompiler during code generation
impl ByteCompiler {
    fn emit_with_location(&mut self, opcode: Opcode, node: &Node) {
        let pc = self.next_instruction_offset();
        let location = node.location();  // Get from AST
        
        // Record mapping
        self.source_mapping.add_mapping(
            pc,
            location.line,
            location.column
        );
        
        self.emit(opcode);
    }
}
```

**Current Engine Issue**: AST nodes may not have accurate location info.

**Engine Fixes Needed:**
- [ ] Ensure parser preserves line/column for all nodes
- [ ] Propagate location through AST transformations
- [ ] Fix offset drift in bytecompiler
- [ ] Validate locations in test suite

**Tasks:**
- [ ] Implement SourceMapping struct
- [ ] Modify bytecompiler to generate mappings
- [ ] Store mapping in ScriptInfo
- [ ] Add API: `debugger.set_breakpoint_by_line(url, line)`
- [ ] Test line-to-PC round trip accuracy
- [ ] Add column-level precision

**Enables**: Line-based breakpoints (what IDEs use)

---

### 2.4 Conditional Breakpoints

**Current State**: Condition stored but never evaluated.

**Engine Work Required:**
```rust
// In breakpoint checking logic
if let Some(breakpoint) = debugger.get_breakpoint(script_id, pc) {
    // Check condition if present
    if let Some(condition) = &breakpoint.condition {
        let frame = DebuggerFrame::from_call_frame(frame);
        let result = frame.eval(condition, context)?;
        
        if !result.to_boolean() {
            continue; // Condition false, don't break
        }
    }
    
    // Check hit count condition
    breakpoint.actual_hit_count += 1;
    if let Some(hit_condition) = &breakpoint.hit_condition {
        if !evaluate_hit_condition(hit_condition, breakpoint.actual_hit_count) {
            continue;
        }
    }
    
    // Break!
    return Ok(true);
}
```

**Prerequisites**: Requires frame.eval() (see Phase 3)

**Tasks:**
- [ ] Implement condition evaluation
- [ ] Implement hit count checking (">N", "==N", "%N")
- [ ] Update actual_hit_count on each hit
- [ ] Test with various conditions

**Enables**: Conditional breakpoints in DAP

---

### 2.5 Logpoints

**Current State**: Log message stored but never printed.

**Engine Work Required:**
```rust
// In breakpoint checking logic
if let Some(log_message) = &breakpoint.log_message {
    let interpolated = interpolate_log_message(log_message, frame, context)?;
    
    // Send as output event via DAP
    send_output_event(&interpolated, "console");
    
    // Logpoint doesn't pause - continue execution
    continue;
}
```

**Message Interpolation:**
```rust
fn interpolate_log_message(template: &str, frame: &DebuggerFrame, context: &mut Context) -> JsResult<String> {
    // "Value is {x}" → evaluate {x} and substitute
    let mut result = template.to_string();
    
    for capture in EXPR_REGEX.captures_iter(template) {
        let expr = &capture[1];
        let value = frame.eval(expr, context)?;
        let formatted = format_value(&value);
        result = result.replace(&format!("{{{}}}", expr), &formatted);
    }
    
    Ok(result)
}
```

**Tasks:**
- [ ] Implement message interpolation
- [ ] Support expression evaluation in {braces}
- [ ] Send output via DAP
- [ ] Test with various log messages

**Enables**: Logpoints (breakpoints that log without pausing)

---

## Phase 3: Call Stack & Frame Introspection

**Why Third**: Needed for stackTrace, scopes, variables DAP commands.

### 3.1 DebuggerFrame Implementation

**Current State**: Empty struct with placeholders.

**Required API:**
```rust
impl DebuggerFrame {
    // Frame identification
    pub fn id(&self) -> FrameId;
    pub fn function_name(&self) -> String;
    pub fn script_id(&self) -> ScriptId;
    pub fn pc(&self) -> u32;
    pub fn line(&self) -> u32;
    pub fn column(&self) -> u32;
    
    // Environment access
    pub fn eval(&self, code: &str, context: &mut Context) -> JsResult<JsValue>;
    pub fn this(&self) -> JsValue;
    pub fn locals(&self) -> HashMap<String, JsValue>;
    pub fn arguments(&self) -> Vec<JsValue>;
    
    // Scope chain
    pub fn environment(&self) -> &DeclarativeEnvironment;
    pub fn global_object(&self) -> JsObject;
}
```

**Engine Challenge**: Safe access to frame's environment.

**Solution:**
```rust
// Store frame reference/index instead of raw pointer
pub struct DebuggerFrame {
    frame_index: usize,  // Index in context.vm.frames
    snapshot: Option<FrameSnapshot>,  // Cached data
}

impl DebuggerFrame {
    // Accessing requires Context
    pub fn eval(&self, code: &str, context: &mut Context) -> JsResult<JsValue> {
        let frame = context.vm.frames.get(self.frame_index)?;
        let env = &frame.env;
        
        // Compile and eval in frame's environment
        let compiled = context.compile_in_scope(code, env)?;
        context.eval_compiled(compiled)
    }
}
```

**Tasks:**
- [ ] Design safe frame access mechanism
- [ ] Implement frame.eval() with proper scope
- [ ] Implement this binding access
- [ ] Implement local variable enumeration
- [ ] Implement argument access
- [ ] Add tests for nested scopes
- [ ] Add tests for closures

**Enables**: Variable inspection, expression evaluation

---

### 3.2 Call Stack Introspection

**Required for**: stackTrace DAP command

**API Design:**
```rust
impl Debugger {
    pub fn get_stack_frames(&self, context: &Context) -> Vec<DebuggerFrame>;
    pub fn get_frame(&self, frame_id: FrameId, context: &Context) -> Option<DebuggerFrame>;
}
```

**Engine Enhancement:**
```rust
// Ensure frames have all needed metadata
pub struct CallFrame {
    // Existing fields...
    
    // Add if missing:
    pub function_name: Option<JsString>,
    pub script_id: ScriptId,
    pub this_binding: JsValue,
}
```

**Tasks:**
- [ ] Implement stack frame enumeration
- [ ] Add frame ID management
- [ ] Include function names in frames
- [ ] Map PCs to line numbers
- [ ] Test with recursive calls
- [ ] Test with async functions

**Enables**: Call stack view in IDE

---

## Phase 4: DAP Command Implementation

**Strategy**: Implement commands one by one, each command fully functional before moving to next.

### 4.1 setBreakpoints Command

**Current State**: Stores breakpoints but doesn't work.

**Full Implementation:**
```rust
fn handle_set_breakpoints(&mut self, args: SetBreakpointsArguments) -> DapResult {
    let source_path = args.source.path.ok_or("Missing source path")?;
    
    // Look up script by path
    let script_id = self.session.script_registry
        .get_by_url(&source_path)
        .ok_or("Script not found")?;
    
    // Clear existing breakpoints for this source
    self.debugger.clear_breakpoints_for_script(script_id);
    
    // Set new breakpoints
    let mut breakpoints = Vec::new();
    for bp_req in args.breakpoints {
        // Translate line to PC
        let pc = self.session.line_to_pc(script_id, bp_req.line)?;
        
        // Create breakpoint
        let bp_id = self.debugger.set_breakpoint(script_id, pc);
        
        // Add condition if present
        if let Some(condition) = bp_req.condition {
            self.debugger.set_breakpoint_condition(bp_id, condition);
        }
        
        // Add log message if present
        if let Some(log_message) = bp_req.log_message {
            self.debugger.set_breakpoint_log_message(bp_id, log_message);
        }
        
        breakpoints.push(Breakpoint {
            id: Some(bp_id.0 as i64),
            verified: true,
            line: Some(bp_req.line),
            source: Some(args.source.clone()),
            ..Default::default()
        });
    }
    
    Ok(SetBreakpointsResponse { breakpoints })
}
```

**Prerequisites:**
- [x] PC-based breakpoints working
- [ ] Script registry (Phase 2.2)
- [ ] Line-to-PC mapping (Phase 2.3)

**Tasks:**
- [ ] Implement setBreakpoints handler
- [ ] Handle source path lookup
- [ ] Translate lines to PCs
- [ ] Support conditions and logpoints
- [ ] Return verified breakpoints
- [ ] Handle breakpoint resolution failures gracefully

**Test Cases:**
- [ ] Set breakpoint on valid line
- [ ] Set breakpoint on invalid line (no code)
- [ ] Set multiple breakpoints
- [ ] Update existing breakpoints
- [ ] Clear all breakpoints

---

### 4.2 stackTrace Command

**Functionality**: Return call stack with file names, line numbers.

**Implementation:**
```rust
fn handle_stack_trace(&mut self, args: StackTraceArguments) -> DapResult {
    let frames = self.debugger.get_stack_frames(&self.context);
    
    let dap_frames: Vec<StackFrame> = frames.iter().enumerate()
        .skip(args.start_frame.unwrap_or(0) as usize)
        .take(args.levels.unwrap_or(frames.len()) as usize)
        .map(|(index, frame)| {
            let script = self.session.script_registry.get_by_id(frame.script_id()).unwrap();
            
            StackFrame {
                id: index as i64,
                name: frame.function_name(),
                source: Some(Source {
                    path: script.url.clone(),
                    ..Default::default()
                }),
                line: frame.line() as i64,
                column: frame.column() as i64,
                module_id: None,
                presentation_hint: Some("normal".to_string()),
            }
        })
        .collect();
    
    Ok(StackTraceResponse {
        stack_frames: dap_frames,
        total_frames: Some(frames.len() as i64),
    })
}
```

**Prerequisites:**
- [ ] Frame introspection (Phase 3.2)
- [ ] Script registry (Phase 2.2)
- [ ] PC-to-line mapping (Phase 2.3)

**Tasks:**
- [ ] Implement stackTrace handler
- [ ] Map frames to DAP format
- [ ] Include source information
- [ ] Support pagination (start_frame, levels)
- [ ] Test with deep call stacks

---

### 4.3 scopes Command

**Functionality**: Return variable scopes for a frame (Local, Global, etc.).

**Implementation:**
```rust
fn handle_scopes(&mut self, args: ScopesArguments) -> DapResult {
    let frame = self.debugger.get_frame(args.frame_id as usize, &self.context)
        .ok_or("Invalid frame")?;
    
    let scopes = vec![
        Scope {
            name: "Local".to_string(),
            presentation_hint: Some("locals".to_string()),
            variables_reference: self.create_variable_reference(
                VariableContainer::Locals(args.frame_id)
            ),
            named_variables: Some(frame.locals().len() as i64),
            expensive: false,
        },
        Scope {
            name: "Global".to_string(),
            presentation_hint: Some("globals".to_string()),
            variables_reference: self.create_variable_reference(
                VariableContainer::GlobalObject
            ),
            expensive: true,
        },
    ];
    
    Ok(ScopesResponse { scopes })
}
```

**Variable Reference Management:**
```rust
struct VariableReferenceManager {
    next_ref: AtomicI64,
    references: HashMap<i64, VariableContainer>,
}

enum VariableContainer {
    Locals(FrameId),
    GlobalObject,
    Object(JsObject),
    Array(JsObject),
}
```

**Prerequisites:**
- [ ] Frame.locals() (Phase 3.1)

**Tasks:**
- [ ] Implement scopes handler
- [ ] Create variable reference system
- [ ] Support Local scope
- [ ] Support Global scope
- [ ] Add Closure scope (future)
- [ ] Add Block scope (future)

---

### 4.4 variables Command

**Functionality**: Return variables in a scope.

**Implementation:**
```rust
fn handle_variables(&mut self, args: VariablesArguments) -> DapResult {
    let container = self.variable_refs.get(args.variables_reference)
        .ok_or("Invalid reference")?;
    
    let variables = match container {
        VariableContainer::Locals(frame_id) => {
            let frame = self.debugger.get_frame(*frame_id, &self.context)?;
            frame.locals().iter().map(|(name, value)| {
                self.js_value_to_dap_variable(name, value)
            }).collect()
        }
        VariableContainer::Object(obj) => {
            obj.own_properties().iter().map(|(name, prop)| {
                self.js_value_to_dap_variable(name, &prop.value())
            }).collect()
        }
        VariableContainer::GlobalObject => {
            let global = self.context.global_object();
            // ... similar to Object case
        }
    };
    
    Ok(VariablesResponse { variables })
}

fn js_value_to_dap_variable(&mut self, name: &str, value: &JsValue) -> Variable {
    Variable {
        name: name.to_string(),
        value: self.format_value(value),
        type_: Some(self.value_type(value)),
        variables_reference: if value.is_object() {
            self.create_variable_reference(VariableContainer::Object(value.as_object().unwrap()))
        } else {
            0
        },
        indexed_variables: self.get_indexed_count(value),
        named_variables: self.get_named_count(value),
        presentation_hint: self.get_presentation_hint(value),
    }
}
```

**Value Formatting:**
```rust
fn format_value(&self, value: &JsValue) -> String {
    match value {
        JsValue::Undefined => "undefined".to_string(),
        JsValue::Null => "null".to_string(),
        JsValue::Boolean(b) => b.to_string(),
        JsValue::String(s) => format!("\"{}\"", s),
        JsValue::Number(n) => n.to_string(),
        JsValue::BigInt(bi) => format!("{}n", bi),
        JsValue::Object(obj) => {
            if obj.is_array() {
                format!("Array({})", obj.length().unwrap_or(0))
            } else if obj.is_function() {
                format!("Function {}", obj.get_function_name())
            } else {
                format!("Object {{ ... }}")
            }
        }
        JsValue::Symbol(s) => format!("Symbol({})", s.description()),
    }
}
```

**Prerequisites:**
- [ ] Frame.locals() (Phase 3.1)
- [ ] Object property enumeration

**Tasks:**
- [ ] Implement variables handler
- [ ] Resolve variable references
- [ ] Format values appropriately
- [ ] Support nested objects
- [ ] Support arrays with indexedVariables
- [ ] Support getters/setters (future)
- [ ] Handle circular references

---

### 4.5 evaluate Command

**Functionality**: Evaluate expression in frame context or global context.

**Implementation:**
```rust
fn handle_evaluate(&mut self, args: EvaluateArguments) -> DapResult {
    let result = match args.frame_id {
        Some(frame_id) => {
            // Evaluate in frame context
            let frame = self.debugger.get_frame(frame_id as usize, &self.context)?;
            frame.eval(&args.expression, &mut self.context)?
        }
        None => {
            // Evaluate in global context
            self.context.eval(Source::from_bytes(&args.expression))?
        }
    };
    
    Ok(EvaluateResponse {
        result: self.format_value(&result),
        type_: Some(self.value_type(&result)),
        variables_reference: if result.is_object() {
            self.create_variable_reference(
                VariableContainer::Object(result.as_object().unwrap())
            )
        } else {
            0
        },
        indexed_variables: self.get_indexed_count(&result),
        named_variables: self.get_named_count(&result),
        presentation_hint: self.get_presentation_hint(&result),
    })
}
```

**Contexts:**
- **watch**: Evaluate watch expression (run on every pause)
- **repl**: Evaluate in debug console
- **hover**: Evaluate for hover tooltip
- **clipboard**: Evaluate for copy value

**Prerequisites:**
- [ ] Frame.eval() (Phase 3.1)

**Tasks:**
- [ ] Implement evaluate handler
- [ ] Support frame context evaluation
- [ ] Support global context evaluation
- [ ] Handle evaluation errors gracefully
- [ ] Support different contexts (watch, repl, hover)
- [ ] Add side-effect detection (future)

---

### 4.6 continue Command

**Current State**: Infrastructure exists, needs testing.

**Implementation:**
```rust
fn handle_continue(&mut self, args: ContinueArguments) -> DapResult {
    self.debugger.resume();
    self.condvar.notify_all();
    
    Ok(ContinueResponse {
        all_threads_continued: true,
    })
}
```

**Tasks:**
- [ ] Verify continue works after breakpoint
- [ ] Test with multiple threads (future)
- [ ] Ensure proper event sequencing

---

### 4.7 next/stepIn/stepOut Commands

**Current State**: API exists but no frame depth tracking.

**Implementation:**
```rust
fn handle_next(&mut self, args: NextArguments) -> DapResult {
    let current_depth = self.context.vm.frames.len();
    self.debugger.step_over(current_depth);
    self.condvar.notify_all();
    Ok(())
}

fn handle_step_in(&mut self, args: StepInArguments) -> DapResult {
    self.debugger.step_in();
    self.condvar.notify_all();
    Ok(())
}

fn handle_step_out(&mut self, args: StepOutArguments) -> DapResult {
    let current_depth = self.context.vm.frames.len();
    self.debugger.step_out(current_depth);
    self.condvar.notify_all();
    Ok(())
}
```

**Prerequisites:**
- [ ] on_enter_frame/on_exit_frame hooks (Phase 1.2)
- [ ] Frame depth tracking

**Tasks:**
- [ ] Implement next handler
- [ ] Implement stepIn handler
- [ ] Implement stepOut handler
- [ ] Test with nested function calls
- [ ] Handle edge cases (stepping into native code)

---

### 4.8 pause Command

**Current State**: Works, needs verification.

**Implementation:**
```rust
fn handle_pause(&mut self, args: PauseArguments) -> DapResult {
    self.debugger.pause();
    // Note: Pause takes effect at next on_step() call
    Ok(())
}
```

**Tasks:**
- [ ] Verify pause works during execution
- [ ] Ensure "stopped" event sent with reason="pause"
- [ ] Test timing (pause may not be immediate)

---

### 4.9 threads Command

**Current State**: Returns dummy thread, needs real implementation.

**Implementation:**
```rust
fn handle_threads(&mut self) -> DapResult {
    // Currently single-threaded
    let threads = vec![
        Thread {
            id: 1,
            name: "JavaScript".to_string(),
        }
    ];
    
    Ok(ThreadsResponse { threads })
}
```

**Future Enhancement**: Support Web Workers/parallel execution.

**Tasks:**
- [ ] Document single-threaded behavior
- [ ] Plan for future multi-threading

---

### 4.10 exceptionInfo Command

**Functionality**: Get details about caught exception.

**Implementation:**
```rust
fn handle_exception_info(&mut self, args: ExceptionInfoArguments) -> DapResult {
    let exception = self.debugger.current_exception()
        .ok_or("No exception")?;
    
    Ok(ExceptionInfoResponse {
        exception_id: exception.name(),
        description: Some(exception.message()),
        break_mode: ExceptionBreakMode::Always,
        details: Some(ExceptionDetails {
            message: Some(exception.message()),
            type_name: Some(exception.name()),
            stack_trace: Some(exception.stack_trace()),
            ..Default::default()
        }),
    })
}
```

**Prerequisites:**
- [ ] Exception tracking in debugger
- [ ] on_exception_unwind hook

**Tasks:**
- [ ] Store current exception in debugger
- [ ] Implement exceptionInfo handler
- [ ] Extract exception details
- [ ] Format stack trace

---

### 4.11 setExceptionBreakpoints Command

**Functionality**: Configure when to break on exceptions.

**Implementation:**
```rust
fn handle_set_exception_breakpoints(&mut self, args: SetExceptionBreakpointsArguments) -> DapResult {
    for filter in args.filters {
        match filter.as_str() {
            "all" => self.debugger.set_exception_break_mode(ExceptionBreakMode::All),
            "uncaught" => self.debugger.set_exception_break_mode(ExceptionBreakMode::Uncaught),
            _ => {}
        }
    }
    
    Ok(SetExceptionBreakpointsResponse {
        breakpoints: vec![],
    })
}
```

**Engine Work Required:**
```rust
// In exception handling code
if let Err(exception) = result {
    let mode = debugger.exception_break_mode();
    let should_break = match mode {
        ExceptionBreakMode::All => true,
        ExceptionBreakMode::Uncaught => !is_caught,
        ExceptionBreakMode::Never => false,
    };
    
    if should_break {
        debugger.set_current_exception(exception.clone());
        debugger.pause();
        wait_while_paused();
    }
    
    return Err(exception);
}
```

**Tasks:**
- [ ] Add exception break mode to debugger
- [ ] Detect caught vs uncaught exceptions
- [ ] Implement setExceptionBreakpoints handler
- [ ] Store current exception for exceptionInfo
- [ ] Test with try/catch blocks

---

### 4.12 source Command

**Functionality**: Return source code for a script.

**Implementation:**
```rust
fn handle_source(&mut self, args: SourceArguments) -> DapResult {
    let source_ref = args.source_reference.ok_or("Missing source reference")?;
    let script = self.session.script_registry.get_by_id(ScriptId(source_ref as u32))
        .ok_or("Script not found")?;
    
    Ok(SourceResponse {
        content: script.source.clone(),
        mime_type: Some("text/javascript".to_string()),
    })
}
```

**Prerequisites:**
- [ ] Script registry (Phase 2.2)

**Tasks:**
- [ ] Implement source handler
- [ ] Store source in registry
- [ ] Handle source references
- [ ] Support dynamically generated code

---

### 4.13 completions Command (Future)

**Functionality**: Auto-complete in debug console.

**Implementation**: Requires JavaScript parser/analyzer integration.

**Tasks:**
- [ ] Enumerate available variables in scope
- [ ] Enumerate object properties
- [ ] Handle partial expressions
- [ ] Return sorted completions

---

### 4.14 setVariable Command (Future)

**Functionality**: Modify variable value during debugging.

**Implementation:**
```rust
fn handle_set_variable(&mut self, args: SetVariableArguments) -> DapResult {
    let container = self.variable_refs.get(args.variables_reference)?;
    
    match container {
        VariableContainer::Locals(frame_id) => {
            let frame = self.debugger.get_frame(*frame_id, &mut self.context)?;
            let new_value = self.parse_value(&args.value)?;
            frame.set_local(&args.name, new_value)?;
        }
        VariableContainer::Object(obj) => {
            let new_value = self.parse_value(&args.value)?;
            obj.set(&args.name, new_value, true, &mut self.context)?;
        }
        _ => return Err("Cannot set variable in this scope".into()),
    }
    
    Ok(SetVariableResponse {
        value: args.value,
        type_: None,
        variables_reference: 0,
    })
}
```

**Prerequisites:**
- [ ] Frame.set_local() method
- [ ] Value parsing from string

---

### 4.15 restartFrame Command (Future)

**Functionality**: Restart execution from a specific frame.

**Complexity**: HIGH - requires unwinding and rewinding stack.

---

## Phase 5: Engine Enhancements

### 5.1 Accurate Source Location Tracking

**Current Problem**: Line offsets may be inaccurate, especially with:
- Multi-line expressions
- Template literals
- Comments
- Destructuring assignments

**Required Fixes:**
```rust
// In parser - ensure all nodes have accurate Span
impl Parser {
    fn parse_expression(&mut self) -> Node {
        let start = self.current_token().span().start();
        let expr = self.parse_expr_inner();
        let end = self.current_token().span().end();
        
        expr.with_span(Span::new(start, end))
    }
}

// In bytecompiler - preserve spans through compilation
impl ByteCompiler {
    fn compile_node(&mut self, node: &Node) {
        let span = node.span();
        
        // All emitted instructions get this span
        self.current_span = Some(span);
        
        node.compile(self);
        
        self.current_span = None;
    }
}
```

**Test Strategy:**
- [ ] Create test suite with complex multi-line code
- [ ] Verify each line maps to correct PC
- [ ] Test edge cases (comments, strings, etc.)
- [ ] Add regression tests

---

### 5.2 Function Name Preservation

**Current Problem**: Anonymous functions lose their inferred names.

**Fix:**
```rust
// Store function name in CallFrame
pub struct CallFrame {
    pub function_name: Option<JsString>,
    // ... other fields
}

// Infer names during compilation
let inferred_name = match context {
    Assignment => lhs_name,
    PropertyValue => property_key,
    ArrayElement => None,
};

code_block.function_name = inferred_name;
```

---

### 5.3 this Binding Access

**Current Problem**: Can't access `this` value from debugger.

**Fix:**
```rust
pub struct CallFrame {
    pub this_binding: JsValue,
    // ... other fields
}

// Set when creating frame
let frame = CallFrame {
    this_binding: this.clone(),
    // ... other fields
};
```

---

### 5.4 Environment Chain Access

**Current Problem**: Can't traverse scope chain for closure inspection.

**Fix:**
```rust
impl DebuggerFrame {
    pub fn outer_environments(&self, context: &Context) -> Vec<Environment> {
        let mut envs = Vec::new();
        let mut current = self.environment(context);
        
        while let Some(outer) = current.outer() {
            envs.push(outer.clone());
            current = outer;
        }
        
        envs
    }
}
```

---

## Phase 6: Advanced Features

### 6.1 Watch Expressions

**Design:**
```rust
pub struct WatchExpression {
    id: WatchId,
    expression: String,
    last_value: Option<JsValue>,
}

impl Debugger {
    pub fn add_watch(&mut self, expr: String) -> WatchId;
    pub fn remove_watch(&mut self, id: WatchId);
    
    // Called automatically on every pause
    fn evaluate_watches(&mut self, context: &mut Context) {
        for watch in &mut self.watches {
            let new_value = context.eval(Source::from_bytes(&watch.expression));
            
            if new_value != watch.last_value {
                notify_watch_changed(watch.id, &new_value);
                watch.last_value = Some(new_value);
            }
        }
    }
}
```

---

### 6.2 Hot Reload / Edit and Continue

**Complexity**: VERY HIGH

**Requirements:**
- Replace code in running function
- Update breakpoint PCs
- Preserve variable state
- Handle signature changes

**Phased Approach:**
1. Module-level reload (easier)
2. Function-level reload (harder)
3. Mid-execution reload (very hard)

---

### 6.3 Async Stack Traces

**Requirements:**
- Track promise creation sites
- Link promise chains
- Show async stack in stackTrace

**Design:**
```rust
pub struct AsyncStackTrace {
    description: String,
    frames: Vec<DebuggerFrame>,
    parent: Option<Box<AsyncStackTrace>>,
}
```

---

## Testing Strategy

### Unit Tests
- [ ] Breakpoint hit detection
- [ ] Stepping logic (all modes)
- [ ] Condition evaluation
- [ ] Variable formatting
- [ ] Line-to-PC mapping

### Integration Tests
- [ ] Full DAP message sequence
- [ ] Multiple breakpoints
- [ ] Nested function calls
- [ ] Exception handling
- [ ] Complex expressions

### End-to-End Tests
- [ ] VS Code integration
- [ ] Real-world scripts
- [ ] Performance benchmarks
- [ ] Memory leak detection

---

## Success Criteria

### Core Functionality
- [ ] All SpiderMonkey Debugger API features implemented
- [ ] All essential DAP commands functional
- [ ] Accurate source mapping
- [ ] Reliable breakpoint hits
- [ ] Complete variable inspection

### Performance
- [ ] <5% overhead with debugging disabled
- [ ] <20% overhead with debugging enabled, no breakpoints
- [ ] <50% overhead with active breakpoints and stepping

### Quality
- [ ] 80%+ test coverage
- [ ] No memory leaks
- [ ] No crashes/panics
- [ ] Comprehensive documentation

### User Experience
- [ ] Full VS Code debugging workflow
- [ ] Responsive stepping (no lag)
- [ ] Accurate error messages
- [ ] Helpful DAP event logging

---

## Resources

- **SpiderMonkey Debugger**: https://firefox-source-docs.mozilla.org/devtools/debugger-api/
- **DAP Specification**: https://microsoft.github.io/debug-adapter-protocol/
- **Reference Implementation**: cli/src/debug/dap.rs

---

**Maintained By**: Boa Core Team  
**Status**: Foundation complete, actively developing Phase 1  
**Last Updated**: January 2026

### ✅ Phase 1: Core Functionality (COMPLETE)

**Milestone 1: Basic Infrastructure** ✅
- [x] Debugger struct with state management
- [x] Breakpoint data structures (with conditions, hit counts, logpoints)
- [x] Stepping state machine (StepIn, StepOver, StepOut, StepToFrame)
- [x] Pause/resume mechanism with condition variables
- [x] Thread-safe design (Arc<Mutex<>>)
- [x] Attach/detach from contexts
- [x] DebuggerHooks trait definition
- [x] HostHooks trait extension

**Milestone 2: VM Integration** ✅
- [x] on_step hook call in VM execution loop (vm/mod.rs:863)
- [x] on_debugger_statement for `debugger;` statements
- [x] on_exception_unwind for error handling
- [x] DebuggerHostHooks implementation
- [x] CodeBlock.script_id field
- [x] **Breakpoint checking fully integrated**
- [x] Efficient waiting with condition variables

**Milestone 3: DAP Protocol** ✅
- [x] Complete DAP message types (30+ types)
- [x] JSON-RPC server with stdio transport
- [x] DapServer with request routing
- [x] DebugSession integration layer
- [x] CLI integration (--dap flag)
- [x] Basic command handlers

**Milestone 4: Examples & Documentation** ✅
- [x] debugger_pause_resume.rs example
- [x] debugger_breakpoints.rs example
- [x] Comprehensive documentation
- [x] SpiderMonkey comparison
- [x] Architecture diagrams

**Achievement**: Core debugging workflow fully functional!

---

## 🔄 Phase 2: VM Integration Completion (IN PROGRESS)

**Priority**: HIGH  
**Complexity**: Medium  
**ETA**: 1-2 weeks

### Milestone 5: Frame Enter/Exit Hooks

**Current State**:
- ✅ Hooks defined in HostHooks trait
- ❌ NOT called from VM
- **Blocker**: Borrowing challenges - push_frame() is called on context.vm

**Tasks**:
- [ ] Call on_enter_frame() when pushing frames
- [ ] Call on_exit_frame() when popping frames
- [ ] Resolve borrowing conflicts (3 possible approaches):
  - Option A: Add Context wrapper methods
  - Option B: Refactor all call sites (8 locations)
  - Option C: Pass host_hooks reference to push/pop methods

**Call Sites to Update**:
```
core/engine/src/script.rs:214
core/engine/src/builtins/function/mod.rs:1024, 1145
core/engine/src/builtins/generator/mod.rs:99
core/engine/src/builtins/json/mod.rs:142
core/engine/src/builtins/eval/mod.rs:333
core/engine/src/object/builtins/jspromise.rs:1231, 1288
```

**Impact**: Full frame lifecycle tracking for advanced debugging

**Tests Needed**:
- Frame enter/exit hook invocation
- Frame depth tracking accuracy
- Performance impact measurement

---

## 📋 Phase 3: Script Management (NOT STARTED)

**Priority**: HIGH (required for DAP)  
**Complexity**: Medium  
**ETA**: 2-3 weeks

### Milestone 6: Script Registry

**Current State**: No script tracking exists

**Goals**:
- Assign unique ScriptId during compilation
- Store script metadata (source, path, name)
- Map ScriptId ↔ source code
- Enable script querying by URL/name

**Implementation**:

```rust
pub struct ScriptRegistry {
    scripts: HashMap<ScriptId, ScriptInfo>,
    next_id: AtomicUsize,
    url_index: HashMap<String, Vec<ScriptId>>,
}

pub struct ScriptInfo {
    id: ScriptId,
    source: String,
    path: Option<String>,
    name: Option<String>,
    code_block: Gc<CodeBlock>,
    parent: Option<ScriptId>,
}

impl ScriptRegistry {
    pub fn register(&mut self, source: &str, path: Option<String>) -> ScriptId;
    pub fn get(&self, id: ScriptId) -> Option<&ScriptInfo>;
    pub fn find_by_url(&self, url: &str) -> Vec<ScriptId>;
}
```

**Integration Points**:
- Bytecompiler: Assign ScriptId during compilation
- Context: Store ScriptRegistry
- Debugger: Query scripts for breakpoint resolution
- DAP: Map source references to ScriptId

**Tasks**:
- [ ] Design ScriptRegistry API
- [ ] Integrate with bytecompiler
- [ ] Add to Context
- [ ] Implement query methods
- [ ] Add GC support for script references
- [ ] Call on_new_script() hook

**Impact**: Enables source-based debugging features

---

### Milestone 7: Line-to-PC Mapping

**Current State**: CodeBlock has source info, but no mapping

**Goals**:
- Build bidirectional line ↔ PC mapping
- Support column-level precision
- Handle source maps (future)

**Implementation**:

```rust
pub struct SourceMapping {
    // PC → (line, column)
    pc_to_location: Vec<SourceLocation>,
    // Line → [PC]
    line_to_pcs: HashMap<u32, Vec<u32>>,
}

impl SourceMapping {
    pub fn pc_to_line(&self, pc: u32) -> Option<u32>;
    pub fn line_to_pc(&self, line: u32) -> Option<u32>;
    pub fn get_line_pcs(&self, line: u32) -> &[u32];
}
```

**Integration**:
- Generate mapping during bytecompiler pass
- Store in CodeBlock or ScriptInfo
- Use for breakpoint translation in DAP

**API**:
```rust
// User-friendly breakpoint API
debugger.set_breakpoint_by_line("script.js", 42)?;

// DAP setBreakpoints handler
for bp in breakpoints {
    let pc = mapping.line_to_pc(bp.line)?;
    debugger.set_breakpoint(script_id, pc);
}
```

**Tasks**:
- [ ] Design SourceMapping struct
- [ ] Generate mapping in bytecompiler
- [ ] Store mapping with scripts
- [ ] Implement set_breakpoint_by_line()
- [ ] Update DAP setBreakpoints handler
- [ ] Add column support

**Impact**: User-friendly line-based breakpoints for DAP

---

## 🔧 Phase 4: Reflection Objects (NOT STARTED)

**Priority**: MEDIUM  
**Complexity**: HIGH  
**ETA**: 3-4 weeks

### Milestone 8: DebuggerFrame Implementation

**Current State**: Stub with basic info

**Goals**:
- Evaluate expressions in frame context
- Access local variables
- Traverse scope chain
- Inspect closure variables

**Implementation**:

```rust
impl DebuggerFrame {
    // Already implemented
    pub fn position(&self) -> SourceLocation { /* ... */ }
    pub fn pc(&self) -> u32 { /* ... */ }
    
    // Need to implement
    pub fn eval(&self, code: &str, context: &mut Context) -> JsResult<JsValue>;
    pub fn get_local(&self, name: &str, context: &Context) -> Option<JsValue>;
    pub fn locals(&self, context: &Context) -> Vec<(String, JsValue)>;
    pub fn this(&self, context: &Context) -> JsValue;
    pub fn environment(&self, context: &Context) -> &DeclarativeEnvironment;
}
```

**Challenges**:
- Need safe access to frame's environment
- Eval must run in frame's scope
- Proper handling of this binding
- Closure variable access

**Tasks**:
- [ ] Implement frame.eval() with expression evaluation
- [ ] Add environment/scope access methods
- [ ] Implement local variable enumeration
- [ ] Add this binding access
- [ ] Handle closure variable inspection
- [ ] Add parent frame traversal

**Impact**: Variable inspection in DAP

---

### Milestone 9: DebuggerScript & DebuggerObject

**DebuggerScript Goals**:
```rust
impl DebuggerScript {
    pub fn source(&self) -> &str;
    pub fn start_line(&self) -> u32;
    pub fn line_count(&self) -> u32;
    pub fn get_child_scripts(&self) -> Vec<DebuggerScript>;
    pub fn line_to_pc(&self, line: u32) -> Option<u32>;
    pub fn pc_to_line(&self, pc: u32) -> Option<u32>;
}
```

**DebuggerObject Goals**:
```rust
impl DebuggerObject {
    pub fn class(&self) -> &str;
    pub fn prototype(&self) -> Option<DebuggerObject>;
    pub fn own_property_names(&self) -> Vec<String>;
    pub fn get_property(&self, name: &str) -> Option<JsValue>;
    pub fn call(&mut self, this: &JsValue, args: &[JsValue]) -> JsResult<JsValue>;
}
```

**Tasks**:
- [ ] Implement DebuggerScript methods
- [ ] Implement DebuggerObject methods
- [ ] Add safe cross-compartment access
- [ ] Handle property descriptors
- [ ] Add callable object support

**Impact**: Full object inspection in debugger

---

## 🎨 Phase 5: DAP Command Completion (PARTIAL)

**Priority**: HIGH (for IDE support)  
**Complexity**: Medium  
**ETA**: 2-3 weeks

### Milestone 10: Essential DAP Commands

**Current State**: Basic commands work, advanced need implementation

**stackTrace Command**:
```rust
fn handle_stack_trace(&mut self, context: &Context) -> JsResult<StackTraceResponseBody> {
    let stack = DebugApi::get_call_stack(context);
    let frames = stack.iter().enumerate().map(|(i, frame)| {
        StackFrame {
            id: i as i64,
            name: frame.function_name(),
            source: Some(Source {
                path: Some(frame.path()),
                ..Default::default()
            }),
            line: frame.line_number().unwrap_or(0),
            column: frame.column_number().unwrap_or(0),
        }
    }).collect();
    
    Ok(StackTraceResponseBody { stack_frames: frames, total_frames: Some(frames.len() as i64) })
}
```

**scopes Command**:
```rust
fn handle_scopes(&mut self, frame_id: i64, context: &Context) -> JsResult<ScopesResponseBody> {
    let frame = self.get_frame(frame_id, context)?;
    let scopes = vec![
        Scope {
            name: "Local".to_string(),
            variables_reference: self.create_variable_reference(frame),
            expensive: false,
        },
        Scope {
            name: "Global".to_string(),
            variables_reference: self.create_variable_reference(&context.global_object()),
            expensive: false,
        },
    ];
    Ok(ScopesResponseBody { scopes })
}
```

**variables Command**:
```rust
fn handle_variables(&mut self, var_ref: i64) -> JsResult<VariablesResponseBody> {
    let object = self.resolve_variable_reference(var_ref)?;
    let variables = object.own_property_names().iter().map(|name| {
        let value = object.get_property(name).unwrap();
        Variable {
            name: name.clone(),
            value: format_value(&value),
            type_: Some(value_type(&value)),
            variables_reference: if is_object(&value) {
                self.create_variable_reference(&value)
            } else {
                0
            },
        }
    }).collect();
    
    Ok(VariablesResponseBody { variables })
}
```

**evaluate Command**:
```rust
fn handle_evaluate(&mut self, expr: &str, frame_id: Option<i64>, context: &mut Context) -> JsResult<EvaluateResponseBody> {
    let result = if let Some(fid) = frame_id {
        let frame = self.get_frame(fid, context)?;
        frame.eval(expr, context)?
    } else {
        context.eval(Source::from_bytes(expr))?
    };
    
    Ok(EvaluateResponseBody {
        result: format_value(&result),
        type_: Some(value_type(&result)),
        variables_reference: if is_object(&result) {
            self.create_variable_reference(&result)
        } else {
            0
        },
    })
}
```

**Tasks**:
- [ ] Implement stackTrace with frame introspection
- [ ] Implement scopes with environment access
- [ ] Implement variables with property enumeration
- [ ] Implement evaluate with frame.eval()
- [ ] Add variable reference management
- [ ] Handle complex objects (arrays, functions, etc.)
- [ ] Add value formatting
- [ ] Support hover evaluation

**Impact**: Full VS Code debugging experience

---

## 🚀 Phase 6: Advanced Features (FUTURE)

**Priority**: LOW  
**Complexity**: HIGH  
**ETA**: TBD

### Milestone 11: Conditional Breakpoints & Logpoints

**Goals**:
- Evaluate breakpoint conditions
- Interpolate logpoint messages
- Increment hit counts on actual hits

**Implementation**:
```rust
// In DebuggerHostHooks::on_step()
if let Some(bp) = debugger.get_breakpoint(script_id, pc) {
    bp.increment_hit_count();
    
    if let Some(condition) = &bp.condition {
        let frame = context.vm.frame()?;
        let result = frame.eval(condition, context)?;
        if !result.as_boolean() {
            continue; // Condition false, don't pause
        }
    }
    
    if let Some(log_msg) = &bp.log_message {
        let message = interpolate_message(log_msg, &frame, context);
        println!("{}", message);
        continue; // Logpoint doesn't pause
    }
    
    // Pause for normal breakpoint
    debugger.pause();
}
```

**Tasks**:
- [ ] Implement condition evaluation
- [ ] Implement message interpolation
- [ ] Add hit count tracking in checking logic
- [ ] Support hit count conditions (e.g., ">5", "==3", "%2")
- [ ] Add DAP integration

---

### Milestone 12: Exception Breakpoints

**Goals**:
- Break on thrown exceptions
- Break on uncaught exceptions
- Filter by exception type

**Implementation**:
```rust
pub enum ExceptionBreakMode {
    Never,
    All,
    Uncaught,
    UserUncaught, // Ignore internal errors
}

impl Debugger {
    pub fn set_exception_break_mode(&mut self, mode: ExceptionBreakMode);
}

// In vm/mod.rs exception handler
if let Err(err) = result {
    if should_break_on_exception(&err, is_caught) {
        debugger.pause();
        wait_while_paused();
    }
    return Err(err);
}
```

**Tasks**:
- [ ] Add exception breakpoint settings
- [ ] Detect caught vs uncaught exceptions
- [ ] Filter by exception type/message
- [ ] Add DAP exceptionBreakpointsFilter
- [ ] Test with various error types

---

### Milestone 13: Watch Expressions

**Goals**:
- Evaluate expressions on every pause
- Track expression value changes
- Support in DAP

**Implementation**:
```rust
pub struct WatchExpression {
    id: WatchId,
    expression: String,
    last_value: Option<JsValue>,
}

impl Debugger {
    pub fn add_watch(&mut self, expr: String) -> WatchId;
    pub fn remove_watch(&mut self, id: WatchId);
    pub fn evaluate_watches(&mut self, context: &mut Context) -> Vec<WatchResult>;
}

// On every pause
fn on_pause(&mut self, context: &mut Context) {
    for watch in self.watches() {
        let value = watch.evaluate(context);
        if value != watch.last_value {
            notify_watch_changed(watch.id, value);
        }
    }
}
```

**Tasks**:
- [ ] Implement watch expression storage
- [ ] Auto-evaluate on pause
- [ ] Track value changes
- [ ] Add DAP watch support
- [ ] Support complex expressions

---

### Milestone 14: Async/Promise Debugging

**Goals**:
- Break on promise rejection
- Track promise lifecycle
- Visualize async call chains

**Requires**:
- on_new_promise() hook
- on_promise_settled() hook
- Promise tracking infrastructure

**Tasks**:
- [ ] Implement promise lifecycle hooks
- [ ] Track promise creation and settlement
- [ ] Add async call stack tracking
- [ ] Break on unhandled rejections
- [ ] DAP async stack frames

---

### Milestone 15: Performance Profiling

**Goals**:
- CPU profiling with sampling
- Memory allocation tracking
- Call tree visualization

**Implementation**:
```rust
pub struct Profiler {
    samples: Vec<ProfileSample>,
    start_time: Instant,
}

pub struct ProfileSample {
    timestamp: Duration,
    stack: Vec<FunctionId>,
}

impl Debugger {
    pub fn start_profiling(&mut self);
    pub fn stop_profiling(&mut self) -> ProfileResult;
}
```

**Tasks**:
- [ ] Implement CPU sampling
- [ ] Track function timings
- [ ] Generate call tree
- [ ] Export in Chrome DevTools format
- [ ] Add memory profiling

---

## 🎨 Phase 7: VS Code Extension & IDE Integration (FUTURE)

**Priority**: HIGH (for adoption)  
**Complexity**: MEDIUM  
**ETA**: 2-3 weeks

### Milestone 16: Official Boa VS Code Extension

**Goals**:
- Create official VS Code extension for Boa debugging
- Seamless debugging experience for Boa JavaScript
- Lower barrier to entry for new users
- Increase community adoption

**Features**:

**Debug Configuration**:
```json
// .vscode/launch.json
{
  "type": "boa",
  "request": "launch",
  "name": "Debug with Boa",
  "program": "${file}",
  "stopOnEntry": false,
  "args": []
}
```

**Extension Capabilities**:
- Auto-detect Boa installation
- Syntax highlighting for Boa-specific features
- IntelliSense for Boa runtime APIs
- Integrated REPL/Debug Console
- Quick script execution
- Performance profiling integration
- Test runner integration

**Implementation**:
```typescript
// extension.ts
import * as vscode from 'vscode';
import { BoaDebugAdapterDescriptorFactory } from './debugAdapter';

export function activate(context: vscode.ExtensionContext) {
    // Register debug adapter
    context.subscriptions.push(
        vscode.debug.registerDebugAdapterDescriptorFactory('boa', 
            new BoaDebugAdapterDescriptorFactory()
        )
    );
    
    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('boa.run', runBoaScript),
        vscode.commands.registerCommand('boa.debug', debugBoaScript),
        vscode.commands.registerCommand('boa.repl', openBoaREPL)
    );
}
```

**Debug Adapter Integration**:
```typescript
class BoaDebugAdapterDescriptorFactory implements vscode.DebugAdapterDescriptorFactory {
    createDebugAdapterDescriptor(
        session: vscode.DebugSession,
        executable: vscode.DebugAdapterExecutable | undefined
    ): vscode.ProviderResult<vscode.DebugAdapterDescriptor> {
        // Find boa executable
        const boaPath = findBoaExecutable();
        
        // Launch boa with --dap flag
        return new vscode.DebugAdapterExecutable(
            boaPath,
            ['--dap'],
            { cwd: session.workspaceFolder?.uri.fsPath }
        );
    }
}
```

**Tasks**:
- [ ] Create VS Code extension project structure
- [ ] Implement debug adapter factory
- [ ] Add launch/attach configurations
- [ ] Create syntax highlighting grammar
- [ ] Add code snippets for common patterns
- [ ] Implement IntelliSense provider
- [ ] Add integrated terminal/REPL
- [ ] Create extension documentation
- [ ] Add extension icon and branding
- [ ] Publish to VS Code Marketplace
- [ ] Create tutorial videos/documentation
- [ ] Add telemetry (opt-in) for usage tracking

**Distribution**:
- [ ] Publish to [VS Code Marketplace](https://marketplace.visualstudio.com/)
- [ ] Add extension to Boa documentation
- [ ] Create getting started guide
- [ ] Add extension badge to Boa README
- [ ] Announce on social media/community channels

**User Experience Goals**:
- Zero-config debugging for simple scripts
- One-click "Run in Boa" from editor
- Interactive debugging with breakpoints, watches, call stack
- Integrated documentation on hover
- Quick fixes for common errors
- Performance hints and suggestions

**Example Workflow**:
```javascript
// user writes script.js
function fibonacci(n) {
    if (n <= 1) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

debugger; // Set breakpoint with keyword
console.log(fibonacci(10));

// User presses F5 → VS Code launches Boa with DAP
// Breakpoint hit → user can inspect variables, step through code
// Full debugging experience just like Node.js
```

**Success Metrics**:
- 1000+ extension installs in first 3 months
- 4.5+ star rating on marketplace
- Active user engagement (debugging sessions/week)
- Community contributions to extension
- Reduced "how to debug" support questions

**Impact**: Makes Boa accessible to JavaScript developers familiar with VS Code, significantly increasing adoption and community engagement.

---

## 📊 Feature Completeness Tracking

### Overall Progress: 60%

```
Component                  Complete  In Progress  Not Started
──────────────────────────────────────────────────────────────
Debugger Core              ████████  ▓▓▓▓▓▓▓▓     ░░░░░░░░  100%
VM Integration             ████████  ░░░░░░░░     ░░░░░░░░   80%
Breakpoint System          ████████  ▓▓▓▓▓▓▓▓     ░░░░░░░░  100%
Stepping                   ████████  ▓▓▓▓▓▓▓▓     ░░░░░░░░  100%
Hook System                ████████  ░░░░░░░░     ░░░░░░░░   60%
Script Registry            ░░░░░░░░  ░░░░░░░░     ████████    0%
Line Mapping               ░░░░░░░░  ░░░░░░░░     ████████    0%
Reflection Objects         ████░░░░  ░░░░░░░░     ████████   20%
DAP Protocol               ████████  ▓▓▓▓▓▓▓▓     ░░░░░░░░  100%
DAP Commands               ████░░░░  ░░░░░░░░     ████████   50%
Conditional BP             ████░░░░  ░░░░░░░░     ████████   30%
Exception BP               ░░░░░░░░  ░░░░░░░░     ████████    0%
Watch Expressions          ░░░░░░░░  ░░░░░░░░     ████████    0%
Async Debugging            ░░░░░░░░  ░░░░░░░░     ████████    0%
Profiling                  ░░░░░░░░  ░░░░░░░░     ████████    0%
VS Code Extension          ░░░░░░░░  ░░░░░░░░     ████████    0%
──────────────────────────────────────────────────────────────
TOTAL                      ████████▓▓░░░░░░░░     ████████   60%
```

## 🎯 Next 3 Months Goals

### Month 1: Complete VM Integration
- [ ] Implement frame enter/exit hooks
- [ ] Test hook integration thoroughly
- [ ] Measure performance impact
- [ ] Optimize if needed

### Month 2: Script Management
- [ ] Implement ScriptRegistry
- [ ] Build line-to-PC mapping
- [ ] Update DAP breakpoint handling
- [ ] Enable line-based breakpoints

### Month 3: Reflection & DAP
- [ ] Implement frame.eval()
- [ ] Add variable inspection
- [ ] Complete DAP commands
- [ ] Full VS Code integration

### Month 4+: Extension & Adoption
- [ ] Design VS Code extension architecture
- [ ] Implement debug adapter integration
- [ ] Create syntax highlighting and IntelliSense
- [ ] Publish to VS Code Marketplace
- [ ] Community outreach and documentation

## 📈 Success Metrics

- **Feature Parity**: 80%+ with SpiderMonkey core features
- **Performance**: <20% overhead when debugging enabled
- **IDE Integration**: Full VS Code debugging workflow
- **VS Code Extension**: Published on marketplace with 1000+ installs
- **Documentation**: 100% API coverage
- **Examples**: 5+ comprehensive examples
- **Tests**: 80%+ code coverage
- **Community Adoption**: Active user base and contributions

## 🤝 Community Contributions Welcome

### Good First Issues
- Add more examples
- Improve error messages
- Write unit tests
- Update documentation

### Advanced Issues
- Implement reflection methods
- Add DAP command handlers
- Optimize breakpoint checking
- Add source map support

## 📚 References

- [SpiderMonkey Debugger Docs](https://firefox-source-docs.mozilla.org/devtools/debugger-api/)
- [DAP Specification](https://microsoft.github.io/debug-adapter-protocol/)
- [V8 Inspector](https://v8.dev/docs/inspector)
- [Chrome DevTools Protocol](https://chromedevtools.github.io/devtools-protocol/)

---

**Last Updated**: January 2026  
**Maintained By**: Boa Core Team  
**Status**: Actively Developed
