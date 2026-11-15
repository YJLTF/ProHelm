import { useEffect, useState } from "react";
import { toast, toastError } from "../../store/toast";
import { useUpdateProject } from "../../hooks/useProjects";
import { Button } from "../ui/Button";
import { Dialog } from "../ui/Dialog";
import { Input, Label, Select, Textarea } from "../ui/Input";
import { PROJECT_METHOD_LABEL, PROJECT_STATUS_LABEL } from "../../types";
import type { Project, ProjectMethod, ProjectStatus, UpdateProjectInput } from "../../types";

interface Props {
  project: Project;
  open: boolean;
  onClose: () => void;
}

const STATUS_HINT: Record<ProjectStatus, string> = {
  planning: "尚未开始执行；创建首个任务后会自动转入进行中",
  active: "正常推进",
  paused: "暂缓推进；不计入驾驶舱「进行中」",
  closed: "已收尾归档；不再计入进行中与活跃统计",
};

/**
 * 项目基本信息编辑（含状态流转）。
 * 日期为 COALESCE 语义：清空后保存 = 保留原值（空串会破坏排程锚点，见 conventions §4）。
 */
export function ProjectEditDialog({ project, open, onClose }: Props) {
  const update = useUpdateProject();
  const [name, setName] = useState(project.name);
  const [code, setCode] = useState(project.code ?? "");
  const [method, setMethod] = useState<ProjectMethod>(project.method);
  const [status, setStatus] = useState<ProjectStatus>(project.status);
  const [startDate, setStartDate] = useState(project.startDate ?? "");
  const [targetEndDate, setTargetEndDate] = useState(project.targetEndDate ?? "");
  const [bac, setBac] = useState(project.bac > 0 ? String(project.bac) : "");
  const [description, setDescription] = useState(project.description ?? "");

  // 打开或项目数据刷新时，以最新值为准
  useEffect(() => {
    if (!open) return;
    setName(project.name);
    setCode(project.code ?? "");
    setMethod(project.method);
    setStatus(project.status);
    setStartDate(project.startDate ?? "");
    setTargetEndDate(project.targetEndDate ?? "");
    setBac(project.bac > 0 ? String(project.bac) : "");
    setDescription(project.description ?? "");
  }, [open, project]);

  const submit = () => {
    const trimmed = name.trim();
    if (!trimmed) {
      toastError("项目名称不能为空");
      return;
    }
    // 文本字段传空串 = 清空；日期字段传 null = 保持原值（COALESCE 语义）
    const input: UpdateProjectInput = {
      id: project.id,
      name: trimmed,
      code: code.trim(),
      method,
      status,
      description: description.trim(),
      startDate: startDate || null,
      targetEndDate: targetEndDate || null,
      bac: bac === "" ? 0 : Number(bac) || 0,
    };
    update.mutate(input, {
      onSuccess: () => {
        toast("项目信息已保存");
        onClose();
      },
    });
  };

  return (
    <Dialog open={open} title="编辑项目信息" onClose={onClose} className="max-w-xl">
      <div className="space-y-3">
        <div className="grid grid-cols-2 gap-3">
          <div>
            <Label required>项目名称</Label>
            <Input value={name} onChange={(e) => setName(e.target.value)} autoFocus />
          </div>
          <div>
            <Label>项目编号</Label>
            <Input
              value={code}
              onChange={(e) => setCode(e.target.value)}
              placeholder="清空后保存即删除编号"
            />
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
            <Label>项目状态</Label>
            <Select value={status} onChange={(e) => setStatus(e.target.value as ProjectStatus)}>
              {(Object.keys(PROJECT_STATUS_LABEL) as ProjectStatus[]).map((s) => (
                <option key={s} value={s}>
                  {PROJECT_STATUS_LABEL[s]}
                </option>
              ))}
            </Select>
            <p className="mt-1 text-[11px] text-slate-400">{STATUS_HINT[status]}</p>
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
          <div>
            <Label>总预算（元）</Label>
            <Input
              type="number"
              min={0}
              value={bac}
              onChange={(e) => setBac(e.target.value)}
              placeholder="BAC"
            />
          </div>
        </div>
        <div>
          <Label>项目简述</Label>
          <Textarea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder="一句话目标与成功标准——也是 AI 起草章程的输入"
          />
        </div>
        <p className="text-[11px] text-slate-400">
          日期字段暂不支持清空：清空后保存将保留原值；开始日期是甘特图/关键路径的排程锚点。
        </p>
        <div className="flex justify-end gap-2 pt-1">
          <Button variant="secondary" onClick={onClose} disabled={update.isPending}>
            取消
          </Button>
          <Button onClick={submit} disabled={update.isPending}>
            {update.isPending ? "保存中…" : "保存"}
          </Button>
        </div>
      </div>
    </Dialog>
  );
}
