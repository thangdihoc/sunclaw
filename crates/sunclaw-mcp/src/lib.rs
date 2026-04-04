use async_trait::async_trait;
use sunclaw_core::{Tool, ToolResult, CoreError};
use serde::{Deserialize, Serialize};
use tokio::process::Command;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct McpToolConfig {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
}

impl McpToolConfig {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
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
        Box::leak(self.config.name.clone().into_boxed_str())
    }

    fn definition(&self) -> sunclaw_core::ToolDefinition {
        sunclaw_core::ToolDefinition {
            name: self.config.name.clone(),
            description: format!("MCP Tool: {}", self.config.name),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "arguments": { "type": "object" }
                }
            }),
        }
    }

    async fn run(&self, input: &str) -> Result<ToolResult, CoreError> {
        let mut child = Command::new(&self.config.command)
            .args(&self.config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| CoreError::Tool(format!("Failed to spawn MCP server {}: {}", self.config.name, e)))?;

        let mut stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let mut reader = BufReader::new(stdout).lines();

        // 1. Initialize MCP (Simplified JSON-RPC)
        let init_msg = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "sunclaw", "version": "0.2.0" }
            }
        });
        
        stdin.write_all(format!("{}\n", init_msg).as_bytes()).await
            .map_err(|e| CoreError::Tool(e.to_string()))?;

        // Wait for init response
        let _ = reader.next_line().await.map_err(|e| CoreError::Tool(e.to_string()))?;

        // 2. Call tool
        let args: serde_json::Value = serde_json::from_str(input).unwrap_or(serde_json::json!({}));
        let call_msg = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": self.config.name,
                "arguments": args["arguments"]
            }
        });

        stdin.write_all(format!("{}\n", call_msg).as_bytes()).await
            .map_err(|e| CoreError::Tool(e.to_string()))?;

        if let Some(line) = reader.next_line().await.map_err(|e| CoreError::Tool(e.to_string()))? {
            let res: serde_json::Value = serde_json::from_str(&line)
                .map_err(|e| CoreError::Tool(format!("Invalid MCP response: {}", e)))?;
            
            let output = res["result"]["content"][0]["text"].as_str()
                .unwrap_or("No output from MCP tool").to_string();

            return Ok(ToolResult::simple(&output));
        }

        Err(CoreError::Tool("MCP server closed stream early".into()))
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
