type People {
    name: String,
    age: Int,
    address: String,
    email: String}

def test(x) -> People {
prompt: "extract the  information from ${x}"}

test("regina is 30 yo living in Sydney, with email regina@example.com")
