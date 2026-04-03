use async_trait::async_trait;
use sunclaw_core::{Tool, ToolResult, CoreError};
use serde::{Deserialize, Serialize};
use tokio::process::Command;
use std::process::Stdio;

#[derive(Debug, Serialize, Deserialize)]
pub struct McpToolConfig {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
}

pub struct McpTool {
    config: McpToolConfig,
}

impl McpTool {
    pub fn new(config: McpToolConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Tool for McpTool {
    fn name(&self) -> &'static str {
        // Trait yêu cầu &'static str, chúng ta cần leak chuỗi này vì nó được tạo động
        Box::leak(self.config.name.clone().into_boxed_str())
    }

    fn definition(&self) -> sunclaw_core::ToolDefinition {
        sunclaw_core::ToolDefinition {
            name: self.config.name.clone(),
            description: format!("MCP Tool: {}", self.config.name),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "input": { "type": "string" }
                }
            }),
        }
    }

    async fn run(&self, input: &str) -> Result<ToolResult, CoreError> {
        // Giả lập gọi MCP server qua stdio (Đây là placeholder cho rmcp real logic)
        let mut _child = Command::new(&self.config.command)
            .args(&self.config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|e| CoreError::Tool(format!("Failed to spawn MCP server: {}", e)))?;

        // Ở đây sẽ có logic JSON-RPC qua stdio...
        
        Ok(ToolResult {
            output: format!("MCP Tool result placeholder for {} with input {}", self.config.name, input),
        })
    }
}

pub fn get_default_mcp_configs() -> Vec<McpToolConfig> {
    vec![
        McpToolConfig {
            name: "brave_search".to_string(),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-brave-search".to_string()],
        },
        McpToolConfig {
            name: "fetch".to_string(),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-fetch".to_string()],
        },
    ]
}
