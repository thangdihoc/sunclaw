use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sunclaw_core::{CoreError, Tool, ToolResult};

pub struct WebSearchTool {
    api_key: String,
    client: reqwest::Client,
}

impl WebSearchTool {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Serialize)]
struct TavilyRequest {
    api_key: String,
    query: String,
    search_depth: String,
    include_answer: bool,
    max_results: usize,
}

#[derive(Deserialize)]
struct TavilyResponse {
    answer: Option<String>,
    results: Vec<TavilyResult>,
}

#[derive(Deserialize)]
struct TavilyResult {
    title: String,
    url: String,
    content: String,
}

#[async_trait]
impl Tool for WebSearchTool {
    fn name(&self) -> &'static str {
        "web_search"
    }

    async fn run(&self, input: &str) -> Result<ToolResult, CoreError> {
        if self.api_key == "secret" || self.api_key.is_empty() {
            return Ok(ToolResult {
                output: "[Mock Search] No API key provided. Search query was: ".to_string() + input,
            });
        }

        let request = TavilyRequest {
            api_key: self.api_key.clone(),
            query: input.to_string(),
            search_depth: "basic".to_string(),
            include_answer: true,
            max_results: 3,
        };

        let response = self
            .client
            .post("https://api.tavily.com/search")
            .json(&request)
            .send()
            .await
            .map_err(|e| CoreError::Tool(format!("Search HTTP error: {e}")))?;

        if !response.status().is_success() {
            return Err(CoreError::Tool(format!(
                "Search API error: {}",
                response.status()
            )));
        }

        let body: TavilyResponse = response
            .json()
            .await
            .map_err(|e| CoreError::Tool(format!("Search JSON error: {e}")))?;

        let mut output = String::new();
        if let Some(answer) = body.answer {
            output.push_str(&format!("Summary: {}\n\n", answer));
        }

        for (i, res) in body.results.iter().enumerate() {
            output.push_str(&format!("{}. [{}]({})\n{}\n\n", i + 1, res.title, res.url, res.content));
        }

        Ok(ToolResult { output })
    }
}
