use async_trait::async_trait;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use sunclaw_core::{AuditDecision, AuditEvent, AuditStore, CoreError, MemoryStore, Message, Role, Mission, MissionStatus, MissionStore, Artifact, ArtifactStore};

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

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS traces (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                trace_id TEXT NOT NULL,
                event_type TEXT NOT NULL,
                content TEXT NOT NULL,
                metadata TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Memory(format!("Failed to create traces table: {e}")))?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS missions (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                status TEXT NOT NULL,
                assign_to TEXT, -- AgentRole serialized
                sub_tasks TEXT,
                parent_id TEXT,
                trace_id TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Memory(format!("Failed to create missions table: {e}")))?;
        
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS artifacts (
                id TEXT PRIMARY KEY,
                trace_id TEXT NOT NULL,
                artifact_type TEXT NOT NULL,
                title TEXT NOT NULL,
                data TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Memory(format!("Failed to create artifacts table: {e}")))?;

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

#[async_trait]
impl sunclaw_core::TraceStore for SqliteStore {
    async fn append_trace(&self, event: sunclaw_core::TraceEvent) -> Result<(), CoreError> {
        let metadata_str = event.metadata.map(|m| m.to_string());
        
        sqlx::query(
            "INSERT INTO traces (trace_id, event_type, content, metadata) VALUES (?, ?, ?, ?)",
        )
        .bind(event.trace_id)
        .bind(event.event_type)
        .bind(event.content)
        .bind(metadata_str)
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Memory(format!("Failed to append trace: {e}")))?;

        Ok(())
    }

    async fn load_traces(&self, trace_id: &str) -> Result<Vec<sunclaw_core::TraceEvent>, CoreError> {
        let rows = sqlx::query_as::<_, TraceEventRow>(
            "SELECT trace_id, event_type, content, metadata FROM traces WHERE trace_id = ? ORDER BY id ASC",
        )
        .bind(trace_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Memory(format!("Failed to load traces: {e}")))?;

        Ok(rows.into_iter().map(|r| sunclaw_core::TraceEvent {
            trace_id: r.trace_id,
            event_type: r.event_type,
            content: r.content,
            metadata: r.metadata.and_then(|m| serde_json::from_str(&m).ok()),
        }).collect())
    }
}

#[async_trait]
impl MissionStore for SqliteStore {
    async fn create_mission(&self, mission: Mission) -> Result<(), CoreError> {
        let sub_tasks = serde_json::to_string(&mission.sub_tasks).unwrap_or_else(|_| "[]".to_string());
        let status = serde_json::to_string(&mission.status).unwrap_or_else(|_| "\"Pending\"".to_string());
        let assign_to = mission.assign_to.as_ref().map(|r| serde_json::to_string(r).unwrap_or_default());
        
        sqlx::query(
            "INSERT INTO missions (id, title, description, status, assign_to, sub_tasks, parent_id, trace_id) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(mission.id)
        .bind(mission.title)
        .bind(mission.description)
        .bind(status)
        .bind(assign_to)
        .bind(sub_tasks)
        .bind(mission.parent_id)
        .bind(mission.trace_id)
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Memory(format!("Failed to create mission: {e}")))?;
        Ok(())
    }

    async fn update_mission_status(&self, id: &str, status: MissionStatus) -> Result<(), CoreError> {
        let status_str = serde_json::to_string(&status).unwrap_or_else(|_| "\"Pending\"".to_string());
        sqlx::query("UPDATE missions SET status = ? WHERE id = ?")
            .bind(status_str)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| CoreError::Memory(format!("Failed to update mission: {e}")))?;
        Ok(())
    }

    async fn get_mission(&self, id: &str) -> Result<Mission, CoreError> {
        let row = sqlx::query_as::<_, MissionRow>(
            "SELECT id, title, description, status, assign_to, sub_tasks, parent_id, trace_id FROM missions WHERE id = ?"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Memory(format!("Failed to get mission: {e}")))?;

        Ok(Mission {
            id: row.id,
            title: row.title,
            description: row.description,
            status: serde_json::from_str(&row.status).unwrap_or(MissionStatus::Pending),
            assign_to: row.assign_to.and_then(|r| serde_json::from_str(&r).ok()),
            sub_tasks: serde_json::from_str(&row.sub_tasks).unwrap_or_default(),
            parent_id: row.parent_id,
            trace_id: row.trace_id,
        })
    }

    async fn list_missions(&self) -> Result<Vec<Mission>, CoreError> {
        let rows = sqlx::query_as::<_, MissionRow>(
            "SELECT id, title, description, status, assign_to, sub_tasks, parent_id, trace_id FROM missions ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Memory(format!("Failed to list missions: {e}")))?;

        Ok(rows.into_iter().map(|row| Mission {
            id: row.id,
            title: row.title,
            description: row.description,
            status: serde_json::from_str(&row.status).unwrap_or(MissionStatus::Pending),
            assign_to: row.assign_to.and_then(|r| serde_json::from_str(&r).ok()),
            sub_tasks: serde_json::from_str(&row.sub_tasks).unwrap_or_default(),
            parent_id: row.parent_id,
            trace_id: row.trace_id,
        }).collect())
    }
}

#[async_trait]
impl ArtifactStore for SqliteStore {
    async fn create_artifact(&self, artifact: Artifact) -> Result<(), CoreError> {
        sqlx::query(
            "INSERT INTO artifacts (id, trace_id, artifact_type, title, data) 
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(artifact.id)
        .bind(artifact.trace_id)
        .bind(artifact.artifact_type)
        .bind(artifact.title)
        .bind(artifact.data)
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Memory(format!("Failed to create artifact: {e}")))?;
        Ok(())
    }

    async fn get_artifact(&self, id: &str) -> Result<Artifact, CoreError> {
        let row = sqlx::query_as::<_, ArtifactRow>(
            "SELECT id, trace_id, artifact_type, title, data FROM artifacts WHERE id = ?"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Memory(format!("Failed to get artifact: {e}")))?;

        Ok(Artifact {
            id: row.id,
            trace_id: row.trace_id,
            artifact_type: row.artifact_type,
            title: row.title,
            data: row.data,
        })
    }

    async fn list_artifacts_by_trace(&self, trace_id: &str) -> Result<Vec<Artifact>, CoreError> {
        let rows = sqlx::query_as::<_, ArtifactRow>(
            "SELECT id, trace_id, artifact_type, title, data FROM artifacts WHERE trace_id = ? ORDER BY created_at DESC"
        )
        .bind(trace_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Memory(format!("Failed to list artifacts: {e}")))?;

        Ok(rows.into_iter().map(|row| Artifact {
            id: row.id,
            trace_id: row.trace_id,
            artifact_type: row.artifact_type,
            title: row.title,
            data: row.data,
        }).collect())
    }
}

#[derive(sqlx::FromRow)]
struct ArtifactRow {
    id: String,
    trace_id: String,
    artifact_type: String,
    title: String,
    data: String,
}

#[derive(sqlx::FromRow)]
struct MissionRow {
    id: String,
    title: String,
    description: String,
    status: String,
    assign_to: Option<String>,
    sub_tasks: String,
    parent_id: Option<String>,
    trace_id: Option<String>,
}

#[derive(sqlx::FromRow)]
struct TraceEventRow {
    trace_id: String,
    event_type: String,
    content: String,
    metadata: Option<String>,
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
