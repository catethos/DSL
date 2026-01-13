import pytest
from pathlib import Path

from lattice import Runtime


class TestBasicEval:
    def test_arithmetic(self):
        rt = Runtime()
        assert rt.eval("1 + 2 * 3") == 7

    def test_string(self):
        rt = Runtime()
        assert rt.eval('"hello" + " world"') == "hello world"

    def test_list(self):
        rt = Runtime()
        assert rt.eval("[1, 2, 3]") == [1, 2, 3]

    def test_map(self):
        rt = Runtime()
        result = rt.eval('{"a": 1, "b": 2}')
        assert result == {"a": 1, "b": 2}

    def test_null(self):
        rt = Runtime()
        assert rt.eval("null") is None

    def test_bool(self):
        rt = Runtime()
        assert rt.eval("true") is True
        assert rt.eval("false") is False


class TestBindings:
    def test_eval_with_bindings(self):
        rt = Runtime()
        result = rt.eval("x + y", bindings={"x": 10, "y": 20})
        assert result == 30

    def test_bindings_with_list(self):
        rt = Runtime()
        result = rt.eval("items[0] + items[1]", bindings={"items": [3, 4]})
        assert result == 7


class TestGlobals:
    def test_set_get_global(self):
        rt = Runtime()
        rt.set_global("count", 42)
        assert rt.get_global("count") == 42

    def test_global_in_eval(self):
        rt = Runtime()
        rt.set_global("x", 100)
        assert rt.eval("x * 2") == 200

    def test_dict_syntax(self):
        rt = Runtime()
        rt["foo"] = 123
        assert rt["foo"] == 123

    def test_contains(self):
        rt = Runtime()
        rt["bar"] = 1
        assert "bar" in rt
        assert "baz" not in rt


class TestFunctions:
    def test_define_and_call(self):
        rt = Runtime()
        rt.eval("def add(a: Int, b: Int) -> Int { a + b }")
        assert rt.call("add", 3, 4) == 7

    def test_has_function(self):
        rt = Runtime()
        rt.eval("def foo() -> Int { 42 }")
        assert rt.has_function("foo")
        assert not rt.has_function("bar")

    def test_function_signatures(self):
        rt = Runtime()
        rt.eval("def greet(name: String) -> String { name }")
        sigs = rt.get_function_signatures()
        assert len(sigs) == 1
        assert sigs[0]["name"] == "greet"


class TestTypes:
    def test_struct(self):
        rt = Runtime()
        rt.eval("type Person { name: String, age: Int }")
        types = rt.get_types()
        assert len(types) == 1
        assert types[0]["type"] == "struct"
        assert types[0]["name"] == "Person"

    def test_enum(self):
        rt = Runtime()
        rt.eval("enum Color { Red, Green, Blue }")
        types = rt.get_types()
        assert len(types) == 1
        assert types[0]["type"] == "enum"
        assert types[0]["variants"] == ["Red", "Green", "Blue"]


class TestContextManager:
    def test_context_manager_reset(self):
        rt = Runtime()
        rt["x"] = 42
        with rt:
            assert rt["x"] == 42
        # After exiting, runtime is reset
        assert rt["x"] is None


class TestErrors:
    def test_syntax_error(self):
        rt = Runtime()
        with pytest.raises(RuntimeError):
            rt.eval("1 +")

    def test_undefined_variable(self):
        rt = Runtime()
        with pytest.raises(RuntimeError):
            rt.eval("undefined_var")

    def test_undefined_function(self):
        rt = Runtime()
        with pytest.raises(RuntimeError):
            rt.call("nonexistent")


class TestRepr:
    def test_repr(self):
        rt = Runtime()
        repr_str = repr(rt)
        assert "Runtime" in repr_str
        assert "functions" in repr_str


class TestSQLFeature:
    def test_sql_runtime_creation(self):
        """Test that SQL runtime can be created"""
        rt = Runtime(sql=True)
        assert rt is not None

    def test_sql_basic_query(self):
        """Test basic SQL query execution"""
        rt = Runtime(sql=True)
        result = rt.eval('SQL("SELECT 1 + 1 as result")')
        # Result is a list with one row
        assert result is not None
        # DuckDB returns a list of maps
        assert isinstance(result, list)
        assert len(result) == 1
        assert result[0]["result"] == 2

    def test_sql_multiple_rows(self):
        """Test SQL query returning multiple rows"""
        rt = Runtime(sql=True)
        result = rt.eval('''
SQL("SELECT * FROM (
    SELECT 1 as id, 'Alice' as name
    UNION ALL
    SELECT 2 as id, 'Bob' as name
    UNION ALL
    SELECT 3 as id, 'Carol' as name
)")
''')
        assert len(result) == 3
        names = [row["name"] for row in result]
        assert names == ["Alice", "Bob", "Carol"]

    def test_sql_aggregation(self):
        """Test SQL aggregation functions"""
        rt = Runtime(sql=True)
        result = rt.eval('''
SQL("SELECT COUNT(*) as cnt, SUM(x) as total, AVG(x) as avg FROM (
    SELECT 10 as x UNION ALL SELECT 20 UNION ALL SELECT 30
)")
''')
        assert len(result) == 1
        row = result[0]
        assert row["cnt"] == 3
        assert row["total"] == 60
        assert row["avg"] == 20.0

    def test_sql_with_boolean(self):
        """Test SQL with boolean values"""
        rt = Runtime(sql=True)
        result = rt.eval('''
SQL("
    SELECT 1 as id, 'Alice' as name, true as active
    UNION ALL
    SELECT 2 as id, 'Bob' as name, false as active
")
''')
        assert len(result) == 2
        assert result[0]["id"] == 1
        assert result[0]["name"] == "Alice"
        assert result[0]["active"] == True
        assert result[1]["id"] == 2
        assert result[1]["name"] == "Bob"
        assert result[1]["active"] == False

    def test_sql_with_interpolation(self):
        """Test SQL with f-string interpolation"""
        rt = Runtime(sql=True)
        result = rt.eval('''
let min_id = 2
let query = f"SELECT * FROM (
    SELECT 1 as id, 'Alice' as name
    UNION ALL SELECT 2 as id, 'Bob' as name
    UNION ALL SELECT 3 as id, 'Carol' as name
) WHERE id >= {min_id}"
SQL(query)
''')
        assert len(result) == 2
        names = [row["name"] for row in result]
        assert "Bob" in names
        assert "Carol" in names
        assert "Alice" not in names


class TestLLMFeature:
    def test_llm_runtime_creation(self):
        """Test that LLM runtime can be created (requires OPENROUTER_API_KEY env var)"""
        import os
        if not os.environ.get("OPENROUTER_API_KEY") and not os.environ.get("TEST_API_KEY"):
            pytest.skip("No API key available for LLM testing")

        rt = Runtime(llm=True)
        assert rt is not None

    def test_take_llm_debug_none(self):
        """take_llm_debug returns None when no LLM call was made"""
        rt = Runtime()
        assert rt.take_llm_debug() is None

    def test_llm_function_definition(self):
        """Test that LLM functions can be defined (but not called without API key)"""
        import os
        if not os.environ.get("OPENROUTER_API_KEY") and not os.environ.get("TEST_API_KEY"):
            pytest.skip("No API key available for LLM testing")

        rt = Runtime(llm=True)
        # Define an LLM function using Lattice syntax
        # LLM functions have special config fields: base_url, model, api_key_env, prompt
        rt.eval('''
def summarize(text: String) -> String {
    base_url: "https://openrouter.ai/api/v1"
    model: "openai/gpt-4o-mini"
    api_key_env: "TEST_API_KEY"
    prompt: "Summarize: ${text}"
}
''')
        # Check it's registered as an LLM function
        sigs = rt.get_function_signatures()
        llm_sigs = [s for s in sigs if s["is_llm"]]
        assert len(llm_sigs) == 1
        assert llm_sigs[0]["name"] == "summarize"


class TestLLMIntegration:
    """Integration tests that actually call the LLM API.

    These tests require OPENROUTER_API_KEY to be set in the environment.
    They make real API calls and may incur costs.
    """

    def test_llm_simple_string_response(self):
        """Test LLM function that returns a simple string"""
        import os
        api_key = os.environ.get("OPENROUTER_API_KEY")
        if not api_key:
            pytest.skip("OPENROUTER_API_KEY not set")

        rt = Runtime(llm=True)
        result = rt.eval('''
def say_hello(name: String) -> String {
    base_url: "https://openrouter.ai/api/v1"
    model: "openai/gpt-4o-mini"
    api_key_env: "OPENROUTER_API_KEY"
    temperature: 0.0
    prompt: "Say hello to ${name}. Respond with ONLY a greeting, nothing else."
}

say_hello("World")
''')
        assert isinstance(result, str)
        assert len(result) > 0
        # The response should contain some form of greeting
        result_lower = result.lower()
        assert "hello" in result_lower or "hi" in result_lower or "world" in result_lower

    def test_llm_enum_response(self):
        """Test LLM function that returns an enum value"""
        import os
        api_key = os.environ.get("OPENROUTER_API_KEY")
        if not api_key:
            pytest.skip("OPENROUTER_API_KEY not set")

        rt = Runtime(llm=True)
        result = rt.eval('''
enum Sentiment {
    Positive,
    Negative,
    Neutral
}

def analyze_sentiment(text: String) -> Sentiment {
    base_url: "https://openrouter.ai/api/v1"
    model: "openai/gpt-4o-mini"
    api_key_env: "OPENROUTER_API_KEY"
    temperature: 0.0
    prompt: "Analyze the sentiment of this text: ${text}"
}

analyze_sentiment("I love this product! It's amazing!")
''')
        # Enum values are returned as strings
        assert result in ["Positive", "Negative", "Neutral"]
        # This text should be positive
        assert result == "Positive"

    def test_llm_struct_response(self):
        """Test LLM function that returns a structured type"""
        import os
        api_key = os.environ.get("OPENROUTER_API_KEY")
        if not api_key:
            pytest.skip("OPENROUTER_API_KEY not set")

        rt = Runtime(llm=True)
        result = rt.eval('''
type MathResult {
    answer: Int,
    explanation: String
}

def solve_math(problem: String) -> MathResult {
    base_url: "https://openrouter.ai/api/v1"
    model: "openai/gpt-4o-mini"
    api_key_env: "OPENROUTER_API_KEY"
    temperature: 0.0
    prompt: "Solve this math problem: ${problem}"
}

solve_math("What is 2 + 2?")
''')
        assert isinstance(result, dict)
        assert "answer" in result
        assert "explanation" in result
        assert result["answer"] == 4

    def test_llm_debug_info(self):
        """Test that LLM debug info is captured"""
        import os
        api_key = os.environ.get("OPENROUTER_API_KEY")
        if not api_key:
            pytest.skip("OPENROUTER_API_KEY not set")

        rt = Runtime(llm=True)
        rt.eval('''
def echo(text: String) -> String {
    base_url: "https://openrouter.ai/api/v1"
    model: "openai/gpt-4o-mini"
    api_key_env: "OPENROUTER_API_KEY"
    temperature: 0.0
    prompt: "Echo back: ${text}"
}

echo("test message")
''')
        debug = rt.take_llm_debug()
        assert debug is not None
        assert "prompt" in debug
        assert "raw_response" in debug
        assert "function_name" in debug
        assert "return_type" in debug
        assert debug["function_name"] == "echo"
        assert "test message" in debug["prompt"]

    def test_llm_list_response(self):
        """Test LLM function that returns a list"""
        import os
        api_key = os.environ.get("OPENROUTER_API_KEY")
        if not api_key:
            pytest.skip("OPENROUTER_API_KEY not set")

        rt = Runtime(llm=True)
        result = rt.eval('''
def list_colors(count: Int) -> [String] {
    base_url: "https://openrouter.ai/api/v1"
    model: "openai/gpt-4o-mini"
    api_key_env: "OPENROUTER_API_KEY"
    temperature: 0.0
    prompt: "List exactly ${count} color names. Return only the colors as a list."
}

list_colors(3)
''')
        assert isinstance(result, list)
        assert len(result) == 3
        for color in result:
            assert isinstance(color, str)
