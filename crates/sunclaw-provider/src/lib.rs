use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use sunclaw_core::{AgentContext, AgentRole, CoreError, Decision, Message, ModelProvider};

pub mod openai;
pub use openai::OpenAIProvider;

#[derive(Clone)]
pub struct RetryProvider {
    inner: Arc<dyn ModelProvider>,
    max_retries: usize,
}

impl RetryProvider {
    pub fn new(inner: Arc<dyn ModelProvider>, max_retries: usize) -> Self {
        Self { inner, max_retries }
    }
}

#[async_trait]
impl ModelProvider for RetryProvider {
    async fn decide(
        &self,
        ctx: &AgentContext,
        messages: &[Message],
        tools: &[sunclaw_core::ToolDefinition],
    ) -> Result<Decision, CoreError> {
        let mut last_err = None;

        for attempt in 0..=self.max_retries {
            match self.inner.decide(ctx, messages, tools).await {
                Ok(decision) => return Ok(decision),
                Err(e) => {
                    last_err = Some(e);
                    if attempt < self.max_retries {
                        // Exponential backoff or simple delay can be added here
                        tokio::time::sleep(std::time::Duration::from_millis(500 * (attempt as u64 + 1))).await;
                    }
                }
            }
        }

        Err(last_err.unwrap_or_else(|| CoreError::Provider("Retries failed".to_string())))
    }
}

#[derive(Debug, Clone)]
pub struct ModelRoute {
    pub name: String,
    pub backends: Vec<String>,
}

impl ModelRoute {
    pub fn new(name: impl Into<String>, backends: Vec<String>) -> Self {
        Self {
            name: name.into(),
            backends,
        }
    }
}

pub struct MultiProvider {
    default_route: String,
    routes: HashMap<String, ModelRoute>,
    backends: HashMap<String, Arc<dyn ModelProvider>>,
}

impl MultiProvider {
    pub fn new(default_route: impl Into<String>) -> Self {
        Self {
            default_route: default_route.into(),
            routes: HashMap::new(),
            backends: HashMap::new(),
        }
    }

    pub fn add_route(&mut self, route: ModelRoute) {
        self.routes.insert(route.name.clone(), route);
    }

    pub fn add_backend(&mut self, id: impl Into<String>, backend: Arc<dyn ModelProvider>) {
        self.backends.insert(id.into(), backend);
    }

    fn choose_route<'a>(&'a self, ctx: &'a AgentContext) -> Result<&'a ModelRoute, CoreError> {
        let route_name = if let Some(AgentRole::Planner) = ctx.role {
            "reasoning"
        } else {
            ctx.model_profile.as_ref().unwrap_or(&self.default_route)
        };

        self.routes
            .get(route_name)
            .or_else(|| self.routes.get(&self.default_route))
            .ok_or_else(|| CoreError::Provider(format!("unknown model route: {route_name}")))
    }
}

#[async_trait]
impl ModelProvider for MultiProvider {
    async fn decide(
        &self,
        ctx: &AgentContext,
        messages: &[Message],
        tools: &[sunclaw_core::ToolDefinition],
    ) -> Result<Decision, CoreError> {
        let route = self.choose_route(ctx)?;
        let mut provider_errors = Vec::new();

        for backend_id in &route.backends {
            let Some(backend) = self.backends.get(backend_id) else {
                provider_errors.push(format!("missing backend: {backend_id}"));
                continue;
            };

            match backend.decide(ctx, messages, tools).await {
                Ok(decision) => return Ok(decision),
                Err(CoreError::Provider(err)) => {
                    provider_errors.push(format!("{backend_id}: {err}"))
                }
                Err(other) => return Err(other),
            }
        }

        Err(CoreError::Provider(format!(
            "all backends failed for route '{}': {}",
            route.name,
            provider_errors.join(" | ")
        )))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::sync::Mutex;

    use super::*;

    struct StubProvider {
        responses: Mutex<VecDeque<Result<Decision, CoreError>>>,
    }

    #[async_trait]
    impl ModelProvider for StubProvider {
        async fn decide(
            &self,
            _ctx: &AgentContext,
            _messages: &[Message],
            _tools: &[sunclaw_core::ToolDefinition],
        ) -> Result<Decision, CoreError> {
            self.responses
                .lock()
                .map_err(|e| CoreError::Provider(format!("lock error: {e}")))?
                .pop_front()
                .unwrap_or_else(|| Err(CoreError::Provider("no response".to_string())))
        }
    }

    #[tokio::test]
    async fn falls_back_to_secondary_provider() {
        let mut router = MultiProvider::new("default");
        router.add_route(ModelRoute::new(
            "default",
            vec!["openrouter:deepseek".to_string(), "xai:grok".to_string()],
        ));

        router.add_backend(
            "openrouter:deepseek",
            Arc::new(StubProvider {
                responses: Mutex::new(VecDeque::from(vec![Err(CoreError::Provider(
                    "rate limited".to_string(),
                ))])),
            }),
        );

        router.add_backend(
            "xai:grok",
            Arc::new(StubProvider {
                responses: Mutex::new(VecDeque::from(vec![Ok(Decision::Reply(
                    "fallback works".to_string(),
                ))])),
            }),
        );

        let out = router
            .decide(
                &AgentContext {
                    trace_id: "t".to_string(),
                    skill: None,
                    model_profile: Some("default".to_string()),
                    role: None,
                    max_tokens: None,
                },
                &[],
                &[],
            )
            .await
            .expect("fallback should succeed");

        assert_eq!(out, Decision::Reply("fallback works".to_string()));
    }
}
