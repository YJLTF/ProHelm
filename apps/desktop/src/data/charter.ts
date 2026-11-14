/** 章程分节定义（CharterEditor 与 AI 草稿预览共用；key 与后端 CharterContent 字段一致） */
import type { CharterContent } from "../types";

export interface CharterSectionDef {
  key: keyof CharterContent;
  label: string;
  placeholder: string;
}

export const CHARTER_SECTION_DEFS: CharterSectionDef[] = [
  { key: "objective", label: "项目目的与目标", placeholder: "为什么做这个项目？要达成什么？" },
  { key: "successCriteria", label: "成功标准（可度量）", placeholder: "例如：UAT 一次通过；上线后崩溃率 < 0.1%" },
  { key: "keyDeliverables", label: "主要交付物", placeholder: "系统 / 文档 / 报告…" },
  { key: "milestones", label: "关键里程碑", placeholder: "W3 需求冻结 / W7 开发完成…" },
  { key: "constraints", label: "制约因素", placeholder: "工期、预算、范围、合规…" },
  { key: "assumptions", label: "假设条件", placeholder: "客户按时提供测试环境与数据…" },
  { key: "highLevelRisks", label: "高层级风险", placeholder: "详细管理见「规划 → 风险登记册」" },
  { key: "pmAuthority", label: "项目经理职权", placeholder: "可动用资源、审批权限、对外承诺边界" },
];
