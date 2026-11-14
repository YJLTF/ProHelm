//! S4 集成测试：AI 溯源记录与设置存取。

use helm_core::db::Db;
use helm_core::models::{AiArtifactKind, AiArtifactStatus};
use helm_core::repo::{ai_artifacts, projects, settings};

fn db() -> Db {
    Db::open_in_memory().unwrap()
}

#[test]
fn settings_roundtrip() {
    let db = db();
    assert_eq!(settings::get(&db.conn, "ai.base_url").unwrap(), None);
    settings::set(&db.conn, "ai.base_url", "https://api.example.com/v1").unwrap();
    assert_eq!(
        settings::get(&db.conn, "ai.base_url").unwrap().as_deref(),
        Some("https://api.example.com/v1")
    );
    settings::set(&db.conn, "ai.base_url", "https://changed.example.com/v1").unwrap();
    assert_eq!(
        settings::get(&db.conn, "ai.base_url").unwrap().as_deref(),
        Some("https://changed.example.com/v1")
    );
}

#[test]
fn artifact_create_list_filter_and_status() {
    let db = db();
    let p = projects::create(
        &db.conn,
        &helm_core::models::CreateProjectInput {
            name: "溯源测试项目".into(),
            code: None,
            method: Default::default(),
            description: None,
            start_date: None,
            target_end_date: None,
            bac: 0.0,
        },
    )
    .unwrap()
    .id;

    let coach = ai_artifacts::create(
        &db.conn,
        AiArtifactKind::Coach,
        None,
        "openai-compatible",
        "glm-4-flash",
        "什么是关键路径？",
        "关键路径是……",
    )
    .unwrap();
    assert_eq!(coach.status, AiArtifactStatus::Draft);
    assert!(coach.project_id.is_none());

    let charter = ai_artifacts::create(
        &db.conn,
        AiArtifactKind::CharterDraft,
        Some(&p),
        "openai-compatible",
        "deepseek-chat",
        "生成章程",
        "{\"objective\":\"x\"}",
    )
    .unwrap();

    // 全量 2 条；按项目过滤 1 条
    assert_eq!(ai_artifacts::list(&db.conn, None).unwrap().len(), 2);
    let for_project = ai_artifacts::list(&db.conn, Some(&p)).unwrap();
    assert_eq!(for_project.len(), 1);
    assert_eq!(for_project[0].id, charter.id);

    // input_summary 截断到 200 字符
    let long = ai_artifacts::create(
        &db.conn,
        AiArtifactKind::WbsDraft,
        Some(&p),
        "openai-compatible",
        "glm-4-flash",
        &"长".repeat(500),
        "[]",
    )
    .unwrap();
    assert_eq!(long.input_summary.chars().count(), 200);

    // 状态流转
    let adopted =
        ai_artifacts::set_status(&db.conn, &charter.id, AiArtifactStatus::Adopted).unwrap();
    assert_eq!(adopted.status, AiArtifactStatus::Adopted);
    let discarded =
        ai_artifacts::set_status(&db.conn, &long.id, AiArtifactStatus::Discarded).unwrap();
    assert_eq!(discarded.status, AiArtifactStatus::Discarded);
    assert!(matches!(
        ai_artifacts::set_status(&db.conn, "ghost", AiArtifactStatus::Adopted),
        Err(helm_core::HelmError::NotFound(_))
    ));
}
