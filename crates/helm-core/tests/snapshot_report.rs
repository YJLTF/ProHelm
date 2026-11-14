//! S5 集成测试：快照导出/导入、里程碑标记、报告/CSV、雷达与负荷。

use helm_core::db::Db;
use helm_core::models::{
    CharterContent, CreateProjectInput, CreateRiskInput, CreateStakeholderInput, CreateTaskInput,
    ProjectMethod, UpdateTaskInput,
};
use helm_core::repo::{charters, projects, risks, schedule, stakeholders, stats, tasks};
use helm_core::report;
use helm_core::snapshot;

fn db() -> Db {
    Db::open_in_memory().unwrap()
}

fn project(db: &Db, name: &str) -> String {
    projects::create(
        &db.conn,
        &CreateProjectInput {
            name: name.into(),
            code: None,
            method: ProjectMethod::Hybrid,
            description: Some(format!("{name} 的项目描述")),
            start_date: Some("2027-04-01".into()),
            target_end_date: None,
            bac: 50_000.0,
        },
    )
    .unwrap()
    .id
}

fn task(db: &Db, project_id: &str, title: &str, duration: f64, assignee: Option<&str>) -> String {
    tasks::create(
        &db.conn,
        &CreateTaskInput {
            project_id: project_id.into(),
            parent_id: None,
            title: title.into(),
            notes: None,
            priority: Default::default(),
            estimate_hours: 4.0,
            duration_days: duration,
            start_date: None,
            end_date: None,
            assignee: assignee.map(String::from),
            is_milestone: false,
        },
    )
    .unwrap()
    .id
}

fn seed(db: &Db) -> (String, String, String) {
    let p1 = project(db, "项目甲");
    let p2 = project(db, "项目乙");
    let t1 = task(db, &p1, "需求调研", 3.0, Some("小陈"));
    let t2 = task(db, &p1, "开发实现", 5.0, Some("小陈"));
    task(db, &p2, "视觉设计", 4.0, Some("小美"));
    risks::create(
        &db.conn,
        &CreateRiskInput {
            project_id: p1.clone(),
            title: "高风险：需求蔓延".into(),
            detail: None,
            category: Default::default(),
            probability: 4,
            impact: 5,
            response_strategy: Default::default(),
            response_plan: None,
            owner: Some("王姐".into()),
            trigger_desc: None,
        },
    )
    .unwrap();
    stakeholders::create(
        &db.conn,
        &CreateStakeholderInput {
            project_id: p1.clone(),
            name: "甲方负责人".into(),
            role: None,
            expectation: None,
            influence: 5,
            interest: 5,
            notes: None,
        },
    )
    .unwrap();
    charters::save(
        &db.conn,
        &p1,
        &CharterContent {
            objective: "交付甲方的核心系统".into(),
            ..Default::default()
        },
    )
    .unwrap();
    (p1, t1, t2)
}

#[test]
fn snapshot_export_import_roundtrip() {
    let db1 = db();
    let (p1, _t1, _t2) = seed(&db1);

    let snap = snapshot::export(&db1.conn).unwrap();
    assert_eq!(snap.app, "prohelm");
    assert_eq!(snap.version, snapshot::SNAPSHOT_VERSION);
    assert_eq!(snap.projects.len(), 2);
    assert_eq!(snap.tasks.len(), 3);
    assert_eq!(snap.risks.len(), 1);
    assert_eq!(snap.stakeholders.len(), 1);
    assert_eq!(snap.charters.len(), 1);

    // 序列化 → 反序列化（模拟落盘再读回）
    let json = serde_json::to_string(&snap).unwrap();
    let parsed: helm_core::Snapshot = serde_json::from_str(&json).unwrap();

    let db2 = db();
    let stats = snapshot::import(&db2.conn, &parsed).unwrap();
    assert_eq!(stats.projects, 2);
    assert_eq!(stats.tasks, 3);
    assert_eq!(stats.risks, 1);

    // 原始 ID 保留，数据一致
    let restored = projects::get(&db2.conn, &p1).unwrap();
    assert_eq!(restored.name, "项目甲");
    assert_eq!(restored.tailored.as_ref().unwrap().len() > 0, true);
    let restored_tasks = tasks::list_by_project(&db2.conn, &p1).unwrap();
    assert_eq!(restored_tasks.len(), 2);
    let restored_risks = risks::list_by_project(&db2.conn, &p1).unwrap();
    assert_eq!(restored_risks[0].title, "高风险：需求蔓延");
    let charter = charters::get_or_create(&db2.conn, &p1).unwrap();
    assert_eq!(charter.charter.objective, "交付甲方的核心系统");

    // 导入是幂等覆盖：再导一遍再导一遍
    let snap2 = snapshot::export(&db2.conn).unwrap();
    let stats2 = snapshot::import(&db2.conn, &snap2).unwrap();
    assert_eq!(stats2.tasks, 3);
    assert_eq!(tasks::list_by_project(&db2.conn, &p1).unwrap().len(), 2);
}

#[test]
fn import_rejects_invalid_snapshot() {
    let db = db();
    let mut bad = snapshot::export(&db.conn).unwrap();
    bad.app = "other-app".into();
    assert!(matches!(
        snapshot::import(&db.conn, &bad),
        Err(helm_core::HelmError::Validation(_))
    ));

    bad.version = 99;
    bad.app = "prohelm".into();
    assert!(matches!(
        snapshot::import(&db.conn, &bad),
        Err(helm_core::HelmError::Validation(_))
    ));
}

#[test]
fn milestone_flag_roundtrip() {
    let db = db();
    let p = project(&db, "里程碑项目");
    let mut input = CreateTaskInput {
        project_id: p.clone(),
        parent_id: None,
        title: "需求评审通过".into(),
        notes: None,
        priority: Default::default(),
        estimate_hours: 0.0,
        duration_days: 0.0,
        start_date: None,
        end_date: None,
        assignee: None,
        is_milestone: true,
    };
    let t = tasks::create(&db.conn, &input).unwrap();
    assert!(t.is_milestone);

    input.is_milestone = false;
    input.title = "改名".into();
    let updated = tasks::update(
        &db.conn,
        &UpdateTaskInput {
            id: t.id.clone(),
            title: None,
            notes: None,
            status: None,
            priority: None,
            estimate_hours: None,
            duration_days: None,
            actual_hours: None,
            start_date: None,
            end_date: None,
            assignee: None,
            sort_order: None,
            is_milestone: Some(false),
        },
    )
    .unwrap();
    assert!(!updated.is_milestone);
    // input 仍可正常用于 create（占位使用以避免未使用告警）
    input.project_id = format!("{p}-unused");
    let _ = input;
}

#[test]
fn risk_radar_and_workload() {
    let db = db();
    seed(&db);
    let radar = stats::risk_radar(&db.conn, 5).unwrap();
    assert_eq!(radar.len(), 1);
    assert_eq!(radar[0].score(), 20);
    assert_eq!(radar[0].project_name, "项目甲");

    let workload = stats::workload(&db.conn, 8).unwrap();
    assert_eq!(workload.len(), 2);
    let xiaochen = workload.iter().find(|w| w.assignee == "小陈").unwrap();
    assert_eq!(xiaochen.task_count, 2);
    assert_eq!(xiaochen.project_count, 1);
    assert_eq!(xiaochen.estimate_hours, 8.0);
}

#[test]
fn report_and_csv_generation() {
    let db = db();
    let p = project(&db, "报告项目");
    task(&db, &p, "设计,含逗号", 2.0, Some("小陈"));
    let ms = CreateTaskInput {
        project_id: p.clone(),
        parent_id: None,
        title: "评审通过".into(),
        notes: None,
        priority: Default::default(),
        estimate_hours: 0.0,
        duration_days: 0.0,
        start_date: None,
        end_date: None,
        assignee: None,
        is_milestone: true,
    };
    tasks::create(&db.conn, &ms).unwrap();
    risks::create(
        &db.conn,
        &CreateRiskInput {
            project_id: p.clone(),
            title: "测试风险「带引号」".into(),
            detail: None,
            category: Default::default(),
            probability: 3,
            impact: 3,
            response_strategy: Default::default(),
            response_plan: None,
            owner: None,
            trigger_desc: None,
        },
    )
    .unwrap();

    let md = report::project_report_markdown(&db.conn, &p).unwrap();
    assert!(md.contains("# 项目报告：报告项目"));
    assert!(md.contains("## 三、风险登记册"));
    assert!(md.contains("测试风险「带引号」"));
    assert!(md.contains("◆ 评审通过"));

    let csv = report::tasks_csv(&db.conn, &p).unwrap();
    assert!(csv.starts_with('\u{feff}'));
    assert!(csv.contains("编号,标题,状态"));
    // 任务标题含逗号 → 该字段应被引号包裹转义
    assert!(csv.contains('"'), "含逗号字段应被转义包裹");
    // CPM 排程可用（0 工期里程碑不破坏拓扑）
    let s = schedule::get(&db.conn, &p).unwrap();
    assert!(s.duration_days >= 2.0);
}
