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
#[derive(Debug, Clone, PartialEq, Serialize)]
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
    pub created_at: String,
    pub updated_at: String,
}

/// 任务（任务 = WBS 叶子工作包的最小执行单元；S2 引入层级依赖）。
#[derive(Debug, Clone, PartialEq, Serialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Baseline {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub snapshot: String,
    pub created_at: String,
}
