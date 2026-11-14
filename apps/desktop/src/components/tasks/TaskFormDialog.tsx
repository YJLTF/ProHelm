import { useEffect, useState } from "react";

import { Button } from "../ui/Button";
import { Dialog } from "../ui/Dialog";
import { Input, Label, Select, Textarea } from "../ui/Input";
import {
  useCreateDependency,
  useCreateTask,
  useUpdateTask,
} from "../../hooks/useProjects";
import { toast, toastError } from "../../store/toast";
import { TASK_PRIORITY_LABEL } from "../../types";
import type {
  CreateTaskInput,
  DependencyType,
  Task,
  TaskPriority,
  UpdateTaskInput,
} from "../../types";

interface Props {
  projectId: string;
  tasks: Task[];
  open: boolean;
  /** 非空 = 编辑模式 */
  initial?: Task | null;
  onClose: () => void;
}

interface FormState {
  title: string;
  parentId: string;
  predecessorId: string;
  notes: string;
  priority: TaskPriority;
  estimateHours: string;
  durationDays: string;
  startDate: string;
  endDate: string;
  assignee: string;
}

const EMPTY: FormState = {
  title: "",
  parentId: "",
  predecessorId: "",
  notes: "",
  priority: "medium",
  estimateHours: "",
  durationDays: "1",
  startDate: "",
  endDate: "",
  assignee: "",
};

export function TaskFormDialog({ projectId, tasks, open, initial, onClose }: Props) {
  const isEdit = initial !== null;
  const [form, setForm] = useState<FormState>(EMPTY);
  const create = useCreateTask(projectId);
  const update = useUpdateTask(projectId);
  const createDep = useCreateDependency(projectId);

  useEffect(() => {
    if (!open) return;
    if (initial) {
      setForm({
        title: initial.title,
        parentId: initial.parentId ?? "",
        predecessorId: "",
        notes: initial.notes ?? "",
        priority: initial.priority,
        estimateHours: initial.estimateHours > 0 ? String(initial.estimateHours) : "",
        durationDays: String(initial.durationDays),
        startDate: initial.startDate ?? "",
        endDate: initial.endDate ?? "",
        assignee: initial.assignee ?? "",
      });
    } else {
      setForm(EMPTY);
    }
  }, [open, initial]);

  const set = (patch: Partial<FormState>) => setForm((f) => ({ ...f, ...patch }));

  const submit = () => {
    if (!form.title.trim()) {
      toastError("请填写任务标题");
      return;
    }
    const hours = form.estimateHours ? Number(form.estimateHours) : 0;
    const duration = form.durationDays ? Number(form.durationDays) : 0;
    if (isEdit && initial) {
      const payload: UpdateTaskInput = {
        id: initial.id,
        title: form.title,
        notes: form.notes || null,
        priority: form.priority,
        estimateHours: hours,
        durationDays: duration,
        startDate: form.startDate || null,
        endDate: form.endDate || null,
        assignee: form.assignee || null,
      };
      update.mutate(payload, {
        onSuccess: () => {
          toast("任务已更新");
          onClose();
        },
      });
    } else {
      const payload: CreateTaskInput = {
        projectId,
        parentId: form.parentId || null,
        title: form.title,
        notes: form.notes || null,
        priority: form.priority,
        estimateHours: hours,
        durationDays: form.durationDays ? Number(form.durationDays) : 1,
        startDate: form.startDate || null,
        endDate: form.endDate || null,
        assignee: form.assignee || null,
      };
      create.mutate(payload, {
        onSuccess: (task) => {
          // 前置任务（S2 仅支持创建时指定 FS 依赖，完整依赖编辑 S3 实现）
          if (form.predecessorId) {
            createDep.mutate({
              projectId,
              predecessorId: form.predecessorId,
              successorId: task.id,
              depType: "fs" satisfies DependencyType,
              lagDays: 0,
            });
          }
          toast("任务已创建");
          onClose();
        },
      });
    }
  };

  // 编辑模式不修改父级与依赖（WBS 重排在列表行内完成，依赖编辑 S3 实现）
  const parentOptions = isEdit ? [] : tasks;
  const predecessorOptions = isEdit ? [] : tasks;

  return (
    <Dialog
      open={open}
      title={isEdit ? `编辑任务（${initial?.wbsCode ?? ""}）` : "新建任务"}
      onClose={onClose}
      className="max-w-xl"
    >
      <form
        className="space-y-3"
        onSubmit={(e) => {
          e.preventDefault();
          submit();
        }}
      >
        <div>
          <Label required>任务标题</Label>
          <Input
            value={form.title}
            onChange={(e) => set({ title: e.target.value })}
            placeholder="例如：完成登录模块联调"
            autoFocus
          />
        </div>

        {!isEdit && (
          <div className="grid grid-cols-2 gap-3">
            <div>
              <Label>父任务（决定 WBS 层级）</Label>
              <Select value={form.parentId} onChange={(e) => set({ parentId: e.target.value })}>
                <option value="">— 顶层工作包 —</option>
                {parentOptions.map((t) => (
                  <option key={t.id} value={t.id}>
                    {t.wbsCode} {t.title}
                  </option>
                ))}
              </Select>
            </div>
            <div>
              <Label>前置任务（完成后开始 · FS）</Label>
              <Select
                value={form.predecessorId}
                onChange={(e) => set({ predecessorId: e.target.value })}
              >
                <option value="">— 无 —</option>
                {predecessorOptions.map((t) => (
                  <option key={t.id} value={t.id}>
                    {t.wbsCode} {t.title}
                  </option>
                ))}
              </Select>
            </div>
          </div>
        )}

        <div className="grid grid-cols-2 gap-3">
          <div>
            <Label>优先级</Label>
            <Select
              value={form.priority}
              onChange={(e) => set({ priority: e.target.value as TaskPriority })}
            >
              {(Object.keys(TASK_PRIORITY_LABEL) as TaskPriority[]).map((p) => (
                <option key={p} value={p}>
                  {TASK_PRIORITY_LABEL[p]}
                </option>
              ))}
            </Select>
          </div>
          <div>
            <Label>工期（天）</Label>
            <Input
              type="number"
              min={0}
              step={0.5}
              value={form.durationDays}
              onChange={(e) => set({ durationDays: e.target.value })}
            />
          </div>
          <div>
            <Label>预估工时（小时）</Label>
            <Input
              type="number"
              min={0}
              step={0.5}
              value={form.estimateHours}
              onChange={(e) => set({ estimateHours: e.target.value })}
            />
          </div>
          <div>
            <Label>负责人</Label>
            <Input
              value={form.assignee}
              onChange={(e) => set({ assignee: e.target.value })}
              placeholder="成员名（S3 接入成员表）"
            />
          </div>
          <div>
            <Label>开始日期</Label>
            <Input
              type="date"
              value={form.startDate}
              onChange={(e) => set({ startDate: e.target.value })}
            />
          </div>
          <div>
            <Label>截止日期</Label>
            <Input
              type="date"
              value={form.endDate}
              onChange={(e) => set({ endDate: e.target.value })}
            />
          </div>
        </div>

        <div>
          <Label>备注</Label>
          <Textarea
            value={form.notes}
            onChange={(e) => set({ notes: e.target.value })}
            placeholder="验收标准、依赖说明等"
          />
        </div>

        <div className="flex justify-end gap-2 pt-1">
          <Button type="button" variant="secondary" onClick={onClose}>
            取消
          </Button>
          <Button type="submit" disabled={create.isPending || update.isPending}>
            {isEdit ? "保存修改" : "创建任务"}
          </Button>
        </div>
      </form>
    </Dialog>
  );
}
