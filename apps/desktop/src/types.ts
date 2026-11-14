/// 与 Rust 端 DTO 对齐的类型定义（helm-core models.rs，serde camelCase）。
/// 修改 Rust 模型时必须同步此文件。

export type ProjectMethod = "predictive" | "agile" | "hybrid";
export type ProjectStatus = "planning" | "active" | "paused" | "closed";
export type TaskStatus = "todo" | "in_progress" | "done";
export type TaskPriority = "low" | "medium" | "high" | "urgent";

export interface Project {
  id: string;
  name: string;
  code: string | null;
  method: ProjectMethod;
  status: ProjectStatus;
  description: string | null;
  startDate: string | null;
  targetEndDate: string | null;
  bac: number;
  createdAt: string;
  updatedAt: string;
}

export interface CreateProjectInput {
  name: string;
  code?: string | null;
  method?: ProjectMethod;
  description?: string | null;
  startDate?: string | null;
  targetEndDate?: string | null;
  bac?: number;
}

export interface UpdateProjectInput {
  id: string;
  name?: string | null;
  code?: string | null;
  method?: ProjectMethod;
  status?: ProjectStatus;
  description?: string | null;
  startDate?: string | null;
  targetEndDate?: string | null;
  bac?: number;
}

export interface Task {
  id: string;
  projectId: string;
  parentId: string | null;
  wbsCode: string | null;
  title: string;
  notes: string | null;
  status: TaskStatus;
  priority: TaskPriority;
  estimateHours: number;
  actualHours: number;
  startDate: string | null;
  endDate: string | null;
  assignee: string | null;
  sortOrder: number;
  createdAt: string;
  updatedAt: string;
}

export interface CreateTaskInput {
  projectId: string;
  parentId?: string | null;
  title: string;
  notes?: string | null;
  priority?: TaskPriority;
  estimateHours?: number;
  startDate?: string | null;
  endDate?: string | null;
  assignee?: string | null;
}

export interface UpdateTaskInput {
  id: string;
  title?: string | null;
  notes?: string | null;
  status?: TaskStatus;
  priority?: TaskPriority;
  estimateHours?: number;
  actualHours?: number;
  startDate?: string | null;
  endDate?: string | null;
  assignee?: string | null;
  sortOrder?: number;
}

export interface DashboardStats {
  projectCount: number;
  activeProjects: number;
  tasksTodo: number;
  tasksInProgress: number;
  tasksDone: number;
  overdueTasks: number;
  dueSoonTasks: number;
}

export const PROJECT_METHOD_LABEL: Record<ProjectMethod, string> = {
  predictive: "预测型",
  agile: "敏捷",
  hybrid: "混合",
};

export const PROJECT_STATUS_LABEL: Record<ProjectStatus, string> = {
  planning: "规划中",
  active: "进行中",
  paused: "已暂停",
  closed: "已关闭",
};

export const TASK_STATUS_LABEL: Record<TaskStatus, string> = {
  todo: "待办",
  in_progress: "进行中",
  done: "已完成",
};

export const TASK_PRIORITY_LABEL: Record<TaskPriority, string> = {
  low: "低",
  medium: "中",
  high: "高",
  urgent: "紧急",
};
