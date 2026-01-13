const { Runtime, path } = require('..');

describe('Basic Eval', () => {
  test('arithmetic', () => {
    const rt = new Runtime();
    expect(rt.eval('1 + 2 * 3')).toBe(7);
  });

  test('string', () => {
    const rt = new Runtime();
    expect(rt.eval('"hello" + " world"')).toBe('hello world');
  });

  test('list', () => {
    const rt = new Runtime();
    expect(rt.eval('[1, 2, 3]')).toEqual([1, 2, 3]);
  });

  test('map', () => {
    const rt = new Runtime();
    expect(rt.eval('{"a": 1, "b": 2}')).toEqual({ a: 1, b: 2 });
  });

  test('null', () => {
    const rt = new Runtime();
    expect(rt.eval('null')).toBeNull();
  });

  test('bool true', () => {
    const rt = new Runtime();
    expect(rt.eval('true')).toBe(true);
  });

  test('bool false', () => {
    const rt = new Runtime();
    expect(rt.eval('false')).toBe(false);
  });

  test('float', () => {
    const rt = new Runtime();
    expect(rt.eval('3.14')).toBeCloseTo(3.14);
  });

  test('negative int', () => {
    const rt = new Runtime();
    expect(rt.eval('-42')).toBe(-42);
  });

  test('nested list', () => {
    const rt = new Runtime();
    expect(rt.eval('[[1, 2], [3, 4]]')).toEqual([[1, 2], [3, 4]]);
  });

  test('nested map', () => {
    const rt = new Runtime();
    expect(rt.eval('{"outer": {"inner": 1}}')).toEqual({ outer: { inner: 1 } });
  });
});

describe('Bindings', () => {
  test('eval with bindings', () => {
    const rt = new Runtime();
    const result = rt.eval('x + y', { x: 10, y: 20 });
    expect(result).toBe(30);
  });

  test('bindings with list', () => {
    const rt = new Runtime();
    const result = rt.eval('items[0] + items[1]', { items: [3, 4] });
    expect(result).toBe(7);
  });

  test('bindings with map', () => {
    const rt = new Runtime();
    const result = rt.eval('data["key"]', { data: { key: 42 } });
    expect(result).toBe(42);
  });

  test('bindings with null', () => {
    const rt = new Runtime();
    const result = rt.eval('x', { x: null });
    expect(result).toBeNull();
  });

  test('bindings with bool', () => {
    const rt = new Runtime();
    const result = rt.eval('x', { x: true });
    expect(result).toBe(true);
  });
});

describe('Globals', () => {
  test('set and get global', () => {
    const rt = new Runtime();
    rt.setGlobal('count', 42);
    expect(rt.getGlobal('count')).toBe(42);
  });

  test('global in eval', () => {
    const rt = new Runtime();
    rt.setGlobal('x', 100);
    expect(rt.eval('x * 2')).toBe(200);
  });

  test('get undefined global returns null', () => {
    const rt = new Runtime();
    expect(rt.getGlobal('undefined_var')).toBeNull();
  });

  test('overwrite global', () => {
    const rt = new Runtime();
    rt.setGlobal('x', 1);
    rt.setGlobal('x', 2);
    expect(rt.getGlobal('x')).toBe(2);
  });
});

describe('Functions', () => {
  test('define and call', () => {
    const rt = new Runtime();
    rt.eval('def add(a: Int, b: Int) -> Int { a + b }');
    expect(rt.call('add', 3, 4)).toBe(7);
  });

  test('has function', () => {
    const rt = new Runtime();
    rt.eval('def foo() -> Int { 42 }');
    expect(rt.hasFunction('foo')).toBe(true);
    expect(rt.hasFunction('bar')).toBe(false);
  });

  test('function signatures', () => {
    const rt = new Runtime();
    rt.eval('def greet(name: String) -> String { name }');
    const sigs = rt.getFunctionSignatures();
    expect(sigs.length).toBe(1);
    expect(sigs[0].name).toBe('greet');
    expect(sigs[0].params.length).toBe(1);
    expect(sigs[0].params[0].name).toBe('name');
  });

  test('multiple functions', () => {
    const rt = new Runtime();
    rt.eval('def f1() -> Int { 1 }');
    rt.eval('def f2() -> Int { 2 }');
    expect(rt.hasFunction('f1')).toBe(true);
    expect(rt.hasFunction('f2')).toBe(true);
    expect(rt.call('f1')).toBe(1);
    expect(rt.call('f2')).toBe(2);
  });
});

describe('Types', () => {
  test('struct', () => {
    const rt = new Runtime();
    rt.eval('type Person { name: String, age: Int }');
    const types = rt.getTypes();
    expect(types.length).toBe(1);
    expect(types[0].kind).toBe('struct');
    expect(types[0].name).toBe('Person');
    expect(types[0].fields.length).toBe(2);
  });

  test('enum', () => {
    const rt = new Runtime();
    rt.eval('enum Color { Red, Green, Blue }');
    const types = rt.getTypes();
    expect(types.length).toBe(1);
    expect(types[0].kind).toBe('enum');
    expect(types[0].name).toBe('Color');
    expect(types[0].variants).toEqual(['Red', 'Green', 'Blue']);
  });
});

describe('Reset', () => {
  test('reset clears globals', () => {
    const rt = new Runtime();
    rt.setGlobal('x', 42);
    rt.reset();
    expect(rt.getGlobal('x')).toBeNull();
  });

  test('reset clears functions', () => {
    const rt = new Runtime();
    rt.eval('def foo() -> Int { 42 }');
    expect(rt.hasFunction('foo')).toBe(true);
    rt.reset();
    expect(rt.hasFunction('foo')).toBe(false);
  });
});

describe('Errors', () => {
  test('syntax error', () => {
    const rt = new Runtime();
    expect(() => rt.eval('1 +')).toThrow();
  });

  test('undefined variable', () => {
    const rt = new Runtime();
    expect(() => rt.eval('undefined_var')).toThrow();
  });

  test('undefined function', () => {
    const rt = new Runtime();
    expect(() => rt.call('nonexistent')).toThrow();
  });

  test('type error', () => {
    const rt = new Runtime();
    expect(() => rt.eval('"hello" + 1')).toThrow();
  });
});

describe('Path values', () => {
  test('path helper creates marker', () => {
    const p = path('/home/user/file.txt');
    expect(p.__lattice_path__).toBe(true);
    expect(p.value).toBe('/home/user/file.txt');
  });

  test('path can be passed as binding', () => {
    const rt = new Runtime();
    const p = path('/test/path');
    // Just verify it doesn't throw - paths are handled internally
    rt.setGlobal('p', p);
    const result = rt.getGlobal('p');
    expect(result.__lattice_path__).toBe(true);
    expect(result.value).toBe('/test/path');
  });
});

describe('Custom inspect', () => {
  test('inspect shows function and type count', () => {
    const rt = new Runtime();
    rt.eval('def foo() -> Int { 1 }');
    rt.eval('type Bar { x: Int }');
    const util = require('util');
    const inspected = util.inspect(rt);
    expect(inspected).toMatch(/Runtime \{ 1 functions?, 1 types? \}/);
  });
});
