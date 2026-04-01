use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sunclaw_core::{AgentContext, CoreError, Decision, Message, ModelProvider, Role, ToolCall};

pub struct OpenAIProvider {
    client: reqwest::Client,
    api_key: String,
    model_id: String,
    endpoint: String,
}

impl OpenAIProvider {
    pub fn new(
        api_key: impl Into<String>,
        model_id: impl Into<String>,
        endpoint: Option<String>,
    ) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.into(),
            model_id: model_id.into(),
            endpoint: endpoint.unwrap_or_else(|| "https://openrouter.ai/api/v1".to_string()),
        }
    }
}

#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatCompletionMessage>,
}

#[derive(Serialize)]
struct ChatCompletionMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: Option<String>,
    tool_calls: Option<Vec<ResponseToolCall>>,
}

#[derive(Deserialize)]
struct ResponseToolCall {
    function: FunctionCall,
}

#[derive(Deserialize)]
struct FunctionCall {
    name: String,
    arguments: String,
}

#[async_trait]
impl ModelProvider for OpenAIProvider {
    async fn decide(
        &self,
        _ctx: &AgentContext,
        messages: &[Message],
    ) -> Result<Decision, CoreError> {
        let req_messages = messages
            .iter()
            .map(|m| ChatCompletionMessage {
                role: match m.role {
                    Role::User => "user".to_string(),
                    Role::Agent => "assistant".to_string(),
                    Role::System => "system".to_string(),
                },
                content: m.content.clone(),
            })
            .collect();

        let request = ChatCompletionRequest {
            model: self.model_id.clone(),
            messages: req_messages,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.endpoint))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| CoreError::Provider(format!("HTTP error: {e}")))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(CoreError::Provider(format!(
                "API error ({}): {}",
                response.status(),
                error_text
            )));
        }

        let body: ChatCompletionResponse = response
            .json()
            .await
            .map_err(|e| CoreError::Provider(format!("JSON error: {e}")))?;

        let choice = body
            .choices
            .first()
            .ok_or_else(|| CoreError::Provider("No choices in response".to_string()))?;

        if let Some(tool_calls) = &choice.message.tool_calls {
            if let Some(tc) = tool_calls.first() {
                return Ok(Decision::UseTool(ToolCall {
                    name: tc.function.name.clone(),
                    input: tc.function.arguments.clone(),
                }));
            }
        }

        let content = choice
            .message
            .content
            .clone()
            .unwrap_or_else(|| "".to_string());

        Ok(Decision::Reply(content))
    }
}
