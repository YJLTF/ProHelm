import { useState } from "react";
import { useNavigate } from "react-router";
import { FolderKanban, Plus } from "lucide-react";

import { ProjectCard } from "../components/projects/ProjectCard";
import { Button } from "../components/ui/Button";
import { Dialog } from "../components/ui/Dialog";
import { EmptyState } from "../components/ui/EmptyState";
import { Input, Label, Select, Textarea } from "../components/ui/Input";
import { useCreateProject, useProjects } from "../hooks/useProjects";
import { toast, toastError } from "../store/toast";
import { PROJECT_METHOD_LABEL } from "../types";
import type { ProjectMethod } from "../types";

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
        description="点击右上角「新建项目」开始；也可以在模板库中从行业模板快速起步（S3 上线）。"
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

function NewProjectDialog({ open, onClose }: { open: boolean; onClose: () => void }) {
  const navigate = useNavigate();
  const create = useCreateProject();
  const [name, setName] = useState("");
  const [code, setCode] = useState("");
  const [method, setMethod] = useState<ProjectMethod>("hybrid");
  const [startDate, setStartDate] = useState("");
  const [targetEndDate, setTargetEndDate] = useState("");
  const [bac, setBac] = useState("");
  const [description, setDescription] = useState("");

  const reset = () => {
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
    create.mutate(
      {
        name,
        code: code || null,
        method,
        description: description || null,
        startDate: startDate || null,
        targetEndDate: targetEndDate || null,
        bac: bac ? Number(bac) : 0,
      },
      {
        onSuccess: (project) => {
          toast("项目创建成功");
          reset();
          onClose();
          navigate(`/projects/${project.id}`);
        },
      },
    );
  };

  return (
    <Dialog open={open} title="新建项目" onClose={onClose} className="max-w-xl">
      <form
        className="space-y-3"
        onSubmit={(e) => {
          e.preventDefault();
          submit();
        }}
      >
        <div className="grid grid-cols-2 gap-3">
          <div>
            <Label required>项目名称</Label>
            <Input value={name} onChange={(e) => setName(e.target.value)} placeholder="例如：某商城 App 二期" autoFocus />
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
            <Input type="date" value={targetEndDate} onChange={(e) => setTargetEndDate(e.target.value)} />
          </div>
        </div>
        <div>
          <Label>项目简述</Label>
          <Textarea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder="一句话说明项目目标与成功标准——这也是后续 AI 起草章程的输入（S4 上线）"
          />
        </div>
        <div className="flex justify-end gap-2 pt-1">
          <Button type="button" variant="secondary" onClick={onClose}>
            取消
          </Button>
          <Button type="submit" disabled={create.isPending}>
            创建项目
          </Button>
        </div>
      </form>
    </Dialog>
  );
}
