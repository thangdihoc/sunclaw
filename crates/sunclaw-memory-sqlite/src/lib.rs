use async_trait::async_trait;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use sunclaw_core::{AuditDecision, AuditEvent, AuditStore, CoreError, MemoryStore, Message, Role};

pub struct SqliteStore {
    pool: Pool<Sqlite>,
}

impl SqliteStore {
    pub async fn new(database_url: &str) -> Result<Self, CoreError> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await
            .map_err(|e| CoreError::Memory(format!("Failed to connect to SQLite: {e}")))?;

        let store = Self { pool };
        store.init().await?;
        Ok(store)
    }

    async fn init(&self) -> Result<(), CoreError> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                trace_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Memory(format!("Failed to create messages table: {e}")))?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS audit_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                trace_id TEXT NOT NULL,
                skill TEXT,
                tool_name TEXT NOT NULL,
                decision TEXT NOT NULL,
                reason TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Memory(format!("Failed to create audit_events table: {e}")))?;

        Ok(())
    }
}

#[async_trait]
impl MemoryStore for SqliteStore {
    async fn load_messages(&self, trace_id: &str) -> Result<Vec<Message>, CoreError> {
        let rows = sqlx::query_as::<_, MessageRow>(
            "SELECT role, content FROM messages WHERE trace_id = ? ORDER BY created_at ASC",
        )
        .bind(trace_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Memory(format!("Failed to load messages: {e}")))?;

        Ok(rows.into_iter().map(|r| Message {
            role: match r.role.as_str() {
                "user" => Role::User,
                "agent" => Role::Agent,
                "system" => Role::System,
                _ => Role::System,
            },
            content: r.content,
        }).collect())
    }

    async fn append_message(&self, trace_id: &str, message: Message) -> Result<(), CoreError> {
        let role_str = match message.role {
            Role::User => "user",
            Role::Agent => "agent",
            Role::System => "system",
        };

        sqlx::query("INSERT INTO messages (trace_id, role, content) VALUES (?, ?, ?)")
            .bind(trace_id)
            .bind(role_str)
            .bind(message.content)
            .execute(&self.pool)
            .await
            .map_err(|e| CoreError::Memory(format!("Failed to append message: {e}")))?;

        Ok(())
    }

    async fn list_traces(&self) -> Result<Vec<String>, CoreError> {
        let rows = sqlx::query_as::<_, TraceRow>(
            "SELECT DISTINCT trace_id FROM messages ORDER BY id DESC LIMIT 50",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Memory(format!("Failed to list traces: {e}")))?;

        Ok(rows.into_iter().map(|r| r.trace_id).collect())
    }
}

#[async_trait]
impl AuditStore for SqliteStore {
    async fn append_event(&self, event: AuditEvent) -> Result<(), CoreError> {
        let (decision_str, reason) = match event.decision {
            AuditDecision::Allowed => ("allowed", None),
            AuditDecision::Denied(r) => ("denied", Some(r)),
        };

        sqlx::query(
            "INSERT INTO audit_events (trace_id, skill, tool_name, decision, reason) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(event.trace_id)
        .bind(event.skill)
        .bind(event.tool_name)
        .bind(decision_str)
        .bind(reason)
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Memory(format!("Failed to append audit event: {e}")))?;

        Ok(())
    }

    async fn load_events(&self, trace_id: &str) -> Result<Vec<AuditEvent>, CoreError> {
        let rows = sqlx::query_as::<_, AuditEventRow>(
            "SELECT trace_id, skill, tool_name, decision, reason FROM audit_events WHERE trace_id = ? ORDER BY id ASC",
        )
        .bind(trace_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Memory(format!("Failed to load events: {e}")))?;

        Ok(rows.into_iter().map(|r| AuditEvent {
            trace_id: r.trace_id,
            skill: r.skill,
            tool_name: r.tool_name,
            decision: match r.decision.as_str() {
                "denied" => AuditDecision::Denied(r.reason.unwrap_or_default()),
                _ => AuditDecision::Allowed,
            },
        }).collect())
    }
}

#[derive(sqlx::FromRow)]
struct MessageRow {
    role: String,
    content: String,
}

#[derive(sqlx::FromRow)]
struct TraceRow {
    trace_id: String,
}

#[derive(sqlx::FromRow)]
struct AuditEventRow {
    trace_id: String,
    skill: Option<String>,
    tool_name: String,
    decision: String,
    reason: Option<String>,
}
