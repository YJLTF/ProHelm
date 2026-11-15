import { useState } from "react";
import { useNavigate } from "react-router";
import { useQueryClient } from "@tanstack/react-query";
import { FolderKanban, LayoutTemplate, Plus, Sparkles } from "lucide-react";

import { ProjectCard } from "../components/projects/ProjectCard";
import { Button } from "../components/ui/Button";
import { Dialog } from "../components/ui/Dialog";
import { EmptyState } from "../components/ui/EmptyState";
import { Input, Label, Select, Textarea } from "../components/ui/Input";
import { useProjects } from "../hooks/useProjects";
import { toast, toastError } from "../store/toast";
import { createProjectWithTemplate } from "../lib/templateApply";
import { PROJECT_METHOD_LABEL } from "../types";
import type { ProjectMethod } from "../types";
import { PROJECT_TEMPLATES } from "../data/templates";
import type { ProjectTemplate } from "../data/templates";

export function ProjectsPage() {
  const [dialogOpen, setDialogOpen] = useState(false);
  return (
    <div className="mx-auto max-w-6xl px-8 py-6">
      <header className="mb-5 flex items-center justify-between">
        <div>
          <h1 className="flex items-center gap-2 text-lg font-bold text-slate-900">
            <FolderKanban size={18} className="text-blue-600" /> 项目
          </h1>
          <p className="mt-0.5 text-xs text-slate-500">
            全部项目 · 点击卡片进入工作区（按 PMBOK 8 焦点领域组织）
          </p>
        </div>
        <Button onClick={() => setDialogOpen(true)}>
          <Plus size={15} /> 新建项目
        </Button>
      </header>

      <ProjectGrid />
      <NewProjectDialog open={dialogOpen} onClose={() => setDialogOpen(false)} />
    </div>
  );
}

function ProjectGrid() {
  const projects = useProjects();
  if (projects.isLoading) return <div className="text-sm text-slate-500">加载中…</div>;
  if (projects.isError)
    return <div className="text-sm text-red-600">项目加载失败：{projects.error.message}</div>;
  const list = projects.data ?? [];
  if (list.length === 0) {
    return (
      <EmptyState
        icon={<FolderKanban size={32} />}
        title="还没有项目"
        description="点击右上角「新建项目」：从空白开始，或选择行业模板一键生成 WBS、依赖、风险与章程脚手架。"
      />
    );
  }
  return (
    <div className="grid grid-cols-3 gap-3">
      {list.map((p) => (
        <ProjectCard key={p.id} project={p} />
      ))}
    </div>
  );
}

// ---------- 新建项目向导（S6） ----------

/** 起步方式：空白 + 行业模板（模板会带出任务树 / 风险 / 章程脚手架与方法预置）。 */
const BLANK_TEMPLATE: ProjectTemplate = {
  key: "blank",
  name: "空白项目",
  desc: "从零开始，按 PMBOK 焦点领域逐步完善",
  method: "hybrid",
  tag: "混合",
  tasks: [],
  risks: [],
};

function NewProjectDialog({ open, onClose }: { open: boolean; onClose: () => void }) {
  const navigate = useNavigate();
  const qc = useQueryClient();
  const [templateKey, setTemplateKey] = useState("blank");
  const [name, setName] = useState("");
  const [code, setCode] = useState("");
  const [method, setMethod] = useState<ProjectMethod>("hybrid");
  const [startDate, setStartDate] = useState("");
  const [targetEndDate, setTargetEndDate] = useState("");
  const [bac, setBac] = useState("");
  const [description, setDescription] = useState("");
  const [busy, setBusy] = useState(false);

  const template = [BLANK_TEMPLATE, ...PROJECT_TEMPLATES].find((t) => t.key === templateKey) ?? BLANK_TEMPLATE;
  const hasContent = template.tasks.length > 0;

  const pickTemplate = (key: string) => {
    setTemplateKey(key);
    const t = [BLANK_TEMPLATE, ...PROJECT_TEMPLATES].find((x) => x.key === key);
    if (t) setMethod(t.method); // 跟随模板预置开发方法（可改）
  };

  const reset = () => {
    setTemplateKey("blank");
    setName("");
    setCode("");
    setMethod("hybrid");
    setStartDate("");
    setTargetEndDate("");
    setBac("");
    setDescription("");
  };

  const submit = () => {
    if (!name.trim()) {
      toastError("请填写项目名称");
      return;
    }
    setBusy(true);
    const input = {
      name,
      code: code || null,
      method,
      description: description || null,
      startDate: startDate || null,
      targetEndDate: targetEndDate || null,
      bac: bac ? Number(bac) : 0,
    };
    void (async () => {
      try {
        const project = await createProjectWithTemplate(input, hasContent ? template : null);
        void qc.invalidateQueries();
        toast(hasContent ? `已从「${template.name}」创建项目` : "项目创建成功");
        reset();
        onClose();
        navigate(`/projects/${project.id}`);
      } catch (e) {
        toastError((e as { message?: string }).message ?? "创建失败，请重试");
      } finally {
        setBusy(false);
      }
    })();
  };

  return (
    <Dialog open={open} title="新建项目" onClose={onClose} className="max-w-2xl">
      <div className="space-y-3">
        {/* 第 1 步：起步方式 */}
        <div>
          <Label>起步方式</Label>
          <div className="grid grid-cols-3 gap-2">
            {[BLANK_TEMPLATE, ...PROJECT_TEMPLATES].map((t) => {
              const active = t.key === templateKey;
              return (
                <button
                  key={t.key}
                  type="button"
                  onClick={() => pickTemplate(t.key)}
                  className={`rounded-lg border p-2.5 text-left transition ${
                    active
                      ? "border-blue-500 bg-blue-50/60 ring-2 ring-blue-100"
                      : "border-slate-200 bg-white hover:border-slate-300"
                  }`}
                >
                  <div className="flex items-center gap-1.5">
                    {t.key === "blank" ? (
                      <Sparkles size={13} className="shrink-0 text-blue-500" />
                    ) : (
                      <LayoutTemplate size={13} className="shrink-0 text-slate-400" />
                    )}
                    <span className="truncate text-xs font-semibold text-slate-800">{t.name}</span>
                  </div>
                  <div className="mt-1 line-clamp-2 text-[11px] leading-snug text-slate-400">
                    {t.desc}
                  </div>
                </button>
              );
            })}
          </div>
          {hasContent && (
            <p className="mt-2 text-[11px] text-blue-600">
              将自动创建：{template.tasks.length} 个任务（含{" "}
              {template.tasks.filter((x) => x.predecessorTitle).length} 条依赖）、
              {template.risks.length} 条风险与章程脚手架
            </p>
          )}
        </div>

        {/* 第 2 步：基本信息 */}
        <div className="grid grid-cols-2 gap-3">
          <div>
            <Label required>项目名称</Label>
            <Input
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="例如：某商城 App 二期"
              autoFocus
            />
          </div>
          <div>
            <Label>项目编号</Label>
            <Input value={code} onChange={(e) => setCode(e.target.value)} placeholder="例如：PH-2026-018" />
          </div>
          <div>
            <Label>开发方法</Label>
            <Select value={method} onChange={(e) => setMethod(e.target.value as ProjectMethod)}>
              {(Object.keys(PROJECT_METHOD_LABEL) as ProjectMethod[]).map((m) => (
                <option key={m} value={m}>
                  {PROJECT_METHOD_LABEL[m]}
                </option>
              ))}
            </Select>
          </div>
          <div>
            <Label>总预算（元）</Label>
            <Input
              type="number"
              min={0}
              value={bac}
              onChange={(e) => setBac(e.target.value)}
              placeholder="BAC，用于后续挣值管理"
            />
          </div>
          <div>
            <Label>开始日期</Label>
            <Input type="date" value={startDate} onChange={(e) => setStartDate(e.target.value)} />
          </div>
          <div>
            <Label>目标完成日期</Label>
            <Input
              type="date"
              value={targetEndDate}
              onChange={(e) => setTargetEndDate(e.target.value)}
            />
          </div>
        </div>
        <div>
          <Label>项目简述</Label>
          <Textarea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder="一句话说明项目目标与成功标准——这也是后续 AI 起草章程的输入"
          />
        </div>
        <div className="flex justify-end gap-2 pt-1">
          <Button variant="secondary" onClick={onClose} disabled={busy}>
            取消
          </Button>
          <Button onClick={submit} disabled={busy}>
            {busy ? "创建中…" : "创建项目"}
          </Button>
        </div>
      </div>
    </Dialog>
  );
}
