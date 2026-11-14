//! helm-core：ProHelm 的核心业务层。
//!
//! 设计原则（见 docs/03-技术选型与架构设计.md）：
//! - 所有业务逻辑与数据访问都收敛在本 crate，UI 层（Tauri 命令/前端）只做编排；
//! - 未来移动端 / CLI / MCP 复用本 crate；
//! - 所有写操作可扩展为 OpLog 事件（M3 同步的基础，见 0001 迁移注释）。

pub mod cpm;
pub mod db;
pub mod error;
pub mod migrations;
pub mod models;
pub mod repo;
pub mod util;

pub use error::{HelmError, Result};
pub use models::{
    Baseline, CreateDependencyInput, CreateProjectInput, CreateTaskInput, Dependency,
    DependencyType, MoveTaskInput, Project, ProjectMethod, ProjectStatus, ReorderTaskInput, Task,
    TaskMoveDirection, TaskPriority, TaskStatus, UpdateDependencyInput, UpdateProjectInput,
    UpdateTaskInput,
};
pub use repo::schedule::ProjectSchedule;
pub use repo::stats::DashboardStats;
