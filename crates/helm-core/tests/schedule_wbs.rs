//! S2 集成测试：依赖网络、排程（CPM 落库）、WBS 重排与基线。

use helm_core::HelmError;
use helm_core::db::Db;
use helm_core::models::{
    CreateDependencyInput, CreateProjectInput, CreateTaskInput, DependencyType, MoveTaskInput,
    ProjectMethod, ReorderTaskInput, TaskMoveDirection, UpdateDependencyInput,
};
use helm_core::repo::{baselines, dependencies, projects, schedule, tasks};

fn db() -> Db {
    Db::open_in_memory().unwrap()
}

fn project_with_anchor() -> CreateProjectInput {
    CreateProjectInput {
        name: "排程测试项目".into(),
        code: None,
        method: ProjectMethod::Predictive,
        description: None,
        start_date: Some("2027-03-01".into()),
        target_end_date: None,
        bac: 0.0,
    }
}

fn task(db: &Db, project_id: &str, title: &str, duration_days: f64) -> String {
    tasks::create(
        &db.conn,
        &CreateTaskInput {
            project_id: project_id.into(),
            parent_id: None,
            title: title.into(),
            notes: None,
            priority: Default::default(),
            estimate_hours: 0.0,
            is_milestone: false,
            duration_days,
            start_date: None,
            end_date: None,
            assignee: None,
        },
    )
    .unwrap()
    .id
}

fn dep(db: &Db, project_id: &str, pred: &str, succ: &str) -> String {
    dependencies::create(
        &db.conn,
        &CreateDependencyInput {
            project_id: project_id.into(),
            predecessor_id: pred.into(),
            successor_id: succ.into(),
            dep_type: DependencyType::Fs,
            lag_days: 0.0,
        },
    )
    .unwrap()
    .id
}

#[test]
fn dependency_crud_and_validations() {
    let db = db();
    let p = projects::create(&db.conn, &project_with_anchor()).unwrap();
    let a = task(&db, &p.id, "任务A", 3.0);
    let b = task(&db, &p.id, "任务B", 5.0);
    let c = task(&db, &p.id, "任务C", 2.0);

    let d1 = dep(&db, &p.id, &a, &b);
    assert_eq!(d1.is_empty(), false);

    // 重复依赖
    let dup = CreateDependencyInput {
        project_id: p.id.clone(),
        predecessor_id: a.clone(),
        successor_id: b.clone(),
        dep_type: DependencyType::Fs,
        lag_days: 0.0,
    };
    assert!(matches!(
        dependencies::create(&db.conn, &dup),
        Err(HelmError::Validation(_))
    ));

    // 自依赖
    let self_dep = CreateDependencyInput {
        project_id: p.id.clone(),
        predecessor_id: a.clone(),
        successor_id: a.clone(),
        dep_type: DependencyType::Fs,
        lag_days: 0.0,
    };
    assert!(matches!(
        dependencies::create(&db.conn, &self_dep),
        Err(HelmError::Validation(_))
    ));

    // 循环：b→c 与 c→a 加上已有 a→b 成环
    dep(&db, &p.id, &b, &c);
    let cycle = CreateDependencyInput {
        project_id: p.id.clone(),
        predecessor_id: c.clone(),
        successor_id: a.clone(),
        dep_type: DependencyType::Fs,
        lag_days: 0.0,
    };
    assert!(matches!(
        dependencies::create(&db.conn, &cycle),
        Err(HelmError::Validation(_))
    ));

    // 更新滞后
    let updated = dependencies::update(
        &db.conn,
        &UpdateDependencyInput {
            id: d1.clone(),
            dep_type: None,
            lag_days: Some(2.5),
        },
    )
    .unwrap();
    assert_eq!(updated.lag_days, 2.5);

    dependencies::delete(&db.conn, &d1).unwrap();
    assert!(matches!(
        dependencies::get(&db.conn, &d1),
        Err(HelmError::NotFound(_))
    ));
}

#[test]
fn schedule_matches_cpm_and_dates() {
    let db = db();
    let p = projects::create(&db.conn, &project_with_anchor()).unwrap();
    let design = task(&db, &p.id, "方案设计", 3.0);
    let dev = task(&db, &p.id, "开发", 5.0);
    let testing = task(&db, &p.id, "测试", 2.0);
    dep(&db, &p.id, &design, &dev);
    dep(&db, &p.id, &dev, &testing);

    let s = schedule::get(&db.conn, &p.id).unwrap();
    assert_eq!(s.anchor_date, "2027-03-01");
    assert_eq!(s.duration_days, 10.0);

    // 列表顺序 = sort_order 顺序：设计 → 开发 → 测试
    let t = &s.tasks;
    assert_eq!(t[0].task_id, design);
    assert_eq!(t[0].start_date, "2027-03-01"); // ES=0
    assert_eq!(t[0].end_date, "2027-03-03"); // EF=3 → 含当天
    assert!(t[0].critical);
    assert_eq!(t[1].start_date, "2027-03-04"); // ES=3
    assert_eq!(t[1].end_date, "2027-03-08");
    assert_eq!(t[2].start_date, "2027-03-09"); // ES=8
    assert_eq!(t[2].end_date, "2027-03-10");
    assert!(t[2].critical);
    assert_eq!(t[2].predecessor_ids, vec![dev.clone()]);
}

#[test]
fn float_task_not_critical_in_schedule() {
    let db = db();
    let p = projects::create(&db.conn, &project_with_anchor()).unwrap();
    let a = task(&db, &p.id, "A", 3.0);
    let b = task(&db, &p.id, "B", 5.0);
    let c = task(&db, &p.id, "C", 2.0);
    let d = task(&db, &p.id, "D", 1.0);
    dep(&db, &p.id, &a, &b);
    dep(&db, &p.id, &a, &c);
    dep(&db, &p.id, &b, &d);
    dep(&db, &p.id, &c, &d);

    let s = schedule::get(&db.conn, &p.id).unwrap();
    assert_eq!(s.duration_days, 9.0);
    let c_result = s.tasks.iter().find(|t| t.task_id == c).unwrap();
    assert!(!c_result.critical);
    assert_eq!(c_result.total_float_days, 3.0);
}

#[test]
fn wbs_move_and_renumber() {
    let db = db();
    let p = projects::create(&db.conn, &project_with_anchor()).unwrap();
    let a = task(&db, &p.id, "阶段一", 1.0);
    let b = task(&db, &p.id, "阶段二", 1.0);
    let child = tasks::create(
        &db.conn,
        &CreateTaskInput {
            project_id: p.id.clone(),
            parent_id: Some(a.clone()),
            title: "子任务".into(),
            notes: None,
            priority: Default::default(),
            estimate_hours: 0.0,
            duration_days: 1.0,
            is_milestone: false,
            start_date: None,
            end_date: None,
            assignee: None,
        },
    )
    .unwrap();
    assert_eq!(child.wbs_code.as_deref(), Some("1.1"));

    // 子任务移动到 b 之下 → 2.1
    let moved = tasks::move_task(
        &db.conn,
        &MoveTaskInput {
            task_id: child.id.clone(),
            new_parent_id: Some(b.clone()),
        },
    )
    .unwrap();
    assert_eq!(moved.wbs_code.as_deref(), Some("2.1"));

    // 阶段一移动到阶段二之下 → 顶层仅剩 b，全树重编号：b=1、子=1.1、a=1.2
    tasks::move_task(
        &db.conn,
        &MoveTaskInput {
            task_id: a.clone(),
            new_parent_id: Some(b.clone()),
        },
    )
    .unwrap();
    assert_eq!(
        tasks::get(&db.conn, &b).unwrap().wbs_code.as_deref(),
        Some("1")
    );
    assert_eq!(
        tasks::get(&db.conn, &a).unwrap().wbs_code.as_deref(),
        Some("1.2")
    );
    assert_eq!(
        tasks::get(&db.conn, &child.id).unwrap().wbs_code.as_deref(),
        Some("1.1")
    );

    // 阶段一升回顶层（追加为末位）；a/b 同 sort_order=1 时按创建时间排序，a 先创建排前
    // → a=1、b=2、子=2.1
    let outdented = tasks::move_task(
        &db.conn,
        &MoveTaskInput {
            task_id: a.clone(),
            new_parent_id: None,
        },
    )
    .unwrap();
    assert_eq!(outdented.wbs_code.as_deref(), Some("1"));
    assert_eq!(
        tasks::get(&db.conn, &b).unwrap().wbs_code.as_deref(),
        Some("2")
    );
    assert_eq!(
        tasks::get(&db.conn, &child.id).unwrap().wbs_code.as_deref(),
        Some("2.1")
    );
}

#[test]
fn move_rejects_cycle_and_cross_project() {
    let db = db();
    let p = projects::create(&db.conn, &project_with_anchor()).unwrap();
    let a = task(&db, &p.id, "A", 1.0);
    let c = tasks::create(
        &db.conn,
        &CreateTaskInput {
            project_id: p.id.clone(),
            parent_id: Some(a.clone()),
            title: "C".into(),
            notes: None,
            priority: Default::default(),
            estimate_hours: 0.0,
            duration_days: 1.0,
            is_milestone: false,
            start_date: None,
            end_date: None,
            assignee: None,
        },
    )
    .unwrap();

    // 移到自己的后代之下
    let into_descendant = MoveTaskInput {
        task_id: a.clone(),
        new_parent_id: Some(c.id.clone()),
    };
    assert!(matches!(
        tasks::move_task(&db.conn, &into_descendant),
        Err(HelmError::Validation(_))
    ));

    // 跨项目移动
    let p2 = projects::create(&db.conn, &project_with_anchor()).unwrap();
    let other = task(&db, &p2.id, "他项目任务", 1.0);
    let cross = MoveTaskInput {
        task_id: a.clone(),
        new_parent_id: Some(other.clone()),
    };
    assert!(matches!(
        tasks::move_task(&db.conn, &cross),
        Err(HelmError::Validation(_))
    ));
}

#[test]
fn reorder_up_swaps_siblings_and_renumbers() {
    let db = db();
    let p = projects::create(&db.conn, &project_with_anchor()).unwrap();
    let a = task(&db, &p.id, "甲", 1.0);
    let b = task(&db, &p.id, "乙", 1.0);
    let c = task(&db, &p.id, "丙", 1.0);

    let moved = tasks::reorder_task(
        &db.conn,
        &ReorderTaskInput {
            task_id: c.clone(),
            direction: TaskMoveDirection::Up,
        },
    )
    .unwrap();
    assert_eq!(moved.wbs_code.as_deref(), Some("2"));
    assert_eq!(
        tasks::get(&db.conn, &a).unwrap().wbs_code.as_deref(),
        Some("1")
    );
    assert_eq!(
        tasks::get(&db.conn, &b).unwrap().wbs_code.as_deref(),
        Some("3")
    );

    // 已在顶部再上移 → 原样返回
    let top = tasks::reorder_task(
        &db.conn,
        &ReorderTaskInput {
            task_id: a.clone(),
            direction: TaskMoveDirection::Up,
        },
    )
    .unwrap();
    assert_eq!(top.wbs_code.as_deref(), Some("1"));
}

#[test]
fn baseline_roundtrip() {
    let db = db();
    let p = projects::create(&db.conn, &project_with_anchor()).unwrap();
    let a = task(&db, &p.id, "A", 1.0);
    let b = task(&db, &p.id, "B", 2.0);
    dep(&db, &p.id, &a, &b);

    // 空名称拒绝
    assert!(matches!(
        baselines::create(&db.conn, &p.id, "   "),
        Err(HelmError::Validation(_))
    ));

    let bl = baselines::create(&db.conn, &p.id, "开工基线").unwrap();
    assert_eq!(bl.name, "开工基线");
    assert!(bl.snapshot.contains("capturedAt"));
    assert!(bl.snapshot.contains(&a));
    assert!(bl.snapshot.contains(&b));

    let list = baselines::list_by_project(&db.conn, &p.id).unwrap();
    assert_eq!(list.len(), 1);

    baselines::delete(&db.conn, &bl.id).unwrap();
    assert!(
        baselines::list_by_project(&db.conn, &p.id)
            .unwrap()
            .is_empty()
    );
}
