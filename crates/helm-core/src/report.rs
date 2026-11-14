//! 报告生成：项目 Markdown 报告与任务 CSV 导出（纯函数层）。

use rusqlite::Connection;

use crate::error::Result;
use crate::models::{
    ProjectMethod, ProjectStatus, ResponseStrategy, RiskCategory, RiskStatus, TaskPriority,
    TaskStatus,
};
use crate::repo::{baselines, charters, projects, risks, schedule, stakeholders, tasks};

fn zh_method(m: &ProjectMethod) -> &'static str {
    match m {
        ProjectMethod::Predictive => "预测型",
        ProjectMethod::Agile => "敏捷",
        ProjectMethod::Hybrid => "混合",
    }
}

fn zh_project_status(s: &ProjectStatus) -> &'static str {
    match s {
        ProjectStatus::Planning => "规划中",
        ProjectStatus::Active => "进行中",
        ProjectStatus::Paused => "已暂停",
        ProjectStatus::Closed => "已关闭",
    }
}

fn zh_category(c: &RiskCategory) -> &'static str {
    match c {
        RiskCategory::Technical => "技术",
        RiskCategory::External => "外部",
        RiskCategory::Organizational => "组织",
        RiskCategory::Resource => "资源",
        RiskCategory::Schedule => "进度",
        RiskCategory::Cost => "成本",
        RiskCategory::Quality => "质量",
        RiskCategory::Other => "其他",
    }
}

fn zh_strategy(s: &ResponseStrategy) -> &'static str {
    match s {
        ResponseStrategy::Avoid => "规避",
        ResponseStrategy::Transfer => "转移",
        ResponseStrategy::Mitigate => "减轻",
        ResponseStrategy::Accept => "接受",
        ResponseStrategy::Escalate => "上报",
    }
}

fn zh_risk_status(s: &RiskStatus) -> &'static str {
    match s {
        RiskStatus::Open => "打开",
        RiskStatus::Occurred => "已发生",
        RiskStatus::Closed => "已关闭",
    }
}

fn zh_task_status(s: &TaskStatus) -> &'static str {
    match s {
        TaskStatus::Todo => "待办",
        TaskStatus::InProgress => "进行中",
        TaskStatus::Done => "已完成",
    }
}

fn zh_priority(p: &TaskPriority) -> &'static str {
    match p {
        TaskPriority::Low => "低",
        TaskPriority::Medium => "中",
        TaskPriority::High => "高",
        TaskPriority::Urgent => "紧急",
    }
}

/// 项目 Markdown 报告：概览 / 章程 / 干系人 / 风险 / 任务进度 / 基线。
pub fn project_report_markdown(conn: &Connection, project_id: &str) -> Result<String> {
    let p = projects::get(conn, project_id)?;
    let task_list = tasks::list_by_project(conn, project_id)?;
    let risk_list = risks::list_by_project(conn, project_id)?;
    let stakeholder_list = stakeholders::list_by_project(conn, project_id)?;
    let charter = charters::get_or_create(conn, project_id)?;
    let baseline_list = baselines::list_by_project(conn, project_id)?;
    let duration_days = schedule::get(conn, project_id)
        .map(|s| s.duration_days)
        .unwrap_or(-1.0);

    let done = task_list
        .iter()
        .filter(|t| t.status == TaskStatus::Done)
        .count();
    let in_progress = task_list
        .iter()
        .filter(|t| t.status == TaskStatus::InProgress)
        .count();
    let today = chrono::Local::now()
        .date_naive()
        .format("%Y-%m-%d")
        .to_string();
    let overdue = task_list
        .iter()
        .filter(|t| {
            t.status != TaskStatus::Done
                && t.end_date.as_deref().is_some_and(|d| d < today.as_str())
        })
        .count();

    let mut md = String::new();
    md.push_str(&format!("# 项目报告：{}\n\n", p.name));
    md.push_str(&format!(
        "| 编号 | 开发方法 | 状态 | 计划周期 | 总预算 |\n|---|---|---|---|---|\n| {} | {} | {} | {} ~ {} | {} |\n",
        p.code.as_deref().unwrap_or("—"),
        zh_method(&p.method),
        zh_project_status(&p.status),
        p.start_date.as_deref().unwrap_or("—"),
        p.target_end_date.as_deref().unwrap_or("—"),
        if p.bac > 0.0 { format!("¥{:.0}", p.bac) } else { "未设置".to_string() },
    ));
    md.push_str(&format!(
        "\n> 报告生成时间：{} ｜ 任务 {} 个（完成 {done}、进行中 {in_progress}、逾期 {overdue}）｜ CPM 总工期 {}\n\n",
        today,
        task_list.len(),
        if duration_days >= 0.0 { format!("{duration_days} 天") } else { "无法计算（依赖异常）".to_string() },
    ));

    md.push_str("## 一、项目章程\n\n");
    for (label, value) in [
        ("项目目的与目标", &charter.charter.objective),
        ("成功标准", &charter.charter.success_criteria),
        ("主要交付物", &charter.charter.key_deliverables),
        ("关键里程碑", &charter.charter.milestones),
        ("制约因素", &charter.charter.constraints),
        ("假设条件", &charter.charter.assumptions),
        ("高层级风险", &charter.charter.high_level_risks),
        ("项目经理职权", &charter.charter.pm_authority),
    ] {
        if !value.trim().is_empty() {
            md.push_str(&format!("- **{label}**：{}\n", value.replace('\n', "；")));
        }
    }
    md.push('\n');

    if !stakeholder_list.is_empty() {
        md.push_str("## 二、干系人\n\n");
        md.push_str("| 姓名 | 角色 | 权力 | 利益 | 策略 |\n|---|---|---|---|---|\n");
        for s in &stakeholder_list {
            md.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                s.name,
                s.role.as_deref().unwrap_or("—"),
                s.influence,
                s.interest,
                strategy_label(s.strategy_key()),
            ));
        }
        md.push('\n');
    }

    if !risk_list.is_empty() {
        md.push_str("## 三、风险登记册\n\n");
        md.push_str("| 编号 | 风险 | 类别 | 概率×影响 | 策略 | 责任人 | 状态 |\n|---|---|---|---|---|---|---|\n");
        for r in &risk_list {
            md.push_str(&format!(
                "| R{} | {} | {} | {}×{} = {} | {} | {} | {} |\n",
                r.risk_no,
                r.title,
                zh_category(&r.category),
                r.probability,
                r.impact,
                r.score(),
                zh_strategy(&r.response_strategy),
                r.owner.as_deref().unwrap_or("—"),
                zh_risk_status(&r.status),
            ));
        }
        md.push('\n');
    }

    md.push_str("## 四、任务与进度\n\n");
    if task_list.is_empty() {
        md.push_str("（暂无任务）\n\n");
    } else {
        for t in &task_list {
            let depth = t
                .wbs_code
                .as_deref()
                .map(|c| c.split('.').count() - 1)
                .unwrap_or(0);
            let indent = "  ".repeat(depth);
            let flag = if t.is_milestone { "◆ " } else { "" };
            md.push_str(&format!(
                "{indent}- [{}] {}{}（{}d）｜{}｜{}｜{}｜优先级 {}\n",
                match t.status {
                    TaskStatus::Done => "x",
                    TaskStatus::InProgress => "~",
                    TaskStatus::Todo => " ",
                },
                flag,
                t.title,
                t.duration_days,
                zh_task_status(&t.status),
                t.assignee.as_deref().unwrap_or("未分配"),
                t.end_date.as_deref().unwrap_or("无截止"),
                zh_priority(&t.priority),
            ));
        }
        md.push('\n');
    }

    if !baseline_list.is_empty() {
        md.push_str("## 五、计划基线\n\n");
        for b in &baseline_list {
            md.push_str(&format!(
                "- {}（{}）\n",
                b.name,
                b.created_at
                    .get(..16)
                    .unwrap_or(&b.created_at)
                    .replace('T', " ")
            ));
        }
        md.push('\n');
    }

    Ok(md)
}

fn strategy_label(key: &str) -> &'static str {
    match key {
        "manage_closely" => "重点管理",
        "keep_satisfied" => "令其满意",
        "keep_informed" => "随时告知",
        _ => "监控即可",
    }
}

/// 任务 CSV（带 BOM，Excel 友好）。
pub fn tasks_csv(conn: &Connection, project_id: &str) -> Result<String> {
    let p = projects::get(conn, project_id)?;
    let task_list = tasks::list_by_project(conn, project_id)?;
    let mut out = String::new();
    out.push('\u{feff}');
    out.push_str("编号,标题,状态,优先级,父任务编号,工期(天),预估工时(小时),实际工时(小时),负责人,开始日期,截止日期,里程碑\n");
    let _ = &zh_priority;
    let by_id: std::collections::HashMap<&str, &crate::models::Task> =
        task_list.iter().map(|t| (t.id.as_str(), t)).collect();
    for t in &task_list {
        let parent_code = t
            .parent_id
            .as_deref()
            .and_then(|pid| by_id.get(pid))
            .and_then(|parent| parent.wbs_code.clone())
            .unwrap_or_default();
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{}\n",
            csv_cell(t.wbs_code.as_deref().unwrap_or("")),
            csv_cell(&t.title),
            csv_cell(zh_task_status(&t.status)),
            csv_cell(zh_priority(&t.priority)),
            csv_cell(&parent_code),
            t.duration_days,
            t.estimate_hours,
            t.actual_hours,
            csv_cell(t.assignee.as_deref().unwrap_or("")),
            csv_cell(t.start_date.as_deref().unwrap_or("")),
            csv_cell(t.end_date.as_deref().unwrap_or("")),
            if t.is_milestone { "是" } else { "否" },
        ));
    }
    let _ = p; // 项目名用于前端默认文件名；内容中不重复
    Ok(out)
}

fn csv_cell(v: &str) -> String {
    if v.contains(',') || v.contains('"') || v.contains('\n') || v.contains('\r') {
        format!("\"{}\"", v.replace('"', "\"\""))
    } else {
        v.to_string()
    }
}
