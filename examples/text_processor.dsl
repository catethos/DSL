arg2 as text |> arg1 as operation |>
  operation == "upper" ? upper(text) :
  operation == "lower" ? lower(text) :
  operation == "length" ? length(text) :
  "Unknown operation"
