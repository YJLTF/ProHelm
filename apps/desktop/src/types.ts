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
  /** 方法论裁剪：启用的流程项 key（见 src/data/templates.ts 的 TAILORING_GROUPS） */
  tailored: string[] | null;
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
  durationDays: number;
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
  durationDays?: number;
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
  durationDays?: number;
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

// ---------- S2：进度（依赖 / 排程 / 基线） ----------

export type DependencyType = "fs" | "ss" | "ff" | "sf";

export interface Dependency {
  id: string;
  projectId: string;
  predecessorId: string;
  successorId: string;
  depType: DependencyType;
  lagDays: number;
  createdAt: string;
}

export interface CreateDependencyInput {
  projectId: string;
  predecessorId: string;
  successorId: string;
  depType?: DependencyType;
  lagDays?: number;
}

export interface UpdateDependencyInput {
  id: string;
  depType?: DependencyType;
  lagDays?: number;
}

export interface MoveTaskInput {
  taskId: string;
  newParentId: string | null;
}

export type TaskMoveDirection = "up" | "down";

export interface ReorderTaskInput {
  taskId: string;
  direction: TaskMoveDirection;
}

/** 排程视图的单任务条目（CPM 计算结果 + 锚点日历换算，日期均含当天） */
export interface ScheduleTask {
  taskId: string;
  wbsCode: string | null;
  title: string;
  startDate: string;
  endDate: string;
  esDays: number;
  efDays: number;
  totalFloatDays: number;
  critical: boolean;
  predecessorIds: string[];
}

export interface ProjectSchedule {
  projectId: string;
  anchorDate: string;
  durationDays: number;
  tasks: ScheduleTask[];
}

export interface Baseline {
  id: string;
  projectId: string;
  name: string;
  snapshot: string;
  createdAt: string;
}

export const DEP_TYPE_LABEL: Record<DependencyType, string> = {
  fs: "完成后开始 (FS)",
  ss: "开始后开始 (SS)",
  ff: "完成后完成 (FF)",
  sf: "开始后完成 (SF)",
};

// ---------- S3：风险 / 干系人 / 章程 ----------

export type RiskCategory =
  | "technical"
  | "external"
  | "organizational"
  | "resource"
  | "schedule"
  | "cost"
  | "quality"
  | "other";
export type ResponseStrategy = "avoid" | "transfer" | "mitigate" | "accept" | "escalate";
export type RiskStatusType = "open" | "occurred" | "closed";

export interface Risk {
  id: string;
  projectId: string;
  riskNo: number;
  title: string;
  detail: string | null;
  category: RiskCategory;
  probability: number;
  impact: number;
  responseStrategy: ResponseStrategy;
  responsePlan: string | null;
  owner: string | null;
  triggerDesc: string | null;
  status: RiskStatusType;
  createdAt: string;
  updatedAt: string;
}

export interface CreateRiskInput {
  projectId: string;
  title: string;
  detail?: string | null;
  category?: RiskCategory;
  probability?: number;
  impact?: number;
  responseStrategy?: ResponseStrategy;
  responsePlan?: string | null;
  owner?: string | null;
  triggerDesc?: string | null;
}

export interface UpdateRiskInput {
  id: string;
  title?: string | null;
  detail?: string | null;
  category?: RiskCategory;
  probability?: number;
  impact?: number;
  responseStrategy?: ResponseStrategy;
  responsePlan?: string | null;
  owner?: string | null;
  triggerDesc?: string | null;
  status?: RiskStatusType;
}

export const RISK_CATEGORY_LABEL: Record<RiskCategory, string> = {
  technical: "技术",
  external: "外部",
  organizational: "组织",
  resource: "资源",
  schedule: "进度",
  cost: "成本",
  quality: "质量",
  other: "其他",
};

export const RESPONSE_STRATEGY_LABEL: Record<ResponseStrategy, string> = {
  avoid: "规避",
  transfer: "转移",
  mitigate: "减轻",
  accept: "接受",
  escalate: "上报",
};

export const RISK_STATUS_LABEL: Record<RiskStatusType, string> = {
  open: "打开",
  occurred: "已发生",
  closed: "已关闭",
};

/** 评分阈值：>=15 红（高）、>=8 黄（中）、否则绿（低） */
export function riskScoreTone(score: number): "red" | "amber" | "green" {
  if (score >= 15) return "red";
  if (score >= 8) return "amber";
  return "green";
}

export type StakeholderStrategy = "manage_closely" | "keep_satisfied" | "keep_informed" | "monitor";

export interface Stakeholder {
  id: string;
  projectId: string;
  name: string;
  role: string | null;
  expectation: string | null;
  influence: number;
  interest: number;
  notes: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface CreateStakeholderInput {
  projectId: string;
  name: string;
  role?: string | null;
  expectation?: string | null;
  influence?: number;
  interest?: number;
  notes?: string | null;
}

export interface UpdateStakeholderInput {
  id: string;
  name?: string | null;
  role?: string | null;
  expectation?: string | null;
  influence?: number;
  interest?: number;
  notes?: string | null;
}

export const STAKEHOLDER_STRATEGY_LABEL: Record<StakeholderStrategy, string> = {
  manage_closely: "重点管理",
  keep_satisfied: "令其满意",
  keep_informed: "随时告知",
  monitor: "监控即可",
};

/** 权力/利益方格 → 管理策略（与后端 Stakeholder::strategy_key 一致） */
export function stakeholderStrategy(influence: number, interest: number): StakeholderStrategy {
  if (influence >= 4 && interest >= 4) return "manage_closely";
  if (influence >= 4) return "keep_satisfied";
  if (interest >= 4) return "keep_informed";
  return "monitor";
}

export interface CharterContent {
  objective: string;
  successCriteria: string;
  keyDeliverables: string;
  milestones: string;
  constraints: string;
  assumptions: string;
  highLevelRisks: string;
  pmAuthority: string;
}

export interface CharterEnvelope {
  id: string;
  projectId: string;
  charter: CharterContent;
  updatedAt: string;
  created: boolean;
}

export const EMPTY_CHARTER: CharterContent = {
  objective: "",
  successCriteria: "",
  keyDeliverables: "",
  milestones: "",
  constraints: "",
  assumptions: "",
  highLevelRisks: "",
  pmAuthority: "",
};

// ---------- S4：AI 舵手 ----------

export type AiArtifactKindType = "coach" | "charter_draft" | "wbs_draft";
export type AiArtifactStatusType = "draft" | "adopted" | "discarded";

export interface AiArtifact {
  id: string;
  projectId: string | null;
  kind: AiArtifactKindType;
  provider: string;
  model: string;
  inputSummary: string;
  output: string;
  status: AiArtifactStatusType;
  createdAt: string;
}

export const AI_KIND_LABEL: Record<AiArtifactKindType, string> = {
  coach: "教练问答",
  charter_draft: "章程草稿",
  wbs_draft: "WBS 拆解",
};

export const AI_STATUS_LABEL: Record<AiArtifactStatusType, string> = {
  draft: "草稿",
  adopted: "已采纳",
  discarded: "已弃用",
};

export interface AiSettingsView {
  baseUrl: string | null;
  model: string | null;
  hasApiKey: boolean;
}

export interface WbsDraftItem {
  title: string;
  durationDays: number;
  children: WbsDraftItem[];
}

export interface WbsDraftOut {
  artifactId: string;
  items: WbsDraftItem[];
}

export interface CharterDraftOut {
  artifactId: string;
  charter: CharterContent;
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
