---
name: analyze_sentiment
model: gpt-4o-mini
temperature: 0.0
input:
  text: String
output: Sentiment
---
Analyze the sentiment of the following text and classify it as one of:
- Positive: Expresses positive emotions
- Negative: Expresses negative emotions
- Neutral: Factual or objective
- Mixed: Contains both positive and negative

Text to analyze:
{text}

Return only the sentiment classification.
