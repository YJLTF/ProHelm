/// 模板与方法论裁剪的前端数据源（与 helm-core/src/tailoring.rs 的 key 对应）。
import type {
  CharterContent,
  ProjectMethod,
  ResponseStrategy,
  RiskCategory,
} from "../types";

// ---------- 方法论裁剪项（key 与后端 tailoring::ALL_ITEMS 一致） ----------

export interface TailoringGroup {
  focus: string;
  items: Array<{ key: string; label: string }>;
}

export const TAILORING_GROUPS: TailoringGroup[] = [
  {
    focus: "启动",
    items: [
      { key: "charter", label: "项目章程" },
      { key: "stakeholder", label: "干系人登记册" },
    ],
  },
  {
    focus: "规划",
    items: [
      { key: "wbs", label: "WBS 分解" },
      { key: "schedule", label: "排期与关键路径" },
      { key: "budget", label: "预算与估算" },
      { key: "risk", label: "风险登记册" },
      { key: "communication", label: "沟通计划" },
      { key: "procurement", label: "采购计划" },
      { key: "quality", label: "质量计划" },
    ],
  },
  {
    focus: "执行",
    items: [
      { key: "kanban", label: "任务看板" },
      { key: "timesheet", label: "工时记录" },
      { key: "meeting", label: "会议与行动项" },
    ],
  },
  {
    focus: "监控",
    items: [
      { key: "evm", label: "挣值管理" },
      { key: "change_control", label: "变更控制" },
      { key: "issue", label: "问题登记册" },
    ],
  },
  {
    focus: "收尾",
    items: [
      { key: "acceptance", label: "验收清单" },
      { key: "lessons", label: "经验教训登记册" },
    ],
  },
];

// ---------- 行业项目模板 ----------

export interface TemplateTask {
  title: string;
  durationDays: number;
  /** 父任务标题（同一模板内先声明父级） */
  parentTitle?: string;
  /** FS 前置任务标题 */
  predecessorTitle?: string;
  notes?: string;
}

export interface TemplateRisk {
  title: string;
  category: RiskCategory;
  probability: number;
  impact: number;
  responseStrategy: ResponseStrategy;
}

export interface ProjectTemplate {
  key: string;
  name: string;
  desc: string;
  method: ProjectMethod;
  tag: "敏捷" | "预测型" | "混合";
  tasks: TemplateTask[];
  risks: TemplateRisk[];
  charter?: Partial<CharterContent>;
}

export const PROJECT_TEMPLATES: ProjectTemplate[] = [
  {
    key: "software-agile",
    name: "软件交付 · 敏捷",
    desc: "6 周迭代节奏：待办梳理 → 迭代开发 → 评审回顾，附敏捷常见风险",
    method: "agile",
    tag: "敏捷",
    tasks: [
      { title: "产品待办梳理", durationDays: 2 },
      { title: "迭代规划会", durationDays: 1, predecessorTitle: "产品待办梳理" },
      { title: "核心功能开发", durationDays: 5, predecessorTitle: "迭代规划会" },
      { title: "集成联调", durationDays: 3, predecessorTitle: "核心功能开发" },
      { title: "迭代评审与回顾", durationDays: 1, predecessorTitle: "集成联调" },
    ],
    risks: [
      { title: "需求变更频繁，待办范围失控", category: "external", probability: 4, impact: 4, responseStrategy: "mitigate" },
      { title: "关键开发成员临时不可用", category: "resource", probability: 2, impact: 3, responseStrategy: "mitigate" },
      { title: "第三方服务/接口延期", category: "external", probability: 3, impact: 4, responseStrategy: "mitigate" },
    ],
    charter: {
      objective: "在 6 周内交付核心功能可用的 MVP 版本",
      successCriteria: "核心流程端到端跑通；每个迭代按时发布且无致命缺陷",
      keyDeliverables: "可运行系统、迭代评审记录、发布说明",
      milestones: "W2 迭代一评审 / W4 迭代二评审 / W6 正式发布",
    },
  },
  {
    key: "software-waterfall",
    name: "软件交付 · 预测型",
    desc: "需求→设计→开发→测试→UAT→上线 全链路，含 FS 依赖链与里程碑",
    method: "predictive",
    tag: "预测型",
    tasks: [
      { title: "需求规格说明", durationDays: 3 },
      { title: "方案设计", durationDays: 4, predecessorTitle: "需求规格说明" },
      { title: "开发实现", durationDays: 10, predecessorTitle: "方案设计" },
      { title: "系统测试", durationDays: 4, predecessorTitle: "开发实现" },
      { title: "UAT 验收", durationDays: 2, predecessorTitle: "系统测试" },
      { title: "上线部署", durationDays: 1, predecessorTitle: "UAT 验收" },
    ],
    risks: [
      { title: "需求蔓延导致工期失守", category: "schedule", probability: 4, impact: 4, responseStrategy: "mitigate" },
      { title: "性能指标不达标需返工", category: "technical", probability: 3, impact: 4, responseStrategy: "mitigate" },
      { title: "第三方接口延期阻塞联调", category: "external", probability: 3, impact: 4, responseStrategy: "mitigate" },
    ],
    charter: {
      objective: "按需求规格在计划工期内交付可验收的系统",
      successCriteria: "UAT 一次通过；上线后一周内无 P0/P1 级故障",
      keyDeliverables: "需求规格、系统、测试报告、验收报告",
      milestones: "W3 需求冻结 / W7 开发完成 / W11 UAT 通过 / W12 上线",
    },
  },
  {
    key: "consulting-delivery",
    name: "乙方咨询交付",
    desc: "启动→调研→诊断→方案→试点→移交→验收，干系人协同重",
    method: "predictive",
    tag: "预测型",
    tasks: [
      { title: "项目启动会", durationDays: 1 },
      { title: "现状调研", durationDays: 5, predecessorTitle: "项目启动会" },
      { title: "诊断报告", durationDays: 2, predecessorTitle: "现状调研" },
      { title: "方案设计", durationDays: 8, predecessorTitle: "诊断报告" },
      { title: "试点实施", durationDays: 6, predecessorTitle: "方案设计" },
      { title: "培训与移交", durationDays: 3, predecessorTitle: "试点实施" },
      { title: "验收收尾", durationDays: 2, predecessorTitle: "培训与移交" },
    ],
    risks: [
      { title: "客户方配合延迟（数据/访谈）", category: "external", probability: 4, impact: 3, responseStrategy: "mitigate" },
      { title: "交付范围边界争议", category: "other", probability: 3, impact: 4, responseStrategy: "mitigate" },
      { title: "客户内部数据获取困难", category: "external", probability: 2, impact: 3, responseStrategy: "accept" },
    ],
    charter: {
      objective: "按合同范围完成咨询交付并获得客户正式验收",
      successCriteria: "诊断/方案/试点三项交付物客户签字确认",
      keyDeliverables: "诊断报告、实施方案、试点成果、移交清单",
      milestones: "W1 启动 / W3 诊断评审 / W7 方案评审 / W10 试点验收",
    },
  },
  {
    key: "design-creative",
    name: "设计创意项目",
    desc: "Brief→概念→主视觉→延展→评审→交付，评审节点与改稿风险管理",
    method: "hybrid",
    tag: "混合",
    tasks: [
      { title: "创意 Brief 确认", durationDays: 1 },
      { title: "概念提案", durationDays: 3, predecessorTitle: "创意 Brief 确认" },
      { title: "主视觉设计", durationDays: 5, predecessorTitle: "概念提案" },
      { title: "延展物料设计", durationDays: 4, predecessorTitle: "主视觉设计" },
      { title: "客户评审与修改", durationDays: 3, predecessorTitle: "延展物料设计" },
      { title: "交付与归档", durationDays: 1, predecessorTitle: "客户评审与修改" },
    ],
    risks: [
      { title: "客户反复改稿消耗工期", category: "other", probability: 4, impact: 3, responseStrategy: "mitigate" },
      { title: "素材版权风险", category: "external", probability: 2, impact: 4, responseStrategy: "transfer" },
    ],
    charter: {
      objective: "交付客户认可的品牌视觉方案及全套延展物料",
      successCriteria: "客户评审一次通过率 ≥ 80%；物料按期交付",
      keyDeliverables: "概念提案、主视觉、延展物料包、源文件归档",
      milestones: "W1 Brief / W2 概念评审 / W4 主视觉评审 / W6 交付",
    },
  },
];

// ---------- PMBOK 工件说明卡 ----------

export const ARTIFACT_CARDS: Array<{ name: string; desc: string; where: string }> = [
  { name: "项目章程", desc: "分节结构化编辑，S4 接入 AI 逐节起草", where: "启动页签" },
  { name: "干系人登记册", desc: "权力/利益方格 + 管理策略推导", where: "启动页签" },
  { name: "风险登记册", desc: "概率×影响矩阵（1~5 标度）+ 应对策略", where: "规划页签" },
  { name: "方法论裁剪器", desc: "按开发方法生成最小必要流程集", where: "启动页签" },
  { name: "RACI 责任矩阵", desc: "接入成员表后启用（S5）", where: "规划页签" },
  { name: "变更请求单", desc: "变更控制流（S5）", where: "监控页签" },
];
