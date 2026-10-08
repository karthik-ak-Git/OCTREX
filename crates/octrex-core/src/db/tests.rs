#[cfg(test)]
mod db_integration_tests {
    use crate::db::config::DbConfig;
    use crate::db::lifecycle::DatabaseState;
    use crate::db::manager::DatabaseManager;
    use crate::db::models::*;
    use crate::db::repository::*;
    use crate::ids::{ArtifactId, RequestId};
    use crate::session::{Session, SessionStatus};
    use crate::task::{Task, TaskStatus};
    use crate::workspace::Workspace;
    use std::path::PathBuf;

    fn setup_temp_db() -> DatabaseManager {
        let temp_dir = std::env::temp_dir();
        let db_file = temp_dir.join(format!("octrex_test_{}.db", uuid::Uuid::new_v4().simple()));
        let config = DbConfig::custom(db_file);
        let db = DatabaseManager::new(config);
        db.initialize()
            .expect("Database initialization must succeed");
        db
    }

    #[test]
    fn test_db_lifecycle_and_migrations() {
        let db = setup_temp_db();
        let status = db.get_status();
        assert_eq!(status.state, DatabaseState::DatabaseReady);
        assert_eq!(
            status.schema_version,
            crate::db::migrations::MIGRATIONS.len() as u32
        );
        assert_eq!(status.migration_status, "COMPLETED");
        assert!(status.last_error.is_none());

        // Test running migrations again (idempotency check)
        db.with_conn_mut(|conn| {
            let version = crate::db::migrations::run_migrations(conn)
                .expect("Re-running migrations must succeed");
            assert_eq!(version, crate::db::migrations::MIGRATIONS.len() as u32);
            Ok(())
        })
        .unwrap();

        // Graceful shutdown
        db.shutdown();
        let shutdown_status = db.get_status();
        assert_eq!(shutdown_status.state, DatabaseState::DatabaseClosed);
    }

    #[test]
    fn test_workspace_repository_crud() {
        let db = setup_temp_db();
        let repo = SqliteWorkspaceRepository::new(db.clone());

        let ws = Workspace::new("Test Project", PathBuf::from("d:\\test_project"));
        let created = repo.create_workspace(&ws).unwrap();
        assert_eq!(created.name, "Test Project");

        let fetched = repo.get_workspace(&ws.id).unwrap();
        assert!(fetched.is_some());
        let fetched_ws = fetched.unwrap();
        assert_eq!(fetched_ws.name, "Test Project");

        let mut to_update = fetched_ws;
        to_update.name = "Renamed Project".to_string();
        let updated = repo.update_workspace(&to_update).unwrap();
        assert_eq!(updated.name, "Renamed Project");

        let list = repo.list_workspaces().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "Renamed Project");

        let deleted = repo.delete_workspace(&ws.id).unwrap();
        assert!(deleted);
        assert!(repo.get_workspace(&ws.id).unwrap().is_none());
    }

    #[test]
    fn test_session_repository_crud() {
        let db = setup_temp_db();
        let ws_repo = SqliteWorkspaceRepository::new(db.clone());
        let sess_repo = SqliteSessionRepository::new(db.clone());

        let ws = ws_repo
            .create_workspace(&Workspace::new("WS", PathBuf::from("/ws")))
            .unwrap();
        let sess = Session::new("Chat Session 1", Some(ws.id.clone()));

        let created = sess_repo.create_session(&sess).unwrap();
        assert_eq!(created.title, "Chat Session 1");
        assert_eq!(created.status, SessionStatus::Active);

        let fetched = sess_repo.get_session(&sess.id).unwrap().unwrap();
        assert_eq!(fetched.title, "Chat Session 1");

        let by_ws = sess_repo.list_sessions_by_workspace(&ws.id).unwrap();
        assert_eq!(by_ws.len(), 1);

        let mut to_update = fetched;
        to_update.status = SessionStatus::Closed;
        let updated = sess_repo.update_session(&to_update).unwrap();
        assert_eq!(updated.status, SessionStatus::Closed);

        let deleted = sess_repo.delete_session(&sess.id).unwrap();
        assert!(deleted);
    }

    #[test]
    fn test_message_repository_crud() {
        let db = setup_temp_db();
        let sess_repo = SqliteSessionRepository::new(db.clone());
        let msg_repo = SqliteMessageRepository::new(db.clone());

        let sess = sess_repo
            .create_session(&Session::new("Session", None))
            .unwrap();

        let msg1 = Message {
            id: format!("msg-{}", uuid::Uuid::new_v4().simple()),
            session_id: sess.id.clone(),
            role: MessageRole::User,
            content: "Hello AI Assistant!".to_string(),
            metadata: Some(serde_json::json!({ "client": "web" })),
            created_at: 1000,
        };

        let msg2 = Message {
            id: format!("msg-{}", uuid::Uuid::new_v4().simple()),
            session_id: sess.id.clone(),
            role: MessageRole::Assistant,
            content: "Hello! How can I assist you with Octrex today?".to_string(),
            metadata: None,
            created_at: 1001,
        };

        msg_repo.create_message(&msg1).unwrap();
        msg_repo.create_message(&msg2).unwrap();

        let msgs = msg_repo.list_messages_by_session(&sess.id).unwrap();
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].role, MessageRole::User);
        assert_eq!(msgs[1].role, MessageRole::Assistant);

        let fetched_msg1 = msg_repo.get_message(&msg1.id).unwrap().unwrap();
        assert_eq!(fetched_msg1.content, "Hello AI Assistant!");
    }

    #[test]
    fn test_task_and_task_step_repository() {
        let db = setup_temp_db();
        let task_repo = SqliteTaskRepository::new(db.clone());
        let step_repo = SqliteTaskStepRepository::new(db.clone());

        let req_id = RequestId::new();
        let task = Task::new("Refactor Database Layer", Some(req_id), None, None);
        let created_task = task_repo.create_task(&task).unwrap();
        assert_eq!(created_task.status, TaskStatus::Created);

        let step1 = TaskStep {
            id: format!("step-{}", uuid::Uuid::new_v4().simple()),
            task_id: task.id.clone(),
            sequence: 1,
            objective: "Inspect existing code".to_string(),
            status: TaskStepStatus::Completed,
            created_at: 100,
            started_at: Some(101),
            completed_at: Some(110),
            error: None,
        };
        step_repo.create_step(&step1).unwrap();

        let steps = step_repo.list_steps_by_task(&task.id).unwrap();
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].objective, "Inspect existing code");

        let active = task_repo.list_active_tasks().unwrap();
        assert_eq!(active.len(), 1);

        let mut updated_task = task;
        updated_task.status = TaskStatus::Completed;
        task_repo.update_task(&updated_task).unwrap();

        let active_after = task_repo.list_active_tasks().unwrap();
        assert_eq!(active_after.len(), 0);
    }

    #[test]
    fn test_settings_repository() {
        let db = setup_temp_db();
        let settings_repo = SqliteSettingsRepository::new(db);

        settings_repo
            .set_setting("GENERAL", "theme", "dark")
            .unwrap();
        settings_repo
            .set_setting("PRIVACY", "mode", "strict_local")
            .unwrap();

        let theme = settings_repo
            .get_setting("GENERAL", "theme")
            .unwrap()
            .unwrap();
        assert_eq!(theme.value, "dark");

        let general_settings = settings_repo.list_settings_by_category("GENERAL").unwrap();
        assert_eq!(general_settings.len(), 1);

        settings_repo
            .set_setting("GENERAL", "theme", "light")
            .unwrap();
        let updated_theme = settings_repo
            .get_setting("GENERAL", "theme")
            .unwrap()
            .unwrap();
        assert_eq!(updated_theme.value, "light");
    }

    #[test]
    fn test_permissions_repository() {
        let db = setup_temp_db();
        let repo = SqlitePermissionRepository::new(db);

        let perm = PermissionRecord {
            id: format!("perm-{}", uuid::Uuid::new_v4().simple()),
            scope: "FILESYSTEM_READ".to_string(),
            resource: "d:\\OCTREX".to_string(),
            decision: PermissionDecision::Allow,
            duration: PermissionDuration::Persistent,
            workspace_id: None,
            created_at: 100,
            updated_at: 100,
        };

        repo.create_permission(&perm).unwrap();

        let fetched = repo.get_permission(&perm.id).unwrap().unwrap();
        assert_eq!(fetched.decision, PermissionDecision::Allow);
        assert_eq!(fetched.duration, PermissionDuration::Persistent);
    }

    #[test]
    fn test_artifacts_events_and_audit_repositories() {
        let db = setup_temp_db();
        let art_repo = SqliteArtifactRepository::new(db.clone());
        let evt_repo = SqliteEventRepository::new(db.clone());
        let audit_repo = SqliteAuditRepository::new(db);

        let art = ArtifactRecord {
            id: ArtifactId::new(),
            task_id: None,
            workspace_id: None,
            name: "report.pdf".to_string(),
            path: "/tmp/report.pdf".to_string(),
            artifact_type: "PDF".to_string(),
            size: 2048,
            checksum: Some("sha256:abc123".to_string()),
            created_at: 1000,
            verification_status: "VERIFIED".to_string(),
        };
        art_repo.create_artifact(&art).unwrap();
        assert!(art_repo.get_artifact(&art.id).unwrap().is_some());

        let evt = EventRecord {
            id: format!("evt-{}", uuid::Uuid::new_v4().simple()),
            event_type: "TASK_COMPLETED".to_string(),
            payload: serde_json::json!({ "task_id": "task-1" }),
            timestamp: 2000,
            session_id: None,
            task_id: None,
            workspace_id: None,
        };
        evt_repo.record_event(&evt).unwrap();
        let evts = evt_repo.list_events(10).unwrap();
        assert_eq!(evts.len(), 1);

        let audit = AuditRecord {
            id: format!("audit-{}", uuid::Uuid::new_v4().simple()),
            timestamp: 3000,
            event_type: "PERMISSION_REQUEST".to_string(),
            task_id: None,
            session_id: None,
            workspace_id: None,
            actor: "user".to_string(),
            provider: None,
            model: None,
            route: Some("LOCAL".to_string()),
            privacy_classification: Some("PUBLIC".to_string()),
            policy_source: Some("USER_SETTING".to_string()),
            permission: Some("ALLOW".to_string()),
            tool: None,
            success: true,
            reason: None,
        };
        audit_repo.record_audit(&audit).unwrap();
        let audits = audit_repo.list_audits(10).unwrap();
        assert_eq!(audits.len(), 1);
        assert_eq!(audits[0].actor, "user");
    }
}
