import { invoke } from "@tauri-apps/api/core";
import type {
  Baseline,
  CreateDependencyInput,
  CreateProjectInput,
  CreateTaskInput,
  DashboardStats,
  Dependency,
  MoveTaskInput,
  Project,
  ProjectSchedule,
  ReorderTaskInput,
  Task,
  UpdateDependencyInput,
  UpdateProjectInput,
  UpdateTaskInput,
} from "../types";

/// 类型化 IPC 层：函数签名与 src-tauri/src/commands.rs 一一对应。
export const api = {
  // 项目
  listProjects: () => invoke<Project[]>("list_projects"),
  createProject: (input: CreateProjectInput) =>
    invoke<Project>("create_project", { input }),
  updateProject: (input: UpdateProjectInput) =>
    invoke<Project>("update_project", { input }),
  deleteProject: (id: string) => invoke<void>("delete_project", { id }),

  // 任务
  listTasks: (projectId: string) => invoke<Task[]>("list_tasks", { projectId }),
  createTask: (input: CreateTaskInput) => invoke<Task>("create_task", { input }),
  updateTask: (input: UpdateTaskInput) => invoke<Task>("update_task", { input }),
  deleteTask: (id: string) => invoke<void>("delete_task", { id }),

  // 驾驶舱
  dashboardStats: () => invoke<DashboardStats>("get_dashboard_stats"),

  // 进度：排程 / 依赖（S2）
  getSchedule: (projectId: string) =>
    invoke<ProjectSchedule>("get_schedule", { projectId }),
  createDependency: (input: CreateDependencyInput) =>
    invoke<Dependency>("create_dependency", { input }),
  updateDependency: (input: UpdateDependencyInput) =>
    invoke<Dependency>("update_dependency", { input }),
  deleteDependency: (id: string) => invoke<void>("delete_dependency", { id }),

  // WBS 重排（S2）
  moveTask: (input: MoveTaskInput) => invoke<Task>("move_task", { input }),
  reorderTask: (input: ReorderTaskInput) => invoke<Task>("reorder_task", { input }),

  // 基线（S2）
  createBaseline: (projectId: string, name: string) =>
    invoke<Baseline>("create_baseline", { projectId, name }),
  listBaselines: (projectId: string) =>
    invoke<Baseline[]>("list_baselines", { projectId }),
};
