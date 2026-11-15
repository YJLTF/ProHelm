import { api } from "./ipc";
import { toastError } from "../store/toast";
import type { ProjectTemplate } from "../data/templates";
import type { CharterContent, CreateProjectInput, Project } from "../types";

/**
 * 统一的"新建项目"落库流程：项目 + 模板任务树（含 FS 依赖）+ 风险 + 章程脚手架。
 * 模板为 null 时只创建空白项目。创建中途失败时保留已写入内容并提示（本地操作可删除重来）。
 */
export async function createProjectWithTemplate(
  input: CreateProjectInput,
  template: ProjectTemplate | null,
): Promise<Project> {
  const project = await api.createProject({
    ...input,
    method: template ? template.method : (input.method ?? "hybrid"),
    description:
      input.description ||
      (template?.charter?.objective ? `【${template.name}】${template.charter.objective}` : null),
  });
  if (!template) return project;

  try {
    await applyTemplateContent(project.id, template);
  } catch (e) {
    toastError(
      (e as { message?: string }).message ??
        "项目已创建，但模板内容写入中断；可在项目内手动补充",
    );
  }
  return project;
}

/** 把模板的任务/依赖/风险/章程写入已存在的项目。 */
export async function applyTemplateContent(projectId: string, template: ProjectTemplate) {
  // 任务两遍创建：先父后子；同时按 predecessorTitle 建立 FS 依赖
  const idByTitle = new Map<string, string>();
  for (const t of template.tasks) {
    const created = await api.createTask({
      projectId,
      title: t.title,
      parentId: t.parentTitle ? (idByTitle.get(t.parentTitle) ?? null) : null,
      durationDays: t.durationDays,
      notes: t.notes ?? null,
    });
    idByTitle.set(t.title, created.id);
    if (t.predecessorTitle) {
      const pred = idByTitle.get(t.predecessorTitle);
      if (pred) {
        await api.createDependency({
          projectId,
          predecessorId: pred,
          successorId: created.id,
          depType: "fs",
          lagDays: 0,
        });
      }
    }
  }
  for (const r of template.risks) {
    await api.createRisk({ projectId, ...r });
  }
  if (template.charter) {
    const charter: CharterContent = {
      objective: "",
      successCriteria: "",
      keyDeliverables: "",
      milestones: "",
      constraints: "",
      assumptions: "",
      highLevelRisks: "",
      pmAuthority: "",
      ...template.charter,
    };
    await api.saveCharter(projectId, charter);
  }
}
