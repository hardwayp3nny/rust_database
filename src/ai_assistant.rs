use reqwest::Client;
use serde_json::{json, Value};
use std::error::Error;

pub struct AIAssistant {
    client: Client,
    api_key: String,
}

impl AIAssistant {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn natural_to_sql(&self, natural_query: &str) -> Result<String, Box<dyn Error>> {
        let prompt = format!(
            "將以下自然語言轉換為SQL查詢：\n{}\n只返回SQL語句，不要其他解釋。",
            natural_query
        );

        let response = self.query_openai(&prompt).await?;
        Ok(response)
    }

    pub async fn analyze_data(&self, data: &str) -> Result<String, Box<dyn Error>> {
        let prompt = format!(
            "分析以下數據並提供見解：\n{}\n請提供簡潔的數據分析結果。",
            data
        );

        let response = self.query_openai(&prompt).await?;
        Ok(response)
    }

    async fn query_openai(&self, prompt: &str) -> Result<String, Box<dyn Error>> {
        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&json!({
                "model": "gpt-3.5-turbo",
                "messages": [
                    {
                        "role": "user",
                        "content": prompt
                    }
                ],
                "temperature": 0.7
            }))
            .send()
            .await?;

        let response_json: Value = response.json().await?;
        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(content)
    }
} 