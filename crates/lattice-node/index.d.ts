/**
 * Lattice Runtime for Node.js
 * A statically-typed language for structured LLM interactions
 */

/** Represents a filesystem path in Lattice */
export interface LatticePath {
  __lattice_path__: true;
  value: string;
}

/** Lattice value types that can be passed to/from the runtime */
export type LatticeValue =
  | null
  | boolean
  | number
  | string
  | LatticePath
  | LatticeValue[]
  | { [key: string]: LatticeValue };

/** Runtime configuration options */
export interface RuntimeOptions {
  /** Enable LLM support (requires OPENROUTER_API_KEY env var) */
  llm?: boolean;
  /** Enable SQL/DuckDB support */
  sql?: boolean;
}

/** Type schema for introspection */
export interface TypeSchema {
  kind:
    | "null"
    | "bool"
    | "int"
    | "float"
    | "string"
    | "path"
    | "any"
    | "list"
    | "map"
    | "optional"
    | "struct"
    | "enum"
    | "named";
  /** For list/optional types */
  inner?: TypeSchema;
  /** For map types - key schema */
  key?: TypeSchema;
  /** For map types - value schema */
  value?: TypeSchema;
  /** For struct/enum/named types */
  name?: string;
  /** For struct types */
  fields?: FieldSchema[];
  /** For enum types */
  variants?: string[];
  /** Optional description */
  description?: string;
}

/** Field schema for struct types */
export interface FieldSchema {
  name: string;
  type_schema: TypeSchema;
  optional: boolean;
  description?: string;
}

/** Parameter schema for function signatures */
export interface ParameterSchema {
  name: string;
  type_schema: TypeSchema;
}

/** Function signature for introspection */
export interface FunctionSignature {
  name: string;
  params: ParameterSchema[];
  return_type: TypeSchema;
  is_llm: boolean;
  is_async: boolean;
}

/** Debug information from LLM calls */
export interface LlmDebugInfo {
  prompt: string;
  raw_response: string;
  function_name: string;
  return_type: string;
}

/**
 * Lattice Runtime - the main class for evaluating Lattice code
 */
export class Runtime {
  /**
   * Create a new Lattice runtime
   * @param options Configuration options
   */
  constructor(options?: RuntimeOptions);

  /**
   * Evaluate Lattice source code
   * @param source Lattice source code to evaluate
   * @param bindings Optional variable bindings
   * @returns The result of evaluation
   * @throws Error if evaluation fails
   */
  eval(source: string, bindings?: Record<string, LatticeValue>): LatticeValue;

  /**
   * Evaluate a Lattice file (.lat or .md)
   * @param path Path to the file
   * @returns The result of evaluation
   * @throws Error if file cannot be read or evaluation fails
   */
  evalFile(path: string): LatticeValue;

  /**
   * Call a registered function by name
   * @param name Function name
   * @param args Arguments to pass
   * @returns The function's return value
   * @throws Error if function doesn't exist or call fails
   */
  call(name: string, ...args: LatticeValue[]): LatticeValue;

  /**
   * Get a global variable's value
   * @param name Variable name
   * @returns The value, or null if not found
   */
  getGlobal(name: string): LatticeValue;

  /**
   * Set a global variable
   * @param name Variable name
   * @param value Value to set
   */
  setGlobal(name: string, value: LatticeValue): void;

  /**
   * Check if a function exists
   * @param name Function name
   * @returns true if function exists
   */
  hasFunction(name: string): boolean;

  /**
   * Reset the runtime, clearing all state
   */
  reset(): void;

  /**
   * Get all registered type schemas
   * @returns Array of type schemas
   */
  getTypes(): TypeSchema[];

  /**
   * Get all function signatures
   * @returns Array of function signatures
   */
  getFunctionSignatures(): FunctionSignature[];

  /**
   * Take debug info from the last LLM call
   * @returns Debug info or null if no LLM call was made
   */
  takeLlmDebug(): LlmDebugInfo | null;
}

/**
 * Create a Path value for passing to Lattice
 * @param pathString The path string
 */
export function path(pathString: string): LatticePath;
