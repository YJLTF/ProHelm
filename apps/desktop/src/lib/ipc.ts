import { invoke } from "@tauri-apps/api/core";
import type {
  CreateProjectInput,
  CreateTaskInput,
  DashboardStats,
  Project,
  Task,
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
};
