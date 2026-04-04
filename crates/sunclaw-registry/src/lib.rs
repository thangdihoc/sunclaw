use std::path::Path;
use std::sync::Arc;
use walkdir::WalkDir;
use sunclaw_core::{Tool, CoreError};
use sunclaw_sandbox::SandboxTool;
use sunclaw_mcp::{McpTool, McpToolConfig};

pub struct Registry;

impl Registry {
    /// Quét thư mục plugins và nạp toàn bộ Tool tìm thấy.
    /// - *.wasm => SandboxTool
    /// - *.json => McpTool (nếu đúng schema)
    pub fn load_tools<P: AsRef<Path>>(dir: P) -> Result<Vec<Arc<dyn Tool>>, CoreError> {
        let mut tools: Vec<Arc<dyn Tool>> = Vec::new();
        let path = dir.as_ref();

        if !path.exists() {
            tracing::warn!("Plugins directory not found: {:?}", path);
            return Ok(tools);
        }

        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            let entry_path = entry.path();
            if entry_path.is_file() {
                if let Some(ext) = entry_path.extension() {
                    match ext.to_str() {
                        Some("wasm") => {
                            if let Ok(wasm_bytes) = std::fs::read(entry_path) {
                                let name = entry_path.file_stem().unwrap().to_str().unwrap_or("unknown");
                                let tool = SandboxTool::new(name, &format!("WASM Tool: {}", name), wasm_bytes);
                                tools.push(Arc::new(tool));
                                tracing::info!("Loaded WASM Tool: {}", name);
                            }
                        }
                        Some("json") => {
                            if let Ok(content) = std::fs::read_to_string(entry_path) {
                                if let Ok(config) = serde_json::from_str::<McpToolConfig>(&content) {
                                    let tool = McpTool::new(config.clone());
                                    tools.push(Arc::new(tool));
                                    tracing::info!("Loaded MCP Tool: {}", config.name);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(tools)
    }
}
