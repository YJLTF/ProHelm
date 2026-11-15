//! 挣值管理（EVM）：基于基线快照与工时台账的绩效度量。
//!
//! 纯函数实现，输入全部由仓库层装配（`repo::evm`）：
//! - PV（计划价值）：基线叶子任务的预算工时按计划起止日均摊，累计成计划 S 曲线；
//! - EV（挣值）：已完成叶子任务的基线预算工时，按 `completed_at` 落位；
//! - AC（实际成本）：工时台账按日累计（人时）；
//! - 指标：SV/CV/SPI/CPI/EAC（CPI 法）/VAC/完成率。
//!
//! 价值单位统一为"人时"（工时），与任务预估工时同源；金额型 BAC 字段留作 M2+ 单价换算。

use serde::Serialize;

/// 基线中的任务预算（只参与计算叶子，父级汇总行按结构过滤，避免重复计价）。
#[derive(Debug, Clone, PartialEq)]
pub struct PlannedTask {
    pub id: String,
    pub parent_id: Option<String>,
    pub estimate_hours: f64,
    pub start: Option<String>,
    pub end: Option<String>,
}

/// 当前任务完成状态（EV 按基线预算计价，基线外新任务按当前预估计价）。
#[derive(Debug, Clone, PartialEq)]
pub struct CurrentTask {
    pub id: String,
    pub parent_id: Option<String>,
    pub estimate_hours: f64,
    pub done: bool,
    /// 转入完成的时间（ISO）；None = 旧数据，时序上落在统计截止日。
    pub completed_at: Option<String>,
}

/// 工时台账按日聚合后的一条记录。
#[derive(Debug, Clone, PartialEq)]
pub struct LedgerDay {
    /// YYYY-MM-DD
    pub date: String,
    pub hours: f64,
}

/// S 曲线上的一个数据点（均为累计值）。
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EvmPoint {
    pub date: String,
    pub pv: f64,
    pub ev: f64,
    pub ac: f64,
}

/// 截止 `as_of` 的绩效汇总。
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EvmSummary {
    /// 完工预算 = 基线叶子任务预算工时合计。
    pub bac: f64,
    pub pv: f64,
    pub ev: f64,
    pub ac: f64,
    /// 进度偏差 EV - PV；成本偏差 EV - AC。
    pub sv: f64,
    pub cv: f64,
    /// 进度/成本绩效指数；分母为 0 时取 1.0（无法度量视为持平）。
    pub spi: f64,
    pub cpi: f64,
    /// 完工估算：有实际成本数据时按 CPI 法 BAC/CPI，否则 AC + (BAC - EV)。
    pub eac: f64,
    /// 完工偏差 BAC - EAC（负值为超支预警）。
    pub vac: f64,
    /// 完成百分比 = EV / BAC。
    pub percent_complete: f64,
}

/// EVM 报表：`has_baseline = false` 时汇总全 0，前端引导先建基线。
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EvmReport {
    pub has_baseline: bool,
    pub summary: EvmSummary,
    pub series: Vec<EvmPoint>,
    /// 基线中未排期（缺起止日）的预算工时，不进 PV 曲线。
    pub unscheduled_hours: f64,
}

/// 曲线最长天数上限：防御性截断，避免脏数据生成超大序列。
const MAX_SERIES_DAYS: usize = 370;

fn parse_date(s: &str) -> Option<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

fn fmt_date(d: chrono::NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}

/// 收集任务集合中"被当作父级"的 id，用于剔除汇总行。
fn parent_ids<'a, I>(tasks: I) -> std::collections::HashSet<&'a str>
where
    I: IntoIterator<Item = (&'a String, &'a Option<String>)>,
{
    tasks
        .into_iter()
        .filter_map(|(_, parent)| parent.as_deref())
        .collect()
}

/// 逐日均摊：把预算工时均匀铺到 [start, end] 每个日历天（含首尾）。
/// 无日期、起止颠倒或零预算的任务不产出条目。
fn daily_allocations(task: &PlannedTask) -> Vec<(chrono::NaiveDate, f64)> {
    let (Some(start), Some(end)) = (
        task.start.as_deref().and_then(parse_date),
        task.end.as_deref().and_then(parse_date),
    ) else {
        return Vec::new();
    };
    if end < start || task.estimate_hours <= 0.0 {
        return Vec::new();
    }
    let days = (end - start).num_days() + 1;
    let per_day = task.estimate_hours / days as f64;
    (0..days)
        .map(|offset| (start + chrono::Duration::days(offset), per_day))
        .collect()
}

/// EVM 主计算。`as_of` 为统计截止日（YYYY-MM-DD）。
pub fn compute(
    planned: &[PlannedTask],
    current: &[CurrentTask],
    ledger: &[LedgerDay],
    as_of: &str,
) -> EvmReport {
    let as_of_date = parse_date(as_of);

    // ---- 叶子集合（planned / current 各自按结构过滤汇总行） ----
    let planned_parents = parent_ids(planned.iter().map(|t| (&t.id, &t.parent_id)));
    let current_parents = parent_ids(current.iter().map(|t| (&t.id, &t.parent_id)));

    // ---- BAC 与 PV 分布 ----
    let mut bac = 0.0;
    let mut unscheduled = 0.0;
    let mut pv_by_day: std::collections::BTreeMap<chrono::NaiveDate, f64> = Default::default();
    let mut planned_budget: std::collections::HashMap<&str, f64> = Default::default();
    for t in planned {
        if planned_parents.contains(t.id.as_str()) {
            continue; // 汇总行：预算由叶子任务承担，不重复计价
        }
        planned_budget.insert(t.id.as_str(), t.estimate_hours);
        bac += t.estimate_hours;
        let allocs = daily_allocations(t);
        if allocs.is_empty() {
            unscheduled += t.estimate_hours;
            continue;
        }
        for (day, hours) in allocs {
            *pv_by_day.entry(day).or_default() += hours;
        }
    }

    // ---- EV：按完成时间落位；计价优先取基线预算，基线外任务用当前预估 ----
    let mut ev_by_day: std::collections::BTreeMap<chrono::NaiveDate, f64> = Default::default();
    let mut ev_total = 0.0;
    for t in current {
        if !t.done || current_parents.contains(t.id.as_str()) {
            continue;
        }
        // 完成时间晚于统计截止日的按"尚未完成"处理，保持汇总与曲线一致
        if let (Some(done_day), Some(cutoff)) = (
            t.completed_at
                .as_deref()
                .and_then(|iso| parse_date(iso.get(..10).unwrap_or(iso))),
            as_of_date,
        ) && done_day > cutoff
        {
            continue;
        }
        let budget = planned_budget
            .get(t.id.as_str())
            .copied()
            .unwrap_or(t.estimate_hours);
        ev_total += budget;
        let day = t
            .completed_at
            .as_deref()
            .and_then(|iso| parse_date(iso.get(..10).unwrap_or(iso)))
            .or(as_of_date);
        if let Some(day) = day {
            *ev_by_day.entry(day).or_default() += budget;
        }
    }

    // ---- AC：台账按日累计 ----
    let mut ac_by_day: std::collections::BTreeMap<chrono::NaiveDate, f64> = Default::default();
    for e in ledger {
        if let Some(day) = parse_date(&e.date) {
            *ac_by_day.entry(day).or_default() += e.hours;
        }
    }

    // ---- 序列：仅有任一曲线数据时构建；终点 = max(计划终点, 截止日) ----
    let mut series = Vec::new();
    let has_any = !pv_by_day.is_empty() || !ev_by_day.is_empty() || !ac_by_day.is_empty();
    if has_any {
        let horizon_end = pv_by_day
            .keys()
            .next_back()
            .copied()
            .or(ev_by_day.keys().next_back().copied())
            .or(ac_by_day.keys().next_back().copied())
            .or(as_of_date)
            .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(1970, 1, 1).expect("固定日期"));
        let horizon_end = match as_of_date {
            Some(d) if d > horizon_end => d,
            _ => horizon_end,
        };
        let horizon_start = pv_by_day
            .keys()
            .next()
            .copied()
            .or_else(|| ev_by_day.keys().next().copied())
            .or_else(|| ac_by_day.keys().next().copied())
            .or(as_of_date)
            .expect("has_any 时必有起点");
        let days = (horizon_end - horizon_start).num_days().max(0) as usize + 1;
        let mut cum_pv = 0.0;
        let mut cum_ev = 0.0;
        let mut cum_ac = 0.0;
        for offset in 0..days.min(MAX_SERIES_DAYS) {
            let day = horizon_start + chrono::Duration::days(offset as i64);
            cum_pv += pv_by_day.get(&day).copied().unwrap_or(0.0);
            cum_ev += ev_by_day.get(&day).copied().unwrap_or(0.0);
            cum_ac += ac_by_day.get(&day).copied().unwrap_or(0.0);
            series.push(EvmPoint {
                date: fmt_date(day),
                pv: round2(cum_pv),
                ev: round2(cum_ev),
                ac: round2(cum_ac),
            });
        }
    }

    // ---- 截止日汇总（EV 含缺完成时间的已完成任务，时序版本已并入截止日） ----
    let pv_at = as_of_date
        .map(|d| pv_by_day.range(..=d).map(|(_, h)| h).sum::<f64>())
        .unwrap_or(0.0);
    let ac_at = as_of_date
        .map(|d| ac_by_day.range(..=d).map(|(_, h)| h).sum::<f64>())
        .unwrap_or(0.0);
    EvmReport {
        has_baseline: !planned.is_empty(),
        summary: summarize(bac, pv_at, ev_total, ac_at),
        series,
        unscheduled_hours: round2(unscheduled),
    }
}

/// 由四要素推导绩效指标（独立成函数供黄金用例直接验证）。
pub fn summarize(bac: f64, pv: f64, ev: f64, ac: f64) -> EvmSummary {
    let spi = if pv > 0.0 { ev / pv } else { 1.0 };
    let cpi = if ac > 0.0 { ev / ac } else { 1.0 };
    // CPI 法仅在"有实际成本且工作未全部完成"时适用；否则按剩余计划值折算
    let eac = if ac > 0.0 && bac > 0.0 && ev < bac {
        bac / (ev / ac)
    } else {
        ac + (bac - ev).max(0.0)
    };
    EvmSummary {
        bac: round2(bac),
        pv: round2(pv),
        ev: round2(ev),
        ac: round2(ac),
        sv: round2(ev - pv),
        cv: round2(ev - ac),
        spi: round2(spi),
        cpi: round2(cpi),
        eac: round2(eac),
        vac: round2(bac - eac),
        percent_complete: if bac > 0.0 {
            round2(ev / bac * 100.0)
        } else {
            0.0
        },
    }
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

// ---------- 黄金用例 ----------

#[cfg(test)]
mod tests {
    use super::*;

    fn planned(id: &str, hours: f64, start: &str, end: &str) -> PlannedTask {
        PlannedTask {
            id: id.into(),
            parent_id: None,
            estimate_hours: hours,
            start: Some(start.into()),
            end: Some(end.into()),
        }
    }

    fn planned_parent(id: &str, parent: &str, hours: f64, start: &str, end: &str) -> PlannedTask {
        PlannedTask {
            id: id.into(),
            parent_id: Some(parent.into()),
            estimate_hours: hours,
            start: Some(start.into()),
            end: Some(end.into()),
        }
    }

    fn current(id: &str, hours: f64, done: bool, completed: Option<&str>) -> CurrentTask {
        CurrentTask {
            id: id.into(),
            parent_id: None,
            estimate_hours: hours,
            done,
            completed_at: completed.map(Into::into),
        }
    }

    fn ledger(date: &str, hours: f64) -> LedgerDay {
        LedgerDay {
            date: date.into(),
            hours,
        }
    }

    #[test]
    fn pv_curve_even_allocation() {
        // T1 10h [03-01..03-02] 每天 5h；T2 30h [03-03..03-05] 每天 10h
        let planned = vec![
            planned("t1", 10.0, "2026-03-01", "2026-03-02"),
            planned("t2", 30.0, "2026-03-03", "2026-03-05"),
        ];
        let report = compute(&planned, &[], &[], "2026-03-05");
        assert_eq!(report.summary.bac, 40.0);
        assert_eq!(report.series.len(), 5);
        assert_eq!(report.series[0].pv, 5.0); // 03-01
        assert_eq!(report.series[1].pv, 10.0); // 03-02
        assert_eq!(report.series[2].pv, 20.0); // 03-03 +10
        assert_eq!(report.series[4].pv, 40.0); // 03-05 收尾
    }

    #[test]
    fn ev_ac_and_indices() {
        let planned = vec![planned("t1", 10.0, "2026-03-01", "2026-03-04")]; // 2.5h/天
        let current = vec![current("t1", 10.0, true, Some("2026-03-02T18:00:00Z"))];
        let ledger = vec![ledger("2026-03-01", 3.0), ledger("2026-03-02", 4.0)];
        let report = compute(&planned, &current, &ledger, "2026-03-02");
        // PV@02 = 5；EV@02 = 10（提前完成）；AC@02 = 7
        let s = &report.summary;
        assert_eq!(s.pv, 5.0);
        assert_eq!(s.ev, 10.0);
        assert_eq!(s.ac, 7.0);
        assert_eq!(s.spi, 2.0);
        assert!((s.cpi - 10.0 / 7.0).abs() < 0.01);
        assert!(s.sv > 0.0 && s.cv > 0.0, "提前且省钱应双正");
    }

    #[test]
    fn leaf_filter_excludes_parent_rows() {
        // 父任务 30h + 子任务各 10h：BAC 只算叶子（20h），父行不计价
        let planned = vec![
            planned("p", 30.0, "2026-03-01", "2026-03-01"),
            planned_parent("c1", "p", 10.0, "2026-03-01", "2026-03-01"),
            planned_parent("c2", "p", 10.0, "2026-03-01", "2026-03-01"),
        ];
        let report = compute(&planned, &[], &[], "2026-03-01");
        assert_eq!(report.summary.bac, 20.0);
        assert_eq!(report.series[0].pv, 20.0);
    }

    #[test]
    fn unscheduled_tasks_excluded_from_pv() {
        let planned = vec![
            planned("t1", 10.0, "2026-03-01", "2026-03-01"),
            PlannedTask {
                id: "t2".into(),
                parent_id: None,
                estimate_hours: 5.0,
                start: None,
                end: None,
            },
        ];
        let report = compute(&planned, &[], &[], "2026-03-01");
        assert_eq!(report.summary.bac, 15.0);
        assert_eq!(report.unscheduled_hours, 5.0);
        assert_eq!(report.series[0].pv, 10.0);
    }

    #[test]
    fn series_extends_beyond_as_of_for_future_pv() {
        // 计划排到 03-10，统计截止 03-02：PV 曲线继续铺满计划期
        let planned = vec![planned("t1", 10.0, "2026-03-01", "2026-03-10")];
        let current = vec![current("t1", 10.0, true, Some("2026-03-01T10:00:00Z"))];
        let ledger = vec![ledger("2026-03-01", 2.0)];
        let report = compute(&planned, &current, &ledger, "2026-03-02");
        let last = report.series.last().expect("有序列");
        assert_eq!(last.date, "2026-03-10");
        assert_eq!(last.pv, 10.0);
        assert_eq!(last.ev, 10.0);
        assert_eq!(last.ac, 2.0);
        // 截止日汇总：PV@02 = 2h，SPI > 1
        assert_eq!(report.summary.pv, 2.0);
        assert!(report.summary.spi > 1.0);
    }

    #[test]
    fn legacy_done_without_completed_at_counts_into_ev() {
        let planned = vec![planned("t1", 10.0, "2026-03-01", "2026-03-02")];
        let current = vec![current("t1", 10.0, true, None)];
        let report = compute(&planned, &current, &[], "2026-03-02");
        assert_eq!(report.summary.ev, 10.0);
        // 时序上落在截止日
        assert_eq!(report.series.last().expect("有序列").ev, 10.0);
    }

    #[test]
    fn done_after_as_of_excluded_from_ev() {
        let planned = vec![planned("t1", 10.0, "2026-03-01", "2026-03-02")];
        let current = vec![current("t1", 10.0, true, Some("2026-03-09T10:00:00Z"))];
        let report = compute(&planned, &current, &[], "2026-03-02");
        assert_eq!(report.summary.ev, 0.0);
        assert!(report.series.iter().all(|p| p.ev == 0.0));
    }

    #[test]
    fn zero_ac_falls_back_eac_formula() {
        // AC=0 时 CPI 无意义 → EAC = AC + (BAC - EV) = 0 + 80
        let s = summarize(100.0, 50.0, 20.0, 0.0);
        assert_eq!(s.cpi, 1.0);
        assert_eq!(s.eac, 80.0);
        assert_eq!(s.vac, 20.0);
    }

    #[test]
    fn over_budget_eac_signals_vac_negative() {
        // EV=30, AC=60 → CPI=0.5, EAC=200, VAC=-100（超支预警）
        let s = summarize(100.0, 50.0, 30.0, 60.0);
        assert_eq!(s.cpi, 0.5);
        assert_eq!(s.eac, 200.0);
        assert_eq!(s.vac, -100.0);
        assert_eq!(s.percent_complete, 30.0);
    }

    #[test]
    fn empty_baseline_yields_empty_report() {
        let report = compute(&[], &[], &[], "2026-03-01");
        assert!(!report.has_baseline);
        assert!(report.series.is_empty());
        assert_eq!(report.summary.bac, 0.0);
        assert_eq!(report.summary.percent_complete, 0.0);
    }

    #[test]
    fn ledger_only_project_still_builds_ac_curve() {
        // 没有基线但已有工时：AC 曲线可用（has_baseline=false，前端给引导文案）
        let ledger = vec![ledger("2026-03-01", 4.0), ledger("2026-03-02", 2.0)];
        let report = compute(&[], &[], &ledger, "2026-03-02");
        assert!(!report.has_baseline);
        assert_eq!(report.series.last().expect("有序列").ac, 6.0);
        assert_eq!(report.summary.ac, 6.0);
    }

    #[test]
    fn ev_uses_baseline_budget_not_current_estimate() {
        // 完成后把预估从 10h 改成 99h：EV 仍按基线预算 10h 计价
        let planned = vec![planned("t1", 10.0, "2026-03-01", "2026-03-01")];
        let current = vec![current("t1", 99.0, true, Some("2026-03-01T10:00:00Z"))];
        let report = compute(&planned, &current, &[], "2026-03-01");
        assert_eq!(report.summary.ev, 10.0);
    }

    #[test]
    fn task_added_after_baseline_priced_at_current_estimate() {
        // 基线外新增任务完成：按当前预估计价 EV（BAC 不变）
        let planned = vec![planned("t1", 10.0, "2026-03-01", "2026-03-01")];
        let current = vec![
            current("t1", 10.0, false, None),
            current("t-new", 8.0, true, Some("2026-03-02T10:00:00Z")),
        ];
        let report = compute(&planned, &current, &[], "2026-03-02");
        assert_eq!(report.summary.bac, 10.0);
        assert_eq!(report.summary.ev, 8.0);
        assert_eq!(report.summary.percent_complete, 80.0);
    }
}
