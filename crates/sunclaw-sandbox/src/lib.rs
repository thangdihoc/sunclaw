use async_trait::async_trait;
use sunclaw_core::{Tool, ToolDefinition, ToolResult, CoreError};
use extism::Plugin;
use serde_json::Value;

/// A tool that runs in a WebAssembly sandbox using Extism.
pub struct SandboxTool {
    name: String,
    description: String,
    wasm_bytes: Vec<u8>,
    manifest_json: Value,
}

impl SandboxTool {
    pub fn new(name: &str, description: &str, wasm_bytes: Vec<u8>) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            wasm_bytes,
            manifest_json: serde_json::json!({
                "type": "object",
                "properties": {
                    "input": { "type": "string" }
                }
            }),
        }
    }
}

#[async_trait]
impl Tool for SandboxTool {
    fn name(&self) -> &'static str {
        Box::leak(self.name.clone().into_boxed_str())
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name.clone(),
            description: self.description.clone(),
            parameters: self.manifest_json.clone(),
        }
    }

    async fn run(&self, input: &str) -> Result<ToolResult, CoreError> {
        let mut plugin = Plugin::new(&self.wasm_bytes, [], true)
            .map_err(|e| CoreError::Tool(format!("Failed to create WASM plugin: {}", e)))?;

        let output = plugin.call::<&str, &str>("run", input)
            .map_err(|e| CoreError::Tool(format!("WASM execution error: {}", e)))?;

        Ok(ToolResult::simple(&output))
    }
}
