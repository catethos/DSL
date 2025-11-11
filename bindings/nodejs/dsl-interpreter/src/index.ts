/**
 * DSL IR Interpreter - Node.js Bindings
 *
 * High-level TypeScript API for the DSL interpreter using Neon.
 */

import {
  IR,
  Node,
  Value,
  TraceConfig,
  TraceEvent,
  InterpreterError,
} from "./types";

// Import native module (compiled from Rust)
// eslint-disable-next-line @typescript-eslint/no-var-requires
const native = require("./index.node");

// ============================================================================
// Error Class
// ============================================================================

export class DSLInterpreterError extends Error {
  public readonly kind: string;
  public readonly details: InterpreterError;

  constructor(details: InterpreterError) {
    super(details.message);
    this.name = "DSLInterpreterError";
    this.kind = details.kind;
    this.details = details;

    // Maintain proper stack trace (only available in V8)
    if (Error.captureStackTrace) {
      Error.captureStackTrace(this, DSLInterpreterError);
    }
  }
}

// ============================================================================
// Helper Functions
// ============================================================================

/**
 * Normalizes input to JSON string
 */
function toJSON(input: object | string): string {
  if (typeof input === "string") {
    return input;
  }
  return JSON.stringify(input);
}

/**
 * Parses JSON string to object
 */
function fromJSON<T>(json: string): T {
  return JSON.parse(json) as T;
}

// ============================================================================
// Interpreter Class
// ============================================================================

/**
 * Main interpreter class for evaluating DSL IR nodes
 */
export class Interpreter {
  private readonly id: number;

  /**
   * Creates a new interpreter instance
   */
  constructor() {
    this.id = native.newInterpreter();
  }

  /**
   * Creates an interpreter from an IR definition
   */
  static fromIR(ir: IR | string): Interpreter {
    const irJSON = toJSON(ir);
    const id = native.interpreterFromIr(irJSON);
    const interpreter = Object.create(Interpreter.prototype);
    interpreter.id = id;
    return interpreter;
  }

  /**
   * Evaluates an IR node and returns the result
   *
   * @param node - The IR node to evaluate (can be an object or JSON string)
   * @returns Promise that resolves to the evaluated value
   * @throws {DSLInterpreterError} If evaluation fails
   */
  async eval(node: Node | string): Promise<Value> {
    const nodeJSON = toJSON(node);
    try {
      const resultJSON = await native.eval(this.id, nodeJSON);
      return fromJSON<Value>(resultJSON);
    } catch (error) {
      if (error instanceof Error) {
        throw new DSLInterpreterError({
          kind: "RuntimeError",
          message: error.message,
        });
      }
      throw error;
    }
  }

  /**
   * Destroy this interpreter instance and free resources
   */
  destroy(): void {
    native.destroyInterpreter(this.id);
  }

  /**
   * Convenience method: creates an interpreter from IR and evaluates a node in one call
   *
   * @param ir - The IR definition
   * @param node - The node to evaluate (defaults to the entry_expr from IR)
   * @returns Promise that resolves to the evaluated value
   */
  static async run(ir: IR | string, node?: Node | string): Promise<Value> {
    const interpreter = Interpreter.fromIR(ir);

    // If no node provided, use the entry_expr from IR
    if (!node) {
      const irObj = typeof ir === "string" ? fromJSON<IR>(ir) : ir;
      node = irObj.entry_expr;
    }

    try {
      return await interpreter.eval(node);
    } finally {
      interpreter.destroy();
    }
  }
}

// ============================================================================
// TracingInterpreter Class
// ============================================================================

/**
 * Interpreter with tracing capabilities for debugging and observability
 */
export class TracingInterpreter {
  private readonly id: number;

  /**
   * Creates a new tracing interpreter with optional configuration
   *
   * @param config - Trace configuration options (currently ignored - uses default)
   */
  constructor(config: TraceConfig = {}) {
    const configJSON = toJSON(config);
    this.id = native.newTracingInterpreter(configJSON);
  }

  /**
   * Creates a tracing interpreter from an IR definition
   *
   * @param ir - The IR definition
   * @param config - Trace configuration options (currently ignored - uses default)
   */
  static fromIR(ir: IR | string, config: TraceConfig = {}): TracingInterpreter {
    const irJSON = toJSON(ir);
    const configJSON = toJSON(config);
    const id = native.tracingInterpreterFromIr(irJSON, configJSON);
    const interpreter = Object.create(TracingInterpreter.prototype);
    interpreter.id = id;
    return interpreter;
  }

  /**
   * Evaluates an IR node with tracing enabled
   *
   * @param node - The IR node to evaluate
   * @returns Promise that resolves to the result with traces
   * @throws {DSLInterpreterError} If evaluation fails
   */
  async evalWithTrace(node: Node | string): Promise<{
    value: Value;
    traces: TraceEvent[];
  }> {
    const nodeJSON = toJSON(node);
    try {
      const result = await native.evalWithTrace(this.id, nodeJSON);
      return {
        value: fromJSON<Value>(result.value),
        traces: fromJSON<TraceEvent[]>(result.traces),
      };
    } catch (error) {
      if (error instanceof Error) {
        throw new DSLInterpreterError({
          kind: "RuntimeError",
          message: error.message,
        });
      }
      throw error;
    }
  }

  /**
   * Evaluates a node without returning traces (but still collects them internally)
   *
   * @param node - The IR node to evaluate
   * @returns Promise that resolves to the evaluated value
   */
  async eval(node: Node | string): Promise<Value> {
    const result = await this.evalWithTrace(node);
    return result.value;
  }

  /**
   * Clears the internal trace buffer
   */
  clearTrace(): void {
    native.clearTrace(this.id);
  }

  /**
   * Retrieves the current trace buffer
   *
   * @returns Array of trace events
   */
  getTrace(): TraceEvent[] {
    const traceJSON = native.getTrace(this.id);
    return fromJSON<TraceEvent[]>(traceJSON);
  }

  /**
   * Destroy this interpreter instance and free resources
   */
  destroy(): void {
    native.destroyTracingInterpreter(this.id);
  }

  /**
   * Convenience method: creates a tracing interpreter from IR and evaluates a node
   *
   * @param ir - The IR definition
   * @param node - The node to evaluate (defaults to entry_expr from IR)
   * @param config - Trace configuration options
   * @returns Promise that resolves to the result with traces
   */
  static async run(
    ir: IR | string,
    node?: Node | string,
    config: TraceConfig = {}
  ): Promise<{ value: Value; traces: TraceEvent[] }> {
    const interpreter = TracingInterpreter.fromIR(ir, config);

    // If no node provided, use the entry_expr from IR
    if (!node) {
      const irObj = typeof ir === "string" ? fromJSON<IR>(ir) : ir;
      node = irObj.entry_expr;
    }

    try {
      return await interpreter.evalWithTrace(node);
    } finally {
      interpreter.destroy();
    }
  }
}

// ============================================================================
// Exports
// ============================================================================

export * from "./types";

export default {
  Interpreter,
  TracingInterpreter,
  DSLInterpreterError,
};
