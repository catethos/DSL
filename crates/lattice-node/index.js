'use strict';

const native = loadNativeBinding();

function loadNativeBinding() {
  const { platform, arch } = process;

  // Platform package mapping
  const packages = {
    'darwin-arm64': '@lattice-lang/darwin-arm64',
    'linux-x64': '@lattice-lang/linux-x64',
  };

  const key = `${platform}-${arch}`;
  const pkg = packages[key];

  if (!pkg) {
    throw new Error(
      `Unsupported platform: ${platform}-${arch}. ` +
      `Supported platforms: darwin-arm64, linux-x64.`
    );
  }

  try {
    return require(pkg);
  } catch (e) {
    // Fallback to local build for development
    try {
      return require('./index.node');
    } catch {
      throw new Error(
        `Failed to load native binding for ${platform}-${arch}. ` +
        `Package ${pkg} not found. Run: npm install ${pkg}`
      );
    }
  }
}

/**
 * Create a LatticePath marker object
 * @param {string} pathString
 * @returns {{ __lattice_path__: true, value: string }}
 */
function path(pathString) {
  return { __lattice_path__: true, value: pathString };
}

/**
 * Lattice Runtime class
 */
class Runtime {
  #handle;

  /**
   * Create a new Lattice runtime
   * @param {{ llm?: boolean, sql?: boolean }} [options]
   */
  constructor(options = {}) {
    this.#handle = native.createRuntime(options);
  }

  /**
   * Evaluate Lattice source code
   * @param {string} source
   * @param {Record<string, any>} [bindings]
   * @returns {any}
   */
  eval(source, bindings) {
    return native.eval(this.#handle, source, bindings);
  }

  /**
   * Evaluate a Lattice file (.lat or .md)
   * @param {string} path
   * @returns {any}
   */
  evalFile(path) {
    return native.evalFile(this.#handle, path);
  }

  /**
   * Call a registered function by name
   * @param {string} name
   * @param  {...any} args
   * @returns {any}
   */
  call(name, ...args) {
    return native.call(this.#handle, name, args);
  }

  /**
   * Get a global variable's value
   * @param {string} name
   * @returns {any}
   */
  getGlobal(name) {
    return native.getGlobal(this.#handle, name);
  }

  /**
   * Set a global variable
   * @param {string} name
   * @param {any} value
   */
  setGlobal(name, value) {
    native.setGlobal(this.#handle, name, value);
  }

  /**
   * Check if a function exists
   * @param {string} name
   * @returns {boolean}
   */
  hasFunction(name) {
    return native.hasFunction(this.#handle, name);
  }

  /**
   * Reset the runtime, clearing all state
   */
  reset() {
    native.reset(this.#handle);
  }

  /**
   * Get all registered type schemas
   * @returns {Array<object>}
   */
  getTypes() {
    return native.getTypes(this.#handle);
  }

  /**
   * Get all function signatures
   * @returns {Array<object>}
   */
  getFunctionSignatures() {
    return native.getFunctionSignatures(this.#handle);
  }

  /**
   * Take debug info from the last LLM call
   * @returns {object|null}
   */
  takeLlmDebug() {
    return native.takeLlmDebug(this.#handle);
  }

  // Custom inspect for Node.js console
  [Symbol.for('nodejs.util.inspect.custom')]() {
    const funcs = native.getFunctionSignatures(this.#handle).length;
    const types = native.getTypes(this.#handle).length;
    return `Runtime { ${funcs} functions, ${types} types }`;
  }
}

module.exports = { Runtime, path };
