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
    // Note: Parameter names are not preserved in CompiledFunction (core limitation)
    // The param name will be "arg0" instead of "name"
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

describe('SQL Feature', () => {
  test('sql runtime creation', () => {
    const rt = new Runtime({ sql: true });
    expect(rt).toBeDefined();
  });

  test('basic sql query', () => {
    const rt = new Runtime({ sql: true });
    const result = rt.eval('SQL("SELECT 1 + 1 as result")');
    expect(result).toEqual([{ result: 2 }]);
  });

  test('sql multiple rows', () => {
    const rt = new Runtime({ sql: true });
    const result = rt.eval(`
      let people = [
        {id: 1, name: "Alice"},
        {id: 2, name: "Bob"},
        {id: 3, name: "Carol"}
      ]
      SQL("SELECT * FROM people ORDER BY id")
    `);
    expect(result.length).toBe(3);
    expect(result.map(r => r.name)).toEqual(['Alice', 'Bob', 'Carol']);
  });

  test('sql aggregation', () => {
    const rt = new Runtime({ sql: true });
    const result = rt.eval(`
      let data = [{x: 10}, {x: 20}, {x: 30}]
      SQL("SELECT COUNT(*) as cnt, SUM(x) as total FROM data")
    `);
    expect(result[0].cnt).toBe(3);
    expect(result[0].total).toBe(60);
  });
});

describe('SQL on Lattice Data', () => {
  test('sql on lattice variable', () => {
    const rt = new Runtime({ sql: true });
    rt.eval(`
      let users = [
        {id: 1, name: "Alice", age: 30},
        {id: 2, name: "Bob", age: 18},
        {id: 3, name: "Charlie", age: 25}
      ]
    `);
    const result = rt.eval('SQL("SELECT * FROM users WHERE age > 21 ORDER BY id")');
    expect(result.length).toBe(2);
    expect(result[0].name).toBe('Alice');
    expect(result[1].name).toBe('Charlie');
  });

  test('sql aggregate on lattice data', () => {
    const rt = new Runtime({ sql: true });
    rt.eval(`
      let sales = [
        {product: "A", amount: 100},
        {product: "A", amount: 150},
        {product: "B", amount: 200}
      ]
    `);
    const result = rt.eval('SQL("SELECT SUM(amount) as total FROM sales")');
    expect(result[0].total).toBe(450);
  });

  test('sql join lattice tables', () => {
    const rt = new Runtime({ sql: true });
    rt.eval(`
      let customers = [
        {id: 1, name: "Alice"},
        {id: 2, name: "Bob"}
      ]
      let orders = [
        {customer_id: 1, product: "Widget", amount: 100},
        {customer_id: 1, product: "Gadget", amount: 50},
        {customer_id: 2, product: "Widget", amount: 200}
      ]
    `);
    const result = rt.eval(`
      SQL("SELECT c.name, SUM(o.amount) as total
           FROM customers c
           JOIN orders o ON c.id = o.customer_id
           GROUP BY c.name
           ORDER BY c.name")
    `);
    expect(result.length).toBe(2);
    expect(result[0]).toEqual({ name: 'Alice', total: 150 });
    expect(result[1]).toEqual({ name: 'Bob', total: 200 });
  });

  test('sql table not found error', () => {
    const rt = new Runtime({ sql: true });
    expect(() => rt.eval('SQL("SELECT * FROM nonexistent")')).toThrow();
  });

  test('sql wrong type error', () => {
    const rt = new Runtime({ sql: true });
    rt.eval('let not_a_list = "hello"');
    expect(() => rt.eval('SQL("SELECT * FROM not_a_list")')).toThrow();
  });
});

describe('SQL on JavaScript Data', () => {
  test('sql on js array of objects via setGlobal', () => {
    const rt = new Runtime({ sql: true });

    const users = [
      { id: 1, name: 'Alice', age: 30 },
      { id: 2, name: 'Bob', age: 18 },
      { id: 3, name: 'Charlie', age: 25 }
    ];
    rt.setGlobal('users', users);

    const result = rt.eval('SQL("SELECT * FROM users WHERE age >= 25 ORDER BY id")');

    expect(result.length).toBe(2);
    expect(result[0].name).toBe('Alice');
    expect(result[1].name).toBe('Charlie');
  });

  test('sql on js data with bindings', () => {
    const rt = new Runtime({ sql: true });

    const products = [
      { name: 'Widget', price: 9.99, stock: 100 },
      { name: 'Gadget', price: 19.99, stock: 50 },
      { name: 'Gizmo', price: 14.99, stock: 75 }
    ];

    const result = rt.eval(
      'SQL("SELECT name, price FROM products WHERE price > 10 ORDER BY price")',
      { products }
    );

    expect(result.length).toBe(2);
    expect(result[0].name).toBe('Gizmo');
    expect(result[1].name).toBe('Gadget');
  });

  test('sql aggregate on js data', () => {
    const rt = new Runtime({ sql: true });

    const sales = [
      { region: 'North', amount: 1000 },
      { region: 'South', amount: 1500 },
      { region: 'North', amount: 800 },
      { region: 'South', amount: 1200 }
    ];
    rt.setGlobal('sales', sales);

    const result = rt.eval(`
      SQL("SELECT region, SUM(amount) as total, COUNT(*) as count
           FROM sales GROUP BY region ORDER BY region")
    `);

    expect(result.length).toBe(2);
    expect(result[0]).toEqual({ region: 'North', total: 1800, count: 2 });
    expect(result[1]).toEqual({ region: 'South', total: 2700, count: 2 });
  });

  test('sql join js and lattice data', () => {
    const rt = new Runtime({ sql: true });

    // JavaScript data
    const orders = [
      { order_id: 1, customer_id: 1, amount: 100 },
      { order_id: 2, customer_id: 2, amount: 200 },
      { order_id: 3, customer_id: 1, amount: 150 }
    ];
    rt.setGlobal('orders', orders);

    // Lattice data
    rt.eval(`
      let customers = [
        {id: 1, name: "Alice"},
        {id: 2, name: "Bob"}
      ]
    `);

    const result = rt.eval(`
      SQL("SELECT c.name, SUM(o.amount) as total
           FROM customers c
           JOIN orders o ON c.id = o.customer_id
           GROUP BY c.name
           ORDER BY total DESC")
    `);

    expect(result.length).toBe(2);
    expect(result[0]).toEqual({ name: 'Alice', total: 250 });
    expect(result[1]).toEqual({ name: 'Bob', total: 200 });
  });

  test('sql with null values', () => {
    const rt = new Runtime({ sql: true });

    const data = [
      { id: 1, value: 100 },
      { id: 2, value: null },
      { id: 3, value: 300 }
    ];
    rt.setGlobal('data', data);

    const result = rt.eval('SQL("SELECT * FROM data WHERE value IS NOT NULL ORDER BY id")');

    expect(result.length).toBe(2);
    expect(result[0].id).toBe(1);
    expect(result[1].id).toBe(3);
  });

  test('sql with boolean values', () => {
    const rt = new Runtime({ sql: true });

    const users = [
      { name: 'Alice', active: true },
      { name: 'Bob', active: false },
      { name: 'Charlie', active: true }
    ];
    rt.setGlobal('users', users);

    const result = rt.eval('SQL("SELECT name FROM users WHERE active = true ORDER BY name")');

    expect(result.length).toBe(2);
    expect(result[0].name).toBe('Alice');
    expect(result[1].name).toBe('Charlie');
  });
});
