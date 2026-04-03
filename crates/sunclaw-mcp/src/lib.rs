use async_trait::async_trait;
use sunclaw_core::{Tool, ToolResult, CoreError};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
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
    fn name(&self) -> String {
        self.config.name.clone()
    }

    fn description(&self) -> String {
        format!("MCP Tool: {}", self.config.name)
    }

    fn definition(&self) -> serde_json::Value {
        // Tương lai sẽ lấy động từ MCP server
        serde_json::json!({
            "type": "function",
            "function": {
                "name": self.config.name,
                "description": self.description(),
                "parameters": {
                    "type": "object",
                    "properties": {
                        "input": { "type": "string" }
                    }
                }
            }
        })
    }

    async fn call(&self, input: serde_json::Value) -> Result<ToolResult, CoreError> {
        // Giả lập gọi MCP server qua stdio (Đây là placeholder cho rmcp real logic)
        let mut child = Command::new(&self.config.command)
            .args(&self.config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|e| CoreError::ToolError(format!("Failed to spawn MCP server: {}", e)))?;

        // Ở đây sẽ có logic JSON-RPC qua stdio...
        
        Ok(ToolResult {
            output: format!("MCP Tool result placeholder for {}", self.config.name),
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
