# String utility functions

def concat(a, b) => a + b end

def repeat(str, n) => 
  match n {
    0 => "",
    1 => str,
    _ => str + repeat(str, n - 1)
  }
end
