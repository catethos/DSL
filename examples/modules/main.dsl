# Main program demonstrating module imports

import "./math.dsl"
import "./string_utils.dsl"

# Use functions from math module
let result = add(10, 5)
let squared = square(4)

# Use functions from string_utils module
let greeting = concat("Hello", " World!")

# Output the greeting
greeting
