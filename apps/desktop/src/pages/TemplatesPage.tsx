import { useState } from "react";
import { useNavigate } from "react-router";
import { useQueryClient } from "@tanstack/react-query";
import { FileText, LayoutTemplate, Plus } from "lucide-react";

import { Badge } from "../components/ui/Badge";
import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import { Dialog } from "../components/ui/Dialog";
import { Input, Label } from "../components/ui/Input";
import { api } from "../lib/ipc";
import { toast, toastError } from "../store/toast";
import { ARTIFACT_CARDS, PROJECT_TEMPLATES } from "../data/templates";
import type { ProjectTemplate } from "../data/templates";
import { PROJECT_METHOD_LABEL } from "../types";
import type { CharterContent } from "../types";

interface TemplateCardProps {
  template: ProjectTemplate;
  onApply: (t: ProjectTemplate) => void;
}

function TemplateCard({ template, onApply }: TemplateCardProps) {
  const depCount = template.tasks.filter((t) => t.predecessorTitle).length;
  return (
    <Card className="flex h-full flex-col justify-between">
      <div>
        <div className="flex items-start justify-between gap-2">
          <div className="text-sm font-semibold text-slate-800">{template.name}</div>
          <Badge tone={template.tag === "敏捷" ? "green" : template.tag === "预测型" ? "blue" : "amber"}>
            {template.tag}
          </Badge>
        </div>
        <div className="mt-1.5 text-xs leading-relaxed text-slate-500">{template.desc}</div>
        <div className="mt-2 flex gap-1.5">
          <Badge>{template.tasks.length} 个任务</Badge>
          {depCount > 0 && <Badge tone="blue">{depCount} 条依赖</Badge>}
          <Badge tone="amber">{template.risks.length} 个风险</Badge>
        </div>
      </div>
      <div className="mt-3">
        <Button size="sm" onClick={() => onApply(template)}>
          <Plus size={13} /> 使用模板创建
        </Button>
      </div>
    </Card>
  );
}

function ApplyTemplateDialog({
  template,
  onClose,
}: {
  template: ProjectTemplate | null;
  onClose: () => void;
}) {
  const navigate = useNavigate();
  const qc = useQueryClient();
  const [name, setName] = useState("");
  const [code, setCode] = useState("");
  const [startDate, setStartDate] = useState("");
  const [targetEndDate, setTargetEndDate] = useState("");
  const [busy, setBusy] = useState(false);

  if (!template) return null;

  const apply = async () => {
    if (!name.trim()) {
      toastError("请填写项目名称");
      return;
    }
    setBusy(true);
    try {
      const project = await api.createProject({
        name,
        code: code || null,
        method: template.method,
        description: template.charter?.objective
          ? `【${template.name}】${template.charter.objective}`
          : null,
        startDate: startDate || null,
        targetEndDate: targetEndDate || null,
      });
      // 任务两遍创建：先父后子；同时按 predecessorTitle 建立 FS 依赖
      const idByTitle = new Map<string, string>();
      for (const t of template.tasks) {
        const created = await api.createTask({
          projectId: project.id,
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
              projectId: project.id,
              predecessorId: pred,
              successorId: created.id,
              depType: "fs",
              lagDays: 0,
            });
          }
        }
      }
      for (const r of template.risks) {
        await api.createRisk({ projectId: project.id, ...r });
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
        await api.saveCharter(project.id, charter);
      }
      void qc.invalidateQueries();
      toast(`已从「${template.name}」创建项目`);
      onClose();
      navigate(`/projects/${project.id}`);
    } catch (e) {
      toastError((e as { message?: string }).message ?? "创建失败，请重试");
    } finally {
      setBusy(false);
    }
  };

  return (
    <Dialog open title={`从模板创建：${template.name}`} onClose={onClose} className="max-w-lg">
      <form
        className="space-y-3"
        onSubmit={(e) => {
          e.preventDefault();
          void apply();
        }}
      >
        <p className="rounded-lg bg-blue-50 px-3 py-2 text-xs leading-relaxed text-blue-700">
          将自动创建：{template.tasks.length} 个任务（含 {template.tasks.filter((t) => t.predecessorTitle).length} 条依赖）、
          {template.risks.length} 条风险、章程草稿，并按 {PROJECT_METHOD_LABEL[template.method]} 预置方法论裁剪。
        </p>
        <div className="grid grid-cols-2 gap-3">
          <div>
            <Label required>项目名称</Label>
            <Input value={name} onChange={(e) => setName(e.target.value)} autoFocus placeholder="例如：某商城 App 二期" />
          </div>
          <div>
            <Label>项目编号</Label>
            <Input value={code} onChange={(e) => setCode(e.target.value)} placeholder="选填" />
          </div>
          <div>
            <Label>开始日期</Label>
            <Input type="date" value={startDate} onChange={(e) => setStartDate(e.target.value)} />
          </div>
          <div>
            <Label>目标完成日期</Label>
            <Input type="date" value={targetEndDate} onChange={(e) => setTargetEndDate(e.target.value)} />
          </div>
        </div>
        <div className="flex justify-end gap-2 pt-1">
          <Button type="button" variant="secondary" onClick={onClose}>
            取消
          </Button>
          <Button type="submit" disabled={busy}>
            {busy ? "创建中…" : "创建项目"}
          </Button>
        </div>
      </form>
    </Dialog>
  );
}

export function TemplatesPage() {
  const [applying, setApplying] = useState<ProjectTemplate | null>(null);

  return (
    <div className="mx-auto max-w-6xl px-8 py-6">
      <header className="mb-5">
        <h1 className="flex items-center gap-2 text-lg font-bold text-slate-900">
          <LayoutTemplate size={18} className="text-blue-600" /> 模板与工件库
        </h1>
        <p className="mt-0.5 text-xs text-slate-500">
          行业模板一键创建完整项目脚手架（WBS / 依赖 / 风险 / 章程）；PMBOK
          工件已内置在项目工作区对应页签
        </p>
      </header>

      <h2 className="mb-2 text-sm font-semibold text-slate-700">行业项目模板</h2>
      <div className="grid grid-cols-2 gap-3">
        {PROJECT_TEMPLATES.map((t) => (
          <TemplateCard key={t.key} template={t} onApply={setApplying} />
        ))}
      </div>

      <h2 className="mb-2 mt-6 text-sm font-semibold text-slate-700">PMBOK 工件</h2>
      <div className="grid grid-cols-3 gap-3">
        {ARTIFACT_CARDS.map((a) => (
          <Card key={a.name}>
            <div className="flex items-start gap-2">
              <FileText size={15} className="mt-0.5 shrink-0 text-slate-300" />
              <div className="min-w-0">
                <div className="text-sm font-semibold text-slate-800">{a.name}</div>
                <div className="mt-0.5 text-xs text-slate-500">{a.desc}</div>
                <div className="mt-1.5">
                  <Badge tone="blue">{a.where}</Badge>
                </div>
              </div>
            </div>
          </Card>
        ))}
      </div>

      <ApplyTemplateDialog template={applying} onClose={() => setApplying(null)} />
    </div>
  );
}
