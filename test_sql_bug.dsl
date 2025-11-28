let q = SQL("SELECT * FROM '/Users/catethos/Downloads/verbal_testset15.csv'")
print(q)

let result = SQL("SELECT * FROM $q")
print(result)
