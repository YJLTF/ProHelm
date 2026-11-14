//! S3 集成测试：风险登记册、干系人登记册、项目章程与方法论裁剪。

use helm_core::HelmError;
use helm_core::db::Db;
use helm_core::models::{
    CharterContent, CreateProjectInput, CreateRiskInput, CreateStakeholderInput, ProjectMethod,
    ResponseStrategy, RiskCategory, RiskStatus, UpdateRiskInput, UpdateStakeholderInput,
};
use helm_core::repo::{charters, projects, risks, stakeholders};

fn db() -> Db {
    Db::open_in_memory().unwrap()
}

fn project(db: &Db, method: ProjectMethod) -> String {
    projects::create(
        &db.conn,
        &CreateProjectInput {
            name: format!("工件测试-{method:?}"),
            code: None,
            method,
            description: None,
            start_date: None,
            target_end_date: None,
            bac: 0.0,
        },
    )
    .unwrap()
    .id
}

fn risk_input(project_id: &str, title: &str, probability: i64, impact: i64) -> CreateRiskInput {
    CreateRiskInput {
        project_id: project_id.into(),
        title: title.into(),
        detail: None,
        category: RiskCategory::Technical,
        probability,
        impact,
        response_strategy: ResponseStrategy::Mitigate,
        response_plan: None,
        owner: None,
        trigger_desc: None,
    }
}

fn stakeholder_input(
    project_id: &str,
    name: &str,
    influence: i64,
    interest: i64,
) -> CreateStakeholderInput {
    CreateStakeholderInput {
        project_id: project_id.into(),
        name: name.into(),
        role: None,
        expectation: None,
        influence,
        interest,
        notes: None,
    }
}

#[test]
fn risk_crud_scoring_and_ordering() {
    let db = db();
    let p = project(&db, ProjectMethod::Predictive);
    let low = risks::create(&db.conn, &risk_input(&p, "低风险项", 2, 2)).unwrap();
    assert_eq!(low.risk_no, 1);
    assert_eq!(low.score(), 4);

    let high = risks::create(&db.conn, &risk_input(&p, "高风险项", 4, 5)).unwrap();
    assert_eq!(high.risk_no, 2);
    assert_eq!(high.score(), 20);

    // 按评分降序
    let list = risks::list_by_project(&db.conn, &p).unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].id, high.id);
    assert_eq!(list[1].id, low.id);

    // 状态流转 + 部分更新
    let updated = risks::update(
        &db.conn,
        &UpdateRiskInput {
            id: high.id.clone(),
            title: None,
            detail: None,
            category: None,
            probability: None,
            impact: None,
            response_strategy: None,
            response_plan: None,
            owner: Some("王姐".into()),
            trigger_desc: None,
            status: Some(RiskStatus::Occurred),
        },
    )
    .unwrap();
    assert_eq!(updated.status, RiskStatus::Occurred);
    assert_eq!(updated.owner.as_deref(), Some("王姐"));

    risks::delete(&db.conn, &low.id).unwrap();
    assert!(matches!(
        risks::get(&db.conn, &low.id),
        Err(HelmError::NotFound(_))
    ));
}

#[test]
fn risk_validation_rejects_out_of_range_and_empty_title() {
    let db = db();
    let p = project(&db, ProjectMethod::Predictive);
    assert!(matches!(
        risks::create(&db.conn, &risk_input(&p, "越上界", 6, 3)),
        Err(HelmError::Validation(_))
    ));
    assert!(matches!(
        risks::create(&db.conn, &risk_input(&p, "越下界", 0, 3)),
        Err(HelmError::Validation(_))
    ));
    let mut empty_title = risk_input(&p, "占位", 3, 3);
    empty_title.title = "   ".into();
    assert!(matches!(
        risks::create(&db.conn, &empty_title),
        Err(HelmError::Validation(_))
    ));
}

#[test]
fn stakeholder_quadrant_strategies() {
    let db = db();
    let p = project(&db, ProjectMethod::Agile);
    let hh = stakeholders::create(&db.conn, &stakeholder_input(&p, "甲方负责人", 5, 5)).unwrap();
    assert_eq!(hh.strategy_key(), "manage_closely");
    let hl = stakeholders::create(&db.conn, &stakeholder_input(&p, "财务总监", 4, 2)).unwrap();
    assert_eq!(hl.strategy_key(), "keep_satisfied");
    let lh = stakeholders::create(&db.conn, &stakeholder_input(&p, "终端用户代表", 2, 5)).unwrap();
    assert_eq!(lh.strategy_key(), "keep_informed");
    let ll = stakeholders::create(&db.conn, &stakeholder_input(&p, "行政前台", 1, 1)).unwrap();
    assert_eq!(ll.strategy_key(), "monitor");
}

#[test]
fn stakeholder_validation_and_update() {
    let db = db();
    let p = project(&db, ProjectMethod::Agile);
    assert!(matches!(
        stakeholders::create(&db.conn, &stakeholder_input(&p, "  ", 3, 3)),
        Err(HelmError::Validation(_))
    ));
    assert!(matches!(
        stakeholders::create(&db.conn, &stakeholder_input(&p, "越界", 6, 3)),
        Err(HelmError::Validation(_))
    ));

    let s = stakeholders::create(&db.conn, &stakeholder_input(&p, "甲方负责人", 5, 5)).unwrap();
    let updated = stakeholders::update(
        &db.conn,
        &UpdateStakeholderInput {
            id: s.id.clone(),
            name: Some("甲方项目总监".into()),
            role: Some("发起人".into()),
            expectation: None,
            influence: Some(4),
            interest: None,
            notes: None,
        },
    )
    .unwrap();
    assert_eq!(updated.name, "甲方项目总监");
    assert_eq!(updated.role.as_deref(), Some("发起人"));
    assert_eq!(updated.influence, 4);
    // 4 权力 + 5 利益 → 高权力高利益
    assert_eq!(updated.strategy_key(), "manage_closely");
}

#[test]
fn charter_lazy_init_and_roundtrip() {
    let db = db();
    let p = project(&db, ProjectMethod::Hybrid);
    let first = charters::get_or_create(&db.conn, &p).unwrap();
    assert!(first.created);
    assert_eq!(first.charter.objective, "");

    let content = CharterContent {
        objective: "上线新版 App".into(),
        success_criteria: "崩溃率 < 0.1%".into(),
        ..Default::default()
    };
    let saved = charters::save(&db.conn, &p, &content).unwrap();
    assert_eq!(saved.charter.objective, "上线新版 App");
    assert!(!saved.created);

    let again = charters::get_or_create(&db.conn, &p).unwrap();
    assert!(!again.created);
    assert_eq!(again.charter, content);
    assert_eq!(again.id, saved.id);

    assert!(matches!(
        charters::get_or_create(&db.conn, "ghost-project"),
        Err(HelmError::NotFound(_))
    ));
}

#[test]
fn tailoring_preset_set_and_apply() {
    let db = db();
    let p = project(&db, ProjectMethod::Predictive);
    let created = projects::get(&db.conn, &p).unwrap();
    let tailored = created.tailored.clone().unwrap();
    assert!(tailored.contains(&"wbs".to_string()));
    assert!(!tailored.contains(&"kanban".to_string()));

    let customized =
        projects::set_tailored(&db.conn, &p, &["charter".to_string(), "kanban".to_string()])
            .unwrap();
    assert_eq!(
        customized.tailored,
        Some(vec!["charter".to_string(), "kanban".to_string()])
    );

    assert!(matches!(
        projects::set_tailored(&db.conn, &p, &["bogus".to_string()]),
        Err(HelmError::Validation(_))
    ));

    let reset = projects::apply_recommended_tailoring(&db.conn, &p).unwrap();
    let keys = reset.tailored.unwrap();
    assert!(keys.contains(&"evm".to_string()));
    assert!(!keys.contains(&"kanban".to_string()));
}

#[test]
fn artifacts_cascade_with_project() {
    let db = db();
    let p = project(&db, ProjectMethod::Hybrid);
    risks::create(&db.conn, &risk_input(&p, "风险甲", 3, 3)).unwrap();
    stakeholders::create(&db.conn, &stakeholder_input(&p, "干系人甲", 3, 3)).unwrap();
    charters::save(&db.conn, &p, &CharterContent::default()).unwrap();

    projects::delete(&db.conn, &p).unwrap();
    assert_eq!(risks::list_by_project(&db.conn, &p).unwrap().len(), 0);
    assert_eq!(
        stakeholders::list_by_project(&db.conn, &p).unwrap().len(),
        0
    );
    let n: i64 = db
        .conn
        .query_row("SELECT COUNT(*) FROM charters", [], |r| r.get(0))
        .unwrap();
    assert_eq!(n, 0);
}
