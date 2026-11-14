//! helm-core 集成测试：真实迁移 + 业务流（见 docs/04-开发计划.md 测试策略）。

use helm_core::HelmError;
use helm_core::db::Db;
use helm_core::models::{
    CreateProjectInput, CreateTaskInput, ProjectMethod, ProjectStatus, TaskStatus,
    UpdateProjectInput, UpdateTaskInput,
};
use helm_core::repo::{projects, stats, tasks};

fn db() -> Db {
    Db::open_in_memory().unwrap()
}

fn sample_project() -> CreateProjectInput {
    CreateProjectInput {
        name: "示例交付项目".into(),
        code: Some("PH-001".into()),
        method: ProjectMethod::Predictive,
        description: Some("用于测试的示例项目".into()),
        start_date: Some("2026-12-01".into()),
        target_end_date: Some("2027-02-01".into()),
        bac: 100_000.0,
    }
}

fn sample_task(project_id: &str, title: &str) -> CreateTaskInput {
    CreateTaskInput {
        project_id: project_id.into(),
        parent_id: None,
        title: title.into(),
        notes: None,
        priority: Default::default(),
        estimate_hours: 8.0,
        is_milestone: false,
        duration_days: 1.0,
        start_date: None,
        end_date: None,
        assignee: None,
    }
}

#[test]
fn project_crud_roundtrip() {
    let db = db();
    let p = projects::create(&db.conn, &sample_project()).unwrap();
    assert_eq!(p.status, ProjectStatus::Planning);
    assert_eq!(p.method, ProjectMethod::Predictive);
    assert_eq!(p.bac, 100_000.0);

    let updated = projects::update(
        &db.conn,
        &UpdateProjectInput {
            id: p.id.clone(),
            name: Some("改名后的项目".into()),
            code: None,
            method: None,
            status: Some(ProjectStatus::Active),
            description: None,
            start_date: None,
            target_end_date: None,
            bac: None,
        },
    )
    .unwrap();
    assert_eq!(updated.name, "改名后的项目");
    assert_eq!(updated.status, ProjectStatus::Active);
    // 未提供的字段保留旧值
    assert_eq!(updated.code.as_deref(), Some("PH-001"));
    assert_eq!(updated.method, ProjectMethod::Predictive);

    assert_eq!(projects::list(&db.conn).unwrap().len(), 1);

    projects::delete(&db.conn, &p.id).unwrap();
    assert!(projects::list(&db.conn).unwrap().is_empty());
    assert!(matches!(
        projects::get(&db.conn, &p.id),
        Err(HelmError::NotFound(_))
    ));
}

#[test]
fn project_validation_rejects_empty_name() {
    let db = db();
    let mut input = sample_project();
    input.name = "   ".into();
    assert!(matches!(
        projects::create(&db.conn, &input),
        Err(HelmError::Validation(_))
    ));
}

#[test]
fn task_create_assigns_auto_wbs_codes() {
    let db = db();
    let p = projects::create(&db.conn, &sample_project()).unwrap();

    let t1 = tasks::create(&db.conn, &sample_task(&p.id, "需求调研")).unwrap();
    let t2 = tasks::create(&db.conn, &sample_task(&p.id, "方案设计")).unwrap();
    assert_eq!(t1.wbs_code.as_deref(), Some("1"));
    assert_eq!(t2.wbs_code.as_deref(), Some("2"));
    // 创建任务后项目自动激活
    assert_eq!(
        tasks::get(&db.conn, &t1.id).unwrap().status,
        TaskStatus::Todo
    );
    assert_eq!(
        projects::get(&db.conn, &p.id).unwrap().status,
        ProjectStatus::Active
    );

    // 子任务编码 = 父编码.序号
    let child = CreateTaskInput {
        parent_id: Some(t1.id.clone()),
        ..sample_task(&p.id, "访谈提纲")
    };
    let child = tasks::create(&db.conn, &child).unwrap();
    assert_eq!(child.wbs_code.as_deref(), Some("1.1"));

    let child2 = CreateTaskInput {
        parent_id: Some(t1.id.clone()),
        ..sample_task(&p.id, "原型确认")
    };
    let child2 = tasks::create(&db.conn, &child2).unwrap();
    assert_eq!(child2.wbs_code.as_deref(), Some("1.2"));
}

#[test]
fn task_update_and_stats() {
    let db = db();
    let p = projects::create(&db.conn, &sample_project()).unwrap();
    let t = tasks::create(&db.conn, &sample_task(&p.id, "编码实现")).unwrap();

    let updated = tasks::update(
        &db.conn,
        &UpdateTaskInput {
            id: t.id.clone(),
            title: None,
            notes: None,
            status: Some(TaskStatus::Done),
            priority: None,
            estimate_hours: None,
            duration_days: None,
            is_milestone: None,
            actual_hours: Some(10.5),
            start_date: None,
            end_date: None,
            assignee: Some("小陈".into()),
            sort_order: None,
        },
    )
    .unwrap();
    assert_eq!(updated.status, TaskStatus::Done);
    assert_eq!(updated.actual_hours, 10.5);
    assert_eq!(updated.assignee.as_deref(), Some("小陈"));
    assert_eq!(updated.title, "编码实现");

    let s = stats::get(&db.conn).unwrap();
    assert_eq!(s.project_count, 1);
    assert_eq!(s.active_projects, 1);
    assert_eq!(s.tasks_done, 1);
    assert_eq!(s.tasks_todo, 0);
}

#[test]
fn task_rejects_missing_project_and_cross_project_parent() {
    let db = db();
    let p1 = projects::create(&db.conn, &sample_project()).unwrap();
    let p2 = projects::create(
        &db.conn,
        &CreateProjectInput {
            name: "另一个项目".into(),
            ..sample_project()
        },
    )
    .unwrap();

    let ghost = sample_task("no-such-project", "幽灵任务");
    assert!(matches!(
        tasks::create(&db.conn, &ghost),
        Err(HelmError::NotFound(_))
    ));

    let parent = tasks::create(&db.conn, &sample_task(&p1.id, "父任务")).unwrap();
    let bad_child = CreateTaskInput {
        parent_id: Some(parent.id.clone()),
        ..sample_task(&p2.id, "跨项目子任务")
    };
    assert!(matches!(
        tasks::create(&db.conn, &bad_child),
        Err(HelmError::Validation(_))
    ));
}

#[test]
fn delete_project_cascades_tasks_and_records_oplog() {
    let db = db();
    let p = projects::create(&db.conn, &sample_project()).unwrap();
    let t = tasks::create(&db.conn, &sample_task(&p.id, "任务甲")).unwrap();
    tasks::update(
        &db.conn,
        &UpdateTaskInput {
            id: t.id.clone(),
            title: None,
            notes: None,
            status: Some(TaskStatus::InProgress),
            priority: None,
            estimate_hours: None,
            duration_days: None,
            is_milestone: None,
            actual_hours: None,
            start_date: None,
            end_date: None,
            assignee: None,
            sort_order: None,
        },
    )
    .unwrap();

    projects::delete(&db.conn, &p.id).unwrap();
    assert!(tasks::list_by_project(&db.conn, &p.id).unwrap().is_empty());

    // oplog 记录了 create/update/delete
    let n: i64 = db
        .conn
        .query_row("SELECT COUNT(*) FROM oplog", [], |r| r.get(0))
        .unwrap();
    assert!(n >= 4, "oplog 至少应记录 4 条写操作，实际 {n}");
}
