use reqwest::Client;
use scraper::{Html, Selector};
use crate::error::AppError;

pub async fn extract_content(url: &str) -> Result<(String, String), AppError> {
    let client = Client::new();
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| AppError::ExternalServiceError(e.to_string()))?;

    let html = response
        .text()
        .await
        .map_err(|e| AppError::ExternalServiceError(e.to_string()))?;

    let document = Html::parse_document(&html);
    
    // Extract title
    let title = document
        .select(&Selector::parse("title").unwrap())
        .next()
        .map(|element| element.text().collect::<String>())
        .unwrap_or_else(|| String::from("Untitled"));

    // Extract preview content
    let preview = document
        .select(&Selector::parse("meta[name='description']").unwrap())
        .next()
        .and_then(|element| element.value().attr("content"))
        .unwrap_or_else(|| {
            document
                .select(&Selector::parse("p").unwrap())
                .take(2)
                .map(|element| element.text().collect::<String>())
                .collect::<Vec<String>>()
                .join("\n")
                .to_string()
        });

    Ok((title, preview))
}

pub async fn generate_summary(content: &str) -> Result<String, AppError> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| AppError::ExternalServiceError("OpenAI API key not found".to_string()))?;

    let client = Client::new();
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&serde_json::json!({
            "model": "gpt-3.5-turbo",
            "messages": [
                {
                    "role": "system",
                    "content": "You are a helpful assistant that summarizes web content."
                },
                {
                    "role": "user",
                    "content": format!("Please provide a concise summary of the following content:\n\n{}", content)
                }
            ],
            "max_tokens": 150
        }))
        .send()
        .await
        .map_err(|e| AppError::ExternalServiceError(e.to_string()))?;

    let response_data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| AppError::ExternalServiceError(e.to_string()))?;

    let summary = response_data["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| AppError::ExternalServiceError("Failed to extract summary".to_string()))?
        .to_string();

    Ok(summary)
}