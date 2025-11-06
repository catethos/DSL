// Basic chart generation examples
// No LLM required - uses hardcoded data

// Example 1: Bar Chart
let salesData = [
    {label: "Q1", value: 100},
    {label: "Q2", value: 150},
    {label: "Q3", value: 120},
    {label: "Q4", value: 180}
]

let barChart = generateBarChart(salesData)
barChart

// Example 2: Line Chart
let temperatureData = [20, 22, 25, 23, 21, 19, 18, 20, 24, 26, 25, 23]
let lineChart = generateLineChart(temperatureData)
lineChart

// Example 3: Pie Chart
let marketShare = [
    {label: "Product A", value: 35},
    {label: "Product B", value: 25},
    {label: "Product C", value: 20},
    {label: "Product D", value: 15},
    {label: "Other", value: 5}
]

let pieChart = generatePieChart(marketShare)
pieChart
