# Formatting Test File
# This file tests the auto-formatting features

# Unformatted function
def unformatted(x){let y=x*2
let z=y+1
z}

# Another unformatted example
type Person{name:String,age:Int,email:String}

# Nested blocks
def complex(a,b){let inner={x:a,y:b}
let computed=match inner{
{x,y}=>x+y
}
computed}

# Match expression
def categorize(n){match n{
x if x<10=>"small"
x if x<100=>"medium"
_=>"large"
}}
