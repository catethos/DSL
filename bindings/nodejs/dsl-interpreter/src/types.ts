/**
 * Type definitions for DSL IR Interpreter
 */

// ============================================================================
// Span and Location Types
// ============================================================================

export interface Span {
  file: string;
  line: number;
  column: number;
}

// ============================================================================
// Value Types (Runtime values returned by the interpreter)
// ============================================================================

export type Value =
  | { type: "null" }
  | { type: "bool"; value: boolean }
  | { type: "int"; value: number }
  | { type: "float"; value: number }
  | { type: "string"; value: string }
  | { type: "array"; value: Value[] }
  | { type: "object"; value: Record<string, Value> }
  | { type: "function"; name: string; params: string[]; body: Node };

// ============================================================================
// IR Node Types
// ============================================================================

export type Node =
  | LiteralNode
  | VariableNode
  | BinaryOpNode
  | UnaryOpNode
  | IfNode
  | LetNode
  | FunctionCallNode
  | LambdaNode
  | ArrayNode
  | ObjectNode
  | IndexNode
  | MemberAccessNode
  | AssignNode
  | BlockNode
  | LoopNode
  | ReturnNode
  | BreakNode
  | ContinueNode
  | TryCatchNode
  | ThrowNode
  | IntrinsicNode;

export interface LiteralNode {
  type: "Literal";
  value: Value;
  span?: Span;
}

export interface VariableNode {
  type: "Variable";
  name: string;
  span?: Span;
}

export interface BinaryOpNode {
  type: "BinaryOp";
  op: BinaryOperator;
  left: Node;
  right: Node;
  span?: Span;
}

export type BinaryOperator =
  | "Add"
  | "Subtract"
  | "Multiply"
  | "Divide"
  | "Modulo"
  | "Equal"
  | "NotEqual"
  | "LessThan"
  | "LessThanOrEqual"
  | "GreaterThan"
  | "GreaterThanOrEqual"
  | "And"
  | "Or";

export interface UnaryOpNode {
  type: "UnaryOp";
  op: UnaryOperator;
  operand: Node;
  span?: Span;
}

export type UnaryOperator = "Negate" | "Not";

export interface IfNode {
  type: "If";
  condition: Node;
  then_branch: Node;
  else_branch?: Node;
  span?: Span;
}

export interface LetNode {
  type: "Let";
  name: string;
  value: Node;
  body: Node;
  span?: Span;
}

export interface FunctionCallNode {
  type: "FunctionCall";
  function: Node;
  arguments: Node[];
  span?: Span;
}

export interface LambdaNode {
  type: "Lambda";
  params: string[];
  body: Node;
  span?: Span;
}

export interface ArrayNode {
  type: "Array";
  elements: Node[];
  span?: Span;
}

export interface ObjectNode {
  type: "Object";
  fields: Record<string, Node>;
  span?: Span;
}

export interface IndexNode {
  type: "Index";
  object: Node;
  index: Node;
  span?: Span;
}

export interface MemberAccessNode {
  type: "MemberAccess";
  object: Node;
  member: string;
  span?: Span;
}

export interface AssignNode {
  type: "Assign";
  target: Node;
  value: Node;
  span?: Span;
}

export interface BlockNode {
  type: "Block";
  statements: Node[];
  span?: Span;
}

export interface LoopNode {
  type: "Loop";
  condition: Node;
  body: Node;
  span?: Span;
}

export interface ReturnNode {
  type: "Return";
  value?: Node;
  span?: Span;
}

export interface BreakNode {
  type: "Break";
  span?: Span;
}

export interface ContinueNode {
  type: "Continue";
  span?: Span;
}

export interface TryCatchNode {
  type: "TryCatch";
  try_block: Node;
  catch_param: string;
  catch_block: Node;
  span?: Span;
}

export interface ThrowNode {
  type: "Throw";
  value: Node;
  span?: Span;
}

export interface IntrinsicNode {
  type: "Intrinsic";
  name: string;
  arguments: Node[];
  span?: Span;
}

// ============================================================================
// IR Root Type
// ============================================================================

export interface IR {
  version: string;
  entry_expr: Node;
  functions?: Record<string, FunctionDefinition>;
  globals?: Record<string, Node>;
}

export interface FunctionDefinition {
  params: string[];
  body: Node;
}

// ============================================================================
// Tracing Types
// ============================================================================

export interface TraceConfig {
  max_depth?: number;
  capture_values?: boolean;
  capture_spans?: boolean;
  filter_effect_kinds?: EffectKind[];
}

export type EffectKind = "Pure" | "HTTP" | "SQL" | "LLM";

export interface TraceEvent {
  timestamp: number;
  depth: number;
  event_type: TraceEventType;
  node_type?: string;
  value?: Value;
  span?: Span;
  effect_kind?: EffectKind;
  metadata?: Record<string, unknown>;
}

export type TraceEventType =
  | "Enter"
  | "Exit"
  | "Error"
  | "EffectStart"
  | "EffectEnd";

// ============================================================================
// Error Types
// ============================================================================

export type InterpreterErrorKind =
  | "LLMError"
  | "HTTPError"
  | "SQLError"
  | "TypeError"
  | "RuntimeError"
  | "UnknownVariable"
  | "UnknownFunction"
  | "UnknownIntrinsic"
  | "InvalidArguments";

export interface BaseInterpreterError {
  kind: InterpreterErrorKind;
  message: string;
  span?: Span;
}

export interface LLMError extends BaseInterpreterError {
  kind: "LLMError";
  prompt?: string;
  response?: string;
}

export interface HTTPError extends BaseInterpreterError {
  kind: "HTTPError";
  method?: string;
  url?: string;
}

export interface SQLError extends BaseInterpreterError {
  kind: "SQLError";
  query?: string;
}

export interface TypeError extends BaseInterpreterError {
  kind: "TypeError";
  expected?: string;
  got?: string;
}

export interface RuntimeError extends BaseInterpreterError {
  kind: "RuntimeError";
}

export interface UnknownVariableError extends BaseInterpreterError {
  kind: "UnknownVariable";
  variableName: string;
}

export interface UnknownFunctionError extends BaseInterpreterError {
  kind: "UnknownFunction";
  functionName: string;
}

export interface UnknownIntrinsicError extends BaseInterpreterError {
  kind: "UnknownIntrinsic";
  intrinsicName: string;
}

export interface InvalidArgumentsError extends BaseInterpreterError {
  kind: "InvalidArguments";
  functionName?: string;
}

export type InterpreterError =
  | LLMError
  | HTTPError
  | SQLError
  | TypeError
  | RuntimeError
  | UnknownVariableError
  | UnknownFunctionError
  | UnknownIntrinsicError
  | InvalidArgumentsError;

// ============================================================================
// Result Types
// ============================================================================

export interface EvalResult {
  value: Value;
}

export interface EvalWithTraceResult {
  value: Value;
  traces: TraceEvent[];
}
