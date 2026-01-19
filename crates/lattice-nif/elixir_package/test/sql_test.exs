defmodule SQLTest do
  use ExUnit.Case

  describe "SQL Feature" do
    test "sql runtime creation" do
      {:ok, rt} = Lattice.new(sql: true)
      assert rt != nil
    end

    test "basic sql query" do
      {:ok, rt} = Lattice.new(sql: true)
      {:ok, result} = Lattice.eval(rt, ~s[SQL("SELECT 1 + 1 as result")])
      assert result == [%{"result" => 2}]
    end

    test "sql multiple rows" do
      {:ok, rt} = Lattice.new(sql: true)
      {:ok, result} = Lattice.eval(rt, """
        let people = [
          {id: 1, name: "Alice"},
          {id: 2, name: "Bob"},
          {id: 3, name: "Carol"}
        ]
        SQL("SELECT * FROM people ORDER BY id")
      """)
      assert length(result) == 3
      assert Enum.map(result, & &1["name"]) == ["Alice", "Bob", "Carol"]
    end

    test "sql aggregation" do
      {:ok, rt} = Lattice.new(sql: true)
      {:ok, result} = Lattice.eval(rt, """
        let data = [{x: 10}, {x: 20}, {x: 30}]
        SQL("SELECT COUNT(*) as cnt, SUM(x) as total FROM data")
      """)
      assert hd(result)["cnt"] == 3
      assert hd(result)["total"] == 60
    end
  end

  describe "SQL on Lattice Data" do
    test "sql on lattice variable" do
      {:ok, rt} = Lattice.new(sql: true)
      {:ok, _} = Lattice.eval(rt, """
        let users = [
          {id: 1, name: "Alice", age: 30},
          {id: 2, name: "Bob", age: 18},
          {id: 3, name: "Charlie", age: 25}
        ]
      """)
      {:ok, result} = Lattice.eval(rt, ~s[SQL("SELECT * FROM users WHERE age > 21 ORDER BY id")])
      assert length(result) == 2
      assert hd(result)["name"] == "Alice"
      assert Enum.at(result, 1)["name"] == "Charlie"
    end

    test "sql aggregate on lattice data" do
      {:ok, rt} = Lattice.new(sql: true)
      {:ok, _} = Lattice.eval(rt, """
        let sales = [
          {product: "A", amount: 100},
          {product: "A", amount: 150},
          {product: "B", amount: 200}
        ]
      """)
      {:ok, result} = Lattice.eval(rt, ~s[SQL("SELECT SUM(amount) as total FROM sales")])
      assert hd(result)["total"] == 450
    end

    test "sql join lattice tables" do
      {:ok, rt} = Lattice.new(sql: true)
      {:ok, _} = Lattice.eval(rt, """
        let customers = [
          {id: 1, name: "Alice"},
          {id: 2, name: "Bob"}
        ]
        let orders = [
          {customer_id: 1, product: "Widget", amount: 100},
          {customer_id: 1, product: "Gadget", amount: 50},
          {customer_id: 2, product: "Widget", amount: 200}
        ]
      """)
      {:ok, result} = Lattice.eval(rt, """
        SQL("SELECT c.name, SUM(o.amount) as total
             FROM customers c
             JOIN orders o ON c.id = o.customer_id
             GROUP BY c.name
             ORDER BY c.name")
      """)
      assert length(result) == 2
      assert hd(result) == %{"name" => "Alice", "total" => 150}
      assert Enum.at(result, 1) == %{"name" => "Bob", "total" => 200}
    end

    test "sql table not found error" do
      {:ok, rt} = Lattice.new(sql: true)
      {:error, _reason} = Lattice.eval(rt, ~s[SQL("SELECT * FROM nonexistent")])
    end

    test "sql wrong type error" do
      {:ok, rt} = Lattice.new(sql: true)
      {:ok, _} = Lattice.eval(rt, ~s[let not_a_list = "hello"])
      {:error, _reason} = Lattice.eval(rt, ~s[SQL("SELECT * FROM not_a_list")])
    end
  end

  describe "SQL on Elixir Data" do
    test "sql on elixir list of maps via set_global" do
      {:ok, rt} = Lattice.new(sql: true)

      users = [
        %{"id" => 1, "name" => "Alice", "age" => 30},
        %{"id" => 2, "name" => "Bob", "age" => 18},
        %{"id" => 3, "name" => "Charlie", "age" => 25}
      ]
      :ok = Lattice.set_global(rt, "users", users)

      {:ok, result} = Lattice.eval(rt, ~s[SQL("SELECT * FROM users WHERE age >= 25 ORDER BY id")])

      assert length(result) == 2
      assert hd(result)["name"] == "Alice"
      assert Enum.at(result, 1)["name"] == "Charlie"
    end

    test "sql on elixir data with bindings" do
      {:ok, rt} = Lattice.new(sql: true)

      products = [
        %{"name" => "Widget", "price" => 9.99, "stock" => 100},
        %{"name" => "Gadget", "price" => 19.99, "stock" => 50},
        %{"name" => "Gizmo", "price" => 14.99, "stock" => 75}
      ]

      {:ok, result} = Lattice.eval_with_bindings(
        rt,
        ~s[SQL("SELECT name, price FROM products WHERE price > 10 ORDER BY price")],
        [{"products", products}]
      )

      assert length(result) == 2
      assert hd(result)["name"] == "Gizmo"
      assert Enum.at(result, 1)["name"] == "Gadget"
    end

    test "sql aggregate on elixir data" do
      {:ok, rt} = Lattice.new(sql: true)

      sales = [
        %{"region" => "North", "amount" => 1000},
        %{"region" => "South", "amount" => 1500},
        %{"region" => "North", "amount" => 800},
        %{"region" => "South", "amount" => 1200}
      ]
      :ok = Lattice.set_global(rt, "sales", sales)

      {:ok, result} = Lattice.eval(rt, """
        SQL("SELECT region, SUM(amount) as total, COUNT(*) as count
             FROM sales GROUP BY region ORDER BY region")
      """)

      assert length(result) == 2
      assert hd(result) == %{"region" => "North", "total" => 1800, "count" => 2}
      assert Enum.at(result, 1) == %{"region" => "South", "total" => 2700, "count" => 2}
    end

    test "sql join elixir and lattice data" do
      {:ok, rt} = Lattice.new(sql: true)

      # Elixir data
      orders = [
        %{"order_id" => 1, "customer_id" => 1, "amount" => 100},
        %{"order_id" => 2, "customer_id" => 2, "amount" => 200},
        %{"order_id" => 3, "customer_id" => 1, "amount" => 150}
      ]
      :ok = Lattice.set_global(rt, "orders", orders)

      # Lattice data
      {:ok, _} = Lattice.eval(rt, """
        let customers = [
          {id: 1, name: "Alice"},
          {id: 2, name: "Bob"}
        ]
      """)

      {:ok, result} = Lattice.eval(rt, """
        SQL("SELECT c.name, SUM(o.amount) as total
             FROM customers c
             JOIN orders o ON c.id = o.customer_id
             GROUP BY c.name
             ORDER BY total DESC")
      """)

      assert length(result) == 2
      assert hd(result) == %{"name" => "Alice", "total" => 250}
      assert Enum.at(result, 1) == %{"name" => "Bob", "total" => 200}
    end

    test "sql with null values" do
      {:ok, rt} = Lattice.new(sql: true)

      data = [
        %{"id" => 1, "value" => 100},
        %{"id" => 2, "value" => :null},
        %{"id" => 3, "value" => 300}
      ]
      :ok = Lattice.set_global(rt, "data", data)

      {:ok, result} = Lattice.eval(rt, ~s[SQL("SELECT * FROM data WHERE value IS NOT NULL ORDER BY id")])

      assert length(result) == 2
      assert hd(result)["id"] == 1
      assert Enum.at(result, 1)["id"] == 3
    end

    test "sql with boolean values" do
      {:ok, rt} = Lattice.new(sql: true)

      users = [
        %{"name" => "Alice", "active" => true},
        %{"name" => "Bob", "active" => false},
        %{"name" => "Charlie", "active" => true}
      ]
      :ok = Lattice.set_global(rt, "users", users)

      {:ok, result} = Lattice.eval(rt, ~s[SQL("SELECT name FROM users WHERE active = true ORDER BY name")])

      assert length(result) == 2
      assert hd(result)["name"] == "Alice"
      assert Enum.at(result, 1)["name"] == "Charlie"
    end
  end
end
