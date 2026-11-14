import { invoke } from "@tauri-apps/api/core";
import type { Channel } from "@tauri-apps/api/core";
import type {
  AiArtifact,
  AiArtifactStatusType,
  AiSettingsView,
  Baseline,
  CharterContent,
  CharterEnvelope,
  CharterDraftOut,
  CreateDependencyInput,
  CreateProjectInput,
  CreateRiskInput,
  CreateStakeholderInput,
  CreateTaskInput,
  DashboardStats,
  Dependency,
  MoveTaskInput,
  Project,
  ProjectSchedule,
  ReorderTaskInput,
  Risk,
  Stakeholder,
  Task,
  UpdateDependencyInput,
  UpdateProjectInput,
  UpdateRiskInput,
  UpdateStakeholderInput,
  UpdateTaskInput,
  WbsDraftOut,
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

  // 风险登记册（S3）
  listRisks: (projectId: string) => invoke<Risk[]>("list_risks", { projectId }),
  createRisk: (input: CreateRiskInput) => invoke<Risk>("create_risk", { input }),
  updateRisk: (input: UpdateRiskInput) => invoke<Risk>("update_risk", { input }),
  deleteRisk: (id: string) => invoke<void>("delete_risk", { id }),

  // 干系人登记册（S3）
  listStakeholders: (projectId: string) =>
    invoke<Stakeholder[]>("list_stakeholders", { projectId }),
  createStakeholder: (input: CreateStakeholderInput) =>
    invoke<Stakeholder>("create_stakeholder", { input }),
  updateStakeholder: (input: UpdateStakeholderInput) =>
    invoke<Stakeholder>("update_stakeholder", { input }),
  deleteStakeholder: (id: string) => invoke<void>("delete_stakeholder", { id }),

  // 项目章程（S3）
  getCharter: (projectId: string) => invoke<CharterEnvelope>("get_charter", { projectId }),
  saveCharter: (projectId: string, content: CharterContent) =>
    invoke<CharterEnvelope>("save_charter", { projectId, content }),

  // 方法论裁剪（S3）
  setTailored: (projectId: string, keys: string[]) =>
    invoke<Project>("set_tailored", { projectId, keys }),
  applyRecommendedTailoring: (projectId: string) =>
    invoke<Project>("apply_recommended_tailoring", { projectId }),

  // AI 舵手（S4）
  getAiSettings: () => invoke<AiSettingsView>("get_ai_settings"),
  setAiSettings: (baseUrl: string, model: string, apiKey?: string | null) =>
    invoke<AiSettingsView>("set_ai_settings", { baseUrl, model, apiKey: apiKey ?? null }),
  testAiConnection: () => invoke<string>("test_ai_connection"),
  coachAsk: (
    channel: Channel<{ event: "delta" | "done"; text?: string }>,
    question: string,
    projectId?: string | null,
  ) => invoke<string>("coach_ask", { channel, question, projectId: projectId ?? null }),
  generateCharterDraft: (projectId: string) =>
    invoke<CharterDraftOut>("generate_charter_draft", { projectId }),
  generateWbsDraft: (projectId: string, requirements: string) =>
    invoke<WbsDraftOut>("generate_wbs_draft", { projectId, requirements }),
  listAiArtifacts: (projectId?: string | null) =>
    invoke<AiArtifact[]>("list_ai_artifacts", { projectId: projectId ?? null }),
  setArtifactStatus: (id: string, status: AiArtifactStatusType) =>
    invoke<AiArtifact>("set_artifact_status", { id, status }),
};
