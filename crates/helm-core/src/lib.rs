//! helm-core：ProHelm 的核心业务层。
//!
//! 设计原则（见 docs/03-技术选型与架构设计.md）：
//! - 所有业务逻辑与数据访问都收敛在本 crate，UI 层（Tauri 命令/前端）只做编排；
//! - 未来移动端 / CLI / MCP 复用本 crate；
//! - 所有写操作可扩展为 OpLog 事件（M3 同步的基础，见 0001 迁移注释）。

pub mod cpm;
pub mod db;
pub mod error;
pub mod evm;
pub mod migrations;
pub mod models;
pub mod repo;
pub mod report;
pub mod snapshot;
pub mod tailoring;
pub mod util;

pub use error::{HelmError, Result};
pub use models::{
    AiArtifact, AiArtifactKind, AiArtifactStatus, Baseline, CharterContent, CharterEnvelope,
    CreateDependencyInput, CreateProjectInput, CreateRiskInput, CreateStakeholderInput,
    CreateTaskInput, CreateTimeLogInput, Dependency, DependencyType, MoveTaskInput, Project,
    ProjectMethod, ProjectStatus, ReorderTaskInput, ResponseStrategy, Risk, RiskCategory,
    RiskStatus, Stakeholder, Task, TaskMoveDirection, TaskPriority, TaskStatus, TimeLog,
    UpdateDependencyInput, UpdateProjectInput, UpdateRiskInput, UpdateStakeholderInput,
    UpdateTaskInput,
};
pub use repo::schedule::ProjectSchedule;
pub use repo::stats::DashboardStats;
pub use snapshot::{ImportStats, Snapshot};
