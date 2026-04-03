use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sunclaw_core::{sunclaw_tool, CoreError, Tool, ToolResult};
use schemars::JsonSchema;

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

#[derive(Deserialize, JsonSchema)]
pub struct SearchArgs {
    /// Từ khóa tìm kiếm trên internet.
    pub query: String,
}

sunclaw_tool!(
    WebSearchTool,
    SearchArgs,
    "web_search",
    "Tìm kiếm thông tin trên internet để trả lời các câu hỏi về tin tức, sự kiện hiện tại hoặc kiến thức chuyên sâu.",
    self_obj,
    args,
    {
        if self_obj.api_key == "secret" || self_obj.api_key.is_empty() {
            return Ok(ToolResult {
                output: format!("[Mock Search] No API key provided. Search query was: {}", args.query),
            });
        }

        let request = TavilyRequest {
            api_key: self_obj.api_key.clone(),
            query: args.query.clone(),
            search_depth: "basic".to_string(),
            include_answer: true,
            max_results: 3,
        };

        let response = self_obj
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
);
