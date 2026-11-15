use rusqlite::types::{ToSql, ToSqlOutput, ValueRef};
use serde::{Deserialize, Serialize};

use crate::error::{HelmError, Result};

/// 领域枚举通用能力：字符串序列化（前端 camelCase JSON / SQLite TEXT 列共用一套取值）。
/// 默认变体以 `default(变体 => 标签)` 单独声明，同时派生 `Default`。
macro_rules! db_enum {
    ($(#[$meta:meta])* $name:ident, default($dvar:ident => $dtag:literal), {
        $($variant:ident => $tag:literal),+ $(,)?
    }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum $name {
            #[default]
            $dvar,
            $($variant),+
        }

        impl $name {
            pub fn as_str(&self) -> &'static str {
                match self {
                    Self::$dvar => $dtag,
                    $(Self::$variant => $tag,)+
                }
            }
            pub fn parse(s: &str) -> Result<Self> {
                match s {
                    $dtag => Ok(Self::$dvar),
                    $($tag => Ok(Self::$variant),)+
                    other => Err(HelmError::Validation(format!(
                        "无效的 {} 取值: {}", stringify!($name), other
                    ))),
                }
            }
        }

        impl ToSql for $name {
            fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
                Ok(ToSqlOutput::Borrowed(ValueRef::Text(self.as_str().as_bytes())))
            }
        }
    };
}

db_enum!(
    /// 开发方法：预测型 / 敏捷 / 混合（PMBOK 8 支持三类，见 docs/01 第 2 章）。
    ProjectMethod, default(Hybrid => "hybrid"), {
        Predictive => "predictive",
        Agile => "agile",
    }
);

db_enum!(
    /// 项目状态。
    ProjectStatus, default(Planning => "planning"), {
        Active => "active",
        Paused => "paused",
        Closed => "closed",
    }
);

db_enum!(
    /// 任务状态（看板三列；S3 引入 Blocked）。
    TaskStatus, default(Todo => "todo"), {
        InProgress => "in_progress",
        Done => "done",
    }
);

db_enum!(
    /// 任务优先级。
    TaskPriority, default(Medium => "medium"), {
        Low => "low",
        High => "high",
        Urgent => "urgent",
    }
);

/// 项目。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub code: Option<String>,
    pub method: ProjectMethod,
    pub status: ProjectStatus,
    pub description: Option<String>,
    pub start_date: Option<String>,
    pub target_end_date: Option<String>,
    pub bac: f64,
    /// 方法论裁剪：启用的流程项 key 列表（创建时按开发方法预置，见 tailoring.rs）
    pub tailored: Option<Vec<String>>,
    pub created_at: String,
    pub updated_at: String,
}

/// 任务（任务 = WBS 叶子工作包的最小执行单元；S2 引入层级依赖）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub project_id: String,
    pub parent_id: Option<String>,
    pub wbs_code: Option<String>,
    pub title: String,
    pub notes: Option<String>,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub estimate_hours: f64,
    pub duration_days: f64,
    pub actual_hours: f64,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub assignee: Option<String>,
    pub sort_order: i64,
    /// 里程碑标记：零工期的重要节点（甘特图深色呈现）
    pub is_milestone: bool,
    /// 转入 done 的时刻（时间相位挣值 EV 的依据；旧数据/未完成为 None）。
    /// 基线快照中不含该字段，`default` 保证旧快照 JSON 可反序列化。
    #[serde(default)]
    pub completed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectInput {
    pub name: String,
    pub code: Option<String>,
    #[serde(default)]
    pub method: ProjectMethod,
    pub description: Option<String>,
    pub start_date: Option<String>,
    pub target_end_date: Option<String>,
    #[serde(default)]
    pub bac: f64,
}

impl CreateProjectInput {
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(HelmError::Validation("项目名称不能为空".into()));
        }
        Ok(())
    }
}

/// 部分更新：`None` 表示不修改该字段（SQL 层用 COALESCE 保留旧值）。
/// 注意：置空文本字段（如清除编号）需要 S2 引入显式 `Set<T>` 语义。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProjectInput {
    pub id: String,
    pub name: Option<String>,
    pub code: Option<String>,
    pub method: Option<ProjectMethod>,
    pub status: Option<ProjectStatus>,
    pub description: Option<String>,
    pub start_date: Option<String>,
    pub target_end_date: Option<String>,
    pub bac: Option<f64>,
}

impl UpdateProjectInput {
    pub fn validate(&self) -> Result<()> {
        if let Some(name) = &self.name
            && name.trim().is_empty()
        {
            return Err(HelmError::Validation("项目名称不能为空".into()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskInput {
    pub project_id: String,
    pub parent_id: Option<String>,
    pub title: String,
    pub notes: Option<String>,
    #[serde(default)]
    pub priority: TaskPriority,
    #[serde(default)]
    pub estimate_hours: f64,
    #[serde(default = "default_duration_days")]
    pub duration_days: f64,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub assignee: Option<String>,
    #[serde(default)]
    pub is_milestone: bool,
}

impl CreateTaskInput {
    pub fn validate(&self) -> Result<()> {
        if self.title.trim().is_empty() {
            return Err(HelmError::Validation("任务标题不能为空".into()));
        }
        if self.estimate_hours < 0.0 {
            return Err(HelmError::Validation("预估工时不能为负数".into()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTaskInput {
    pub id: String,
    pub title: Option<String>,
    pub notes: Option<String>,
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
    pub estimate_hours: Option<f64>,
    pub duration_days: Option<f64>,
    pub actual_hours: Option<f64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub assignee: Option<String>,
    pub sort_order: Option<i64>,
    pub is_milestone: Option<bool>,
}

impl UpdateTaskInput {
    pub fn validate(&self) -> Result<()> {
        if let Some(title) = &self.title
            && title.trim().is_empty()
        {
            return Err(HelmError::Validation("任务标题不能为空".into()));
        }
        if self.estimate_hours.is_some_and(|v| v < 0.0)
            || self.duration_days.is_some_and(|v| v < 0.0)
            || self.actual_hours.is_some_and(|v| v < 0.0)
        {
            return Err(HelmError::Validation("工时/工期不能为负数".into()));
        }
        Ok(())
    }
}

fn default_duration_days() -> f64 {
    1.0
}

// ---------- S2：进度（依赖与工期）、WBS 重排、基线 ----------

db_enum!(
    /// 依赖类型：FS 完成后开始 / SS 开始后开始 / FF 完成后完成 / SF 开始后完成。
    DependencyType, default(Fs => "fs"), {
        Ss => "ss",
        Ff => "ff",
        Sf => "sf",
    }
);

/// 任务间依赖（CPM 网络的边）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dependency {
    pub id: String,
    pub project_id: String,
    pub predecessor_id: String,
    pub successor_id: String,
    pub dep_type: DependencyType,
    pub lag_days: f64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDependencyInput {
    pub project_id: String,
    pub predecessor_id: String,
    pub successor_id: String,
    #[serde(default)]
    pub dep_type: DependencyType,
    #[serde(default)]
    pub lag_days: f64,
}

/// 部分更新：`None` 表示不修改该字段。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDependencyInput {
    pub id: String,
    pub dep_type: Option<DependencyType>,
    pub lag_days: Option<f64>,
}

/// WBS 重排：把任务移动到新的父级（`None` = 升到顶层，追加为末位）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveTaskInput {
    pub task_id: String,
    pub new_parent_id: Option<String>,
}

db_enum!(
    /// 同级排序方向。
    TaskMoveDirection, default(Up => "up"), {
        Down => "down",
    }
);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReorderTaskInput {
    pub task_id: String,
    #[serde(default)]
    pub direction: TaskMoveDirection,
}

/// 计划基线：创建时刻的任务/依赖结构快照（M2 挣值管理以此为对比基准）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Baseline {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub snapshot: String,
    pub created_at: String,
}

// ---------- S3：风险登记册 / 干系人登记册 / 项目章程 ----------

db_enum!(
    /// 风险类别（PMBOK 风险分解结构 RBS 常用分类）。
    RiskCategory, default(Technical => "technical"), {
        External => "external",
        Organizational => "organizational",
        Resource => "resource",
        Schedule => "schedule",
        Cost => "cost",
        Quality => "quality",
        Other => "other",
    }
);

db_enum!(
    /// 应对策略：规避 / 转移 / 减轻 / 接受 / 上报。
    ResponseStrategy, default(Mitigate => "mitigate"), {
        Avoid => "avoid",
        Transfer => "transfer",
        Accept => "accept",
        Escalate => "escalate",
    }
);

db_enum!(
    /// 风险状态：打开 / 已发生（转为问题） / 关闭。
    RiskStatus, default(Open => "open"), {
        Occurred => "occurred",
        Closed => "closed",
    }
);

/// 风险登记册条目。评分 = 概率 × 影响（1~25），由显示层计算着色。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Risk {
    pub id: String,
    pub project_id: String,
    pub risk_no: i64,
    pub title: String,
    pub detail: Option<String>,
    pub category: RiskCategory,
    pub probability: i64,
    pub impact: i64,
    pub response_strategy: ResponseStrategy,
    pub response_plan: Option<String>,
    pub owner: Option<String>,
    pub trigger_desc: Option<String>,
    pub status: RiskStatus,
    pub created_at: String,
    pub updated_at: String,
}

impl Risk {
    pub fn score(&self) -> i64 {
        self.probability * self.impact
    }
}

fn default_risk_level() -> i64 {
    3
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRiskInput {
    pub project_id: String,
    pub title: String,
    pub detail: Option<String>,
    #[serde(default)]
    pub category: RiskCategory,
    #[serde(default = "default_risk_level")]
    pub probability: i64,
    #[serde(default = "default_risk_level")]
    pub impact: i64,
    #[serde(default)]
    pub response_strategy: ResponseStrategy,
    pub response_plan: Option<String>,
    pub owner: Option<String>,
    pub trigger_desc: Option<String>,
}

impl CreateRiskInput {
    pub fn validate(&self) -> Result<()> {
        if self.title.trim().is_empty() {
            return Err(HelmError::Validation("风险标题不能为空".into()));
        }
        if !(1..=5).contains(&self.probability) || !(1..=5).contains(&self.impact) {
            return Err(HelmError::Validation("概率/影响取值须在 1~5 之间".into()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRiskInput {
    pub id: String,
    pub title: Option<String>,
    pub detail: Option<String>,
    pub category: Option<RiskCategory>,
    pub probability: Option<i64>,
    pub impact: Option<i64>,
    pub response_strategy: Option<ResponseStrategy>,
    pub response_plan: Option<String>,
    pub owner: Option<String>,
    pub trigger_desc: Option<String>,
    pub status: Option<RiskStatus>,
}

impl UpdateRiskInput {
    pub fn validate(&self) -> Result<()> {
        if let Some(title) = &self.title
            && title.trim().is_empty()
        {
            return Err(HelmError::Validation("风险标题不能为空".into()));
        }
        if self.probability.is_some_and(|v| !(1..=5).contains(&v))
            || self.impact.is_some_and(|v| !(1..=5).contains(&v))
        {
            return Err(HelmError::Validation("概率/影响取值须在 1~5 之间".into()));
        }
        Ok(())
    }
}

/// 干系人登记册条目。管理策略由权力/利益方格推导（见 `strategy_key`）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stakeholder {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub role: Option<String>,
    pub expectation: Option<String>,
    pub influence: i64,
    pub interest: i64,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Stakeholder {
    /// 权力/利益方格四象限：manage_closely / keep_satisfied / keep_informed / monitor。
    pub fn strategy_key(&self) -> &'static str {
        match (self.influence >= 4, self.interest >= 4) {
            (true, true) => "manage_closely",
            (true, false) => "keep_satisfied",
            (false, true) => "keep_informed",
            (false, false) => "monitor",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateStakeholderInput {
    pub project_id: String,
    pub name: String,
    pub role: Option<String>,
    pub expectation: Option<String>,
    #[serde(default = "default_risk_level")]
    pub influence: i64,
    #[serde(default = "default_risk_level")]
    pub interest: i64,
    pub notes: Option<String>,
}

impl CreateStakeholderInput {
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(HelmError::Validation("干系人姓名不能为空".into()));
        }
        if !(1..=5).contains(&self.influence) || !(1..=5).contains(&self.interest) {
            return Err(HelmError::Validation("权力/利益取值须在 1~5 之间".into()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStakeholderInput {
    pub id: String,
    pub name: Option<String>,
    pub role: Option<String>,
    pub expectation: Option<String>,
    pub influence: Option<i64>,
    pub interest: Option<i64>,
    pub notes: Option<String>,
}

impl UpdateStakeholderInput {
    pub fn validate(&self) -> Result<()> {
        if let Some(name) = &self.name
            && name.trim().is_empty()
        {
            return Err(HelmError::Validation("干系人姓名不能为空".into()));
        }
        if self.influence.is_some_and(|v| !(1..=5).contains(&v))
            || self.interest.is_some_and(|v| !(1..=5).contains(&v))
        {
            return Err(HelmError::Validation("权力/利益取值须在 1~5 之间".into()));
        }
        Ok(())
    }
}

/// 项目章程（分节结构化）。S4 AI 起草按同一结构逐节生成 → 预览采纳。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CharterContent {
    #[serde(default)]
    pub objective: String,
    #[serde(default)]
    pub success_criteria: String,
    #[serde(default)]
    pub key_deliverables: String,
    #[serde(default)]
    pub milestones: String,
    #[serde(default)]
    pub constraints: String,
    #[serde(default)]
    pub assumptions: String,
    #[serde(default)]
    pub high_level_risks: String,
    #[serde(default)]
    pub pm_authority: String,
}

/// 章程读取结果。`created` = 本次调用新建了默认空章程。
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CharterEnvelope {
    pub id: String,
    pub project_id: String,
    pub charter: CharterContent,
    pub updated_at: String,
    pub created: bool,
}

// ---------- S4：AI 生成溯源 ----------

db_enum!(
    /// AI 生成类型。
    AiArtifactKind, default(Coach => "coach"), {
        CharterDraft => "charter_draft",
        WbsDraft => "wbs_draft",
    }
);

db_enum!(
    /// AI 生成结果状态：草稿（未处理）/ 已采纳 / 已弃用。
    AiArtifactStatus, default(Draft => "draft"), {
        Adopted => "adopted",
        Discarded => "discarded",
    }
);

/// AI 生成记录（溯源）：支撑"可溯源、可拒绝、可撤销"。
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiArtifact {
    pub id: String,
    pub project_id: Option<String>,
    pub kind: AiArtifactKind,
    pub provider: String,
    pub model: String,
    pub input_summary: String,
    pub output: String,
    pub status: AiArtifactStatus,
    pub created_at: String,
}

// ---------- S6：挣值管理（工时台账与 EVM 报表） ----------

/// 工时台账条目：AC（实际成本，以人时计）的唯一事实来源。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeLog {
    pub id: String,
    pub project_id: String,
    pub task_id: String,
    pub log_date: String,
    pub hours: f64,
    pub note: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTimeLogInput {
    pub project_id: String,
    pub task_id: String,
    /// 工时归属日（YYYY-MM-DD，可早于今天补录）。
    pub log_date: String,
    pub hours: f64,
    pub note: Option<String>,
}

impl CreateTimeLogInput {
    pub fn validate(&self) -> Result<()> {
        if self.hours <= 0.0 || self.hours > 24.0 {
            return Err(HelmError::Validation(
                "工时须大于 0 且不超过 24 小时".into(),
            ));
        }
        if chrono::NaiveDate::parse_from_str(&self.log_date, "%Y-%m-%d").is_err() {
            return Err(HelmError::Validation("日期格式须为 YYYY-MM-DD".into()));
        }
        Ok(())
    }
}
