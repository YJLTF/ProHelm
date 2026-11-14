//! CPM 关键路径算法（纯函数层，不触数据库，便于黄金用例快照测试）。
//!
//! 时间以"天"为单位的相对偏移（day 0 = 项目锚点日）：
//! - 正向推导最早开始/结束（ES/EF），反向推导最晚开始/结束（LS/LF）；
//! - 总浮动时间 = LS - ES；浮动为零的任务构成关键路径；
//! - 依赖类型：FS（完成后开始）/ SS / FF / SF，支持滞后（lag）天数。
//!
//! 参考：PMBOK 进度管理·关键路径法（见 docs/01 第 2.2 章）。

use std::collections::{BTreeSet, HashMap};

use crate::error::{HelmError, Result};
use crate::models::DependencyType;

const EPS: f64 = 1e-9;

#[derive(Debug, Clone)]
pub struct CpmTask {
    pub id: String,
    pub duration_days: f64,
}

#[derive(Debug, Clone)]
pub struct CpmEdge {
    pub predecessor_id: String,
    pub successor_id: String,
    pub dep_type: DependencyType,
    pub lag_days: f64,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CpmTaskResult {
    pub es: f64,
    pub ef: f64,
    pub ls: f64,
    pub lf: f64,
    pub total_float: f64,
    pub critical: bool,
}

#[derive(Debug, Clone, Default)]
pub struct CpmResult {
    pub tasks: HashMap<String, CpmTaskResult>,
    pub project_duration_days: f64,
}

fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < EPS
}

/// 推导一个依赖对"后继最早开始"的约束值。
fn forward_constraint(edge: &CpmEdge, es_p: f64, ef_p: f64, dur_s: f64) -> f64 {
    match edge.dep_type {
        DependencyType::Fs => ef_p + edge.lag_days,
        DependencyType::Ss => es_p + edge.lag_days,
        // FF/SF 约束的是后继的"结束"，换算为最早开始需要减去后继工期
        DependencyType::Ff => ef_p + edge.lag_days - dur_s,
        DependencyType::Sf => es_p + edge.lag_days - dur_s,
    }
}

/// 推导一个依赖对"前驱最晚结束"的约束值。
fn backward_constraint(edge: &CpmEdge, ls_s: f64, lf_s: f64, dur_p: f64) -> f64 {
    match edge.dep_type {
        DependencyType::Fs => ls_s - edge.lag_days,
        DependencyType::Ss => ls_s + edge.lag_days - dur_p,
        DependencyType::Ff => lf_s - edge.lag_days,
        DependencyType::Sf => lf_s + edge.lag_days - dur_p,
    }
}

pub fn compute(tasks: &[CpmTask], edges: &[CpmEdge]) -> Result<CpmResult> {
    let n = tasks.len();
    if n == 0 {
        return Ok(CpmResult::default());
    }

    let mut index: HashMap<&str, usize> = HashMap::new();
    let mut duration: Vec<f64> = Vec::with_capacity(n);
    for (i, t) in tasks.iter().enumerate() {
        if t.duration_days < 0.0 || !t.duration_days.is_finite() {
            return Err(HelmError::Validation(format!(
                "任务 {} 的工期无效: {}",
                t.id, t.duration_days
            )));
        }
        index.insert(t.id.as_str(), i);
        duration.push(t.duration_days);
    }

    let mut incoming: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut outgoing: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (ei, e) in edges.iter().enumerate() {
        let p = index.get(e.predecessor_id.as_str()).ok_or_else(|| {
            HelmError::Validation(format!("依赖的前置任务不存在: {}", e.predecessor_id))
        })?;
        let s = index.get(e.successor_id.as_str()).ok_or_else(|| {
            HelmError::Validation(format!("依赖的后继任务不存在: {}", e.successor_id))
        })?;
        outgoing[*p].push(ei);
        incoming[*s].push(ei);
    }

    // ---- Kahn 拓扑排序（BTreeSet 保证确定性顺序）----
    let mut indegree: Vec<usize> = incoming.iter().map(Vec::len).collect();
    let mut ready: BTreeSet<usize> = (0..n).filter(|&i| indegree[i] == 0).collect();
    let mut topo: Vec<usize> = Vec::with_capacity(n);
    while let Some(&i) = ready.iter().next() {
        ready.remove(&i);
        for &ei in &outgoing[i] {
            let s = index[edges[ei].successor_id.as_str()];
            indegree[s] -= 1;
            if indegree[s] == 0 {
                ready.insert(s);
            }
        }
        topo.push(i);
    }
    if topo.len() < n {
        let stuck: Vec<String> = (0..n)
            .filter(|i| indegree[*i] > 0)
            .map(|i| tasks[i].id.clone())
            .collect();
        return Err(HelmError::Validation(format!(
            "依赖关系存在循环，涉及任务: {}",
            stuck.join("、")
        )));
    }

    // ---- 正向推导 ES/EF ----
    let mut es = vec![0.0f64; n];
    let mut ef = vec![0.0f64; n];
    for &i in &topo {
        let mut start = 0.0f64;
        for &ei in &incoming[i] {
            let e = &edges[ei];
            let p = index[e.predecessor_id.as_str()];
            start = start.max(forward_constraint(e, es[p], ef[p], duration[i]));
        }
        es[i] = start.max(0.0);
        ef[i] = es[i] + duration[i];
    }
    let project_duration = ef.iter().cloned().fold(0.0f64, f64::max).max(0.0);

    // ---- 反向推导 LS/LF ----
    let mut lf = vec![project_duration; n];
    let mut ls = vec![project_duration; n];
    for &i in topo.iter().rev() {
        // 后继在逆拓扑序中已处理完毕
        for &ei in &outgoing[i] {
            let e = &edges[ei];
            let s = index[e.successor_id.as_str()];
            lf[i] = lf[i].min(backward_constraint(e, ls[s], lf[s], duration[i]));
        }
        ls[i] = lf[i] - duration[i];
    }

    let mut result = CpmResult {
        tasks: HashMap::with_capacity(n),
        project_duration_days: project_duration,
    };
    for i in 0..n {
        let float = ls[i] - es[i];
        result.tasks.insert(
            tasks[i].id.clone(),
            CpmTaskResult {
                es: es[i],
                ef: ef[i],
                ls: ls[i],
                lf: lf[i],
                total_float: float,
                critical: approx_eq(float, 0.0),
            },
        );
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task(id: &str, dur: f64) -> CpmTask {
        CpmTask {
            id: id.to_string(),
            duration_days: dur,
        }
    }

    fn edge(p: &str, s: &str) -> CpmEdge {
        CpmEdge {
            predecessor_id: p.to_string(),
            successor_id: s.to_string(),
            dep_type: DependencyType::Fs,
            lag_days: 0.0,
        }
    }

    fn edge_typed(p: &str, s: &str, t: DependencyType, lag: f64) -> CpmEdge {
        CpmEdge {
            predecessor_id: p.to_string(),
            successor_id: s.to_string(),
            dep_type: t,
            lag_days: lag,
        }
    }

    fn assert_close(a: f64, b: f64) {
        assert!(approx_eq(a, b), "数值断言失败: 期望 {b}，实际 {a}");
    }

    /// 黄金用例 1：单链 A(3) → B(5) → C(2)，全部关键，总工期 10。
    #[test]
    fn chain_is_all_critical() {
        let tasks = vec![task("A", 3.0), task("B", 5.0), task("C", 2.0)];
        let edges = vec![edge("A", "B"), edge("B", "C")];
        let r = compute(&tasks, &edges).unwrap();

        assert_close(r.project_duration_days, 10.0);
        let a = &r.tasks["A"];
        assert_close(a.es, 0.0);
        assert_close(a.ef, 3.0);
        assert_close(a.total_float, 0.0);
        assert!(a.critical);
        let b = &r.tasks["B"];
        assert_close(b.es, 3.0);
        assert_close(b.ef, 8.0);
        assert!(b.critical);
        let c = &r.tasks["C"];
        assert_close(c.es, 8.0);
        assert_close(c.ef, 10.0);
        assert!(c.critical);
    }

    /// 黄金用例 2：并行路径。A(3)→{B(5), C(2)}→D(1)：
    /// 关键路径 A→B→D（9 天），C 有 3 天浮动。
    #[test]
    fn parallel_paths_produce_float() {
        let tasks = vec![
            task("A", 3.0),
            task("B", 5.0),
            task("C", 2.0),
            task("D", 1.0),
        ];
        let edges = vec![
            edge("A", "B"),
            edge("A", "C"),
            edge("B", "D"),
            edge("C", "D"),
        ];
        let r = compute(&tasks, &edges).unwrap();

        assert_close(r.project_duration_days, 9.0);
        assert!(r.tasks["A"].critical);
        assert!(r.tasks["B"].critical);
        assert!(r.tasks["D"].critical);
        let c = &r.tasks["C"];
        assert_close(c.es, 3.0);
        assert_close(c.ef, 5.0);
        assert_close(c.lf, 8.0);
        assert_close(c.total_float, 3.0);
        assert!(!c.critical);
    }

    /// 黄金用例 3：SS 依赖。B 可与 A 同时开始，浮动 3 天。
    #[test]
    fn ss_dependency_starts_together() {
        let tasks = vec![task("A", 5.0), task("B", 2.0)];
        let edges = vec![edge_typed("A", "B", DependencyType::Ss, 0.0)];
        let r = compute(&tasks, &edges).unwrap();

        assert_close(r.project_duration_days, 5.0);
        let b = &r.tasks["B"];
        assert_close(b.es, 0.0);
        assert_close(b.ef, 2.0);
        assert_close(b.total_float, 3.0);
        assert!(!b.critical);
    }

    /// 黄金用例 4：FF 依赖。B 的结束不得早于 A 的结束 → B 必须第 4 天开始。
    #[test]
    fn ff_dependency_pins_finish() {
        let tasks = vec![task("A", 5.0), task("B", 2.0)];
        let edges = vec![edge_typed("A", "B", DependencyType::Ff, 0.0)];
        let r = compute(&tasks, &edges).unwrap();

        assert_close(r.project_duration_days, 5.0);
        let b = &r.tasks["B"];
        assert_close(b.es, 3.0);
        assert_close(b.ef, 5.0);
        assert!(b.critical);
    }

    /// 黄金用例 5：FS + 滞后 2 天。
    #[test]
    fn fs_with_lag_pushes_successor() {
        let tasks = vec![task("A", 3.0), task("B", 4.0)];
        let edges = vec![edge_typed("A", "B", DependencyType::Fs, 2.0)];
        let r = compute(&tasks, &edges).unwrap();

        assert_close(r.project_duration_days, 9.0);
        let b = &r.tasks["B"];
        assert_close(b.es, 5.0);
        assert_close(b.ef, 9.0);
        assert!(b.critical);
    }

    #[test]
    fn cycle_is_rejected() {
        let tasks = vec![task("A", 1.0), task("B", 1.0), task("C", 1.0)];
        let edges = vec![edge("A", "B"), edge("B", "C"), edge("C", "A")];
        let err = compute(&tasks, &edges).unwrap_err();
        assert!(err.to_string().contains("循环"), "实际错误: {err}");
    }

    #[test]
    fn dangling_edge_is_rejected() {
        let tasks = vec![task("A", 1.0)];
        let edges = vec![edge("A", "GHOST")];
        assert!(compute(&tasks, &edges).is_err());
    }

    #[test]
    fn empty_project_has_zero_duration() {
        let r = compute(&[], &[]).unwrap();
        assert_close(r.project_duration_days, 0.0);
    }
}
