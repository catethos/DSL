// Test inline lambda syntax

// Simple lambda
def test1() {
    let double = fn x => x * 2 end;
    double
}

// Lambda in function call (when map() is implemented)
// def test2() {
//     map([1, 2, 3], fn x => x * 2 end)
// }

// Multi-param lambda
def test3() {
    let add = fn x, y => x + y end;
    add
}
