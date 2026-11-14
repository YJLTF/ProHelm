import { useState } from "react";
import { ClipboardList, Plus, Trash2 } from "lucide-react";

import { Badge } from "../ui/Badge";
import { Button } from "../ui/Button";
import { EmptyState } from "../ui/EmptyState";
import { Select } from "../ui/Input";
import { useDeleteTask, useUpdateTask } from "../../hooks/useProjects";
import { cn, formatDate, priorityTone } from "../../lib/format";
import { TASK_PRIORITY_LABEL, TASK_STATUS_LABEL } from "../../types";
import type { Task, TaskStatus } from "../../types";
import { TaskFormDialog } from "./TaskFormDialog";

interface Props {
  projectId: string;
  tasks: Task[];
  loading?: boolean;
}

export function TaskList({ projectId, tasks, loading }: Props) {
  const [creating, setCreating] = useState(false);
  const [editing, setEditing] = useState<Task | null>(null);
  const update = useUpdateTask(projectId);
  const remove = useDeleteTask(projectId);

  const depthOf = (t: Task) => (t.wbsCode ? t.wbsCode.split(".").length - 1 : 0);

  if (loading) {
    return <div className="py-6 text-center text-xs text-slate-400">任务加载中…</div>;
  }

  if (tasks.length === 0 && !creating) {
    return (
      <EmptyState
        icon={<ClipboardList size={28} />}
        title="还没有任务"
        description="从 WBS 分解开始：先建顶层工作包（1、2、3…），再逐层拆解子任务（1.1、1.2…）。"
        action={
          <Button size="sm" onClick={() => setCreating(true)}>
            <Plus size={14} /> 新建任务
          </Button>
        }
      />
    );
  }

  return (
    <div>
      <div className="mb-3 flex justify-end">
        <Button size="sm" onClick={() => setCreating(true)}>
          <Plus size={14} /> 新建任务
        </Button>
      </div>

      <div className="space-y-1.5">
        {tasks.map((t) => (
          <div
            key={t.id}
            className={cn(
              "group flex items-center gap-2 rounded-lg border border-slate-100 bg-white px-3 py-2 hover:border-slate-200",
            )}
            style={{ marginLeft: depthOf(t) * 20 }}
          >
            <span className="w-12 shrink-0 font-mono text-xs text-slate-400">{t.wbsCode}</span>
            <button
              className="min-w-0 flex-1 truncate text-left text-sm text-slate-800 hover:text-blue-700"
              title="点击编辑"
              onClick={() => setEditing(t)}
            >
              {t.title}
            </button>
            <Badge tone={priorityTone(t.priority)}>{TASK_PRIORITY_LABEL[t.priority]}</Badge>
            {t.assignee && <span className="shrink-0 text-xs text-slate-500">{t.assignee}</span>}
            <span className="w-20 shrink-0 text-right text-xs text-slate-400">
              {formatDate(t.endDate)}
            </span>
            <span className="w-12 shrink-0 text-right text-xs text-slate-400">
              {t.estimateHours > 0 ? `${t.estimateHours}h` : ""}
            </span>
            <div className="w-24 shrink-0">
              <Select
                className="px-2 py-1 text-xs"
                value={t.status}
                onChange={(e) =>
                  update.mutate({ id: t.id, status: e.target.value as TaskStatus })
                }
              >
                {(Object.keys(TASK_STATUS_LABEL) as TaskStatus[]).map((s) => (
                  <option key={s} value={s}>
                    {TASK_STATUS_LABEL[s]}
                  </option>
                ))}
              </Select>
            </div>
            <button
              title="删除任务（子任务将一并删除）"
              className="text-slate-300 opacity-0 transition hover:text-red-500 group-hover:opacity-100"
              onClick={() => {
                if (window.confirm(`删除任务「${t.title}」？其子任务将一并删除。`)) {
                  remove.mutate(t.id);
                }
              }}
            >
              <Trash2 size={14} />
            </button>
          </div>
        ))}
      </div>

      <TaskFormDialog
        projectId={projectId}
        tasks={tasks}
        open={creating || editing !== null}
        initial={editing}
        onClose={() => {
          setCreating(false);
          setEditing(null);
        }}
      />
    </div>
  );
}
