// LLM + Chart Generation Example
// Requires OPENAI_API_KEY environment variable

// Define a type for chart data
type ChartData {
    label: String,
    value: Int
}

// Define a function to extract chart data from text using LLM
function ExtractSalesData(text: String) -> List<ChartData> {
    prompt: """
    Extract quarterly sales data from the following text and return as structured data:

    {{ text }}

    Return a list of {label, value} objects.
    """
}

// Sample text with sales data
let salesText = """
Our company had a great year!
- In Q1, we made $100,000 in revenue
- Q2 was even better with $150,000
- Q3 saw a slight dip to $120,000
- But Q4 was amazing with $180,000 in sales!
"""

// Extract data using our custom LLM function
let data = ExtractSalesData(salesText)

// Generate bar chart from extracted data
let chart = generateBarChart(data)
chart

// You can also ask LLM to generate analysis
let analysis = ask("""
Based on this sales data:
Q1: $100k, Q2: $150k, Q3: $120k, Q4: $180k

Provide a brief 2-sentence analysis of the trend.
""")

renderMarkdown(analysis)
