import { useState } from "react";
import type { ReactNode } from "react";
import {
  ChevronDown,
  ChevronUp,
  ClipboardList,
  IndentDecrease,
  IndentIncrease,
  Plus,
  Trash2,
} from "lucide-react";

import { Badge } from "../ui/Badge";
import { Button } from "../ui/Button";
import { EmptyState } from "../ui/EmptyState";
import { Select } from "../ui/Input";
import {
  useDeleteTask,
  useMoveTask,
  useReorderTask,
  useUpdateTask,
} from "../../hooks/useProjects";
import { cn, formatDate, priorityTone } from "../../lib/format";
import { TASK_PRIORITY_LABEL, TASK_STATUS_LABEL } from "../../types";
import type { Task, TaskMoveDirection, TaskStatus } from "../../types";
import { TaskFormDialog } from "./TaskFormDialog";

interface Props {
  projectId: string;
  tasks: Task[];
  loading?: boolean;
}

function RowBtn({
  title,
  onClick,
  disabled,
  children,
}: {
  title: string;
  onClick: () => void;
  disabled?: boolean;
  children: ReactNode;
}) {
  return (
    <button
      title={title}
      disabled={disabled}
      onClick={onClick}
      className="rounded p-0.5 text-slate-300 hover:bg-slate-100 hover:text-slate-600 disabled:opacity-30"
    >
      {children}
    </button>
  );
}

export function TaskList({ projectId, tasks, loading }: Props) {
  const [creating, setCreating] = useState(false);
  const [editing, setEditing] = useState<Task | null>(null);
  const update = useUpdateTask(projectId);
  const remove = useDeleteTask(projectId);
  const move = useMoveTask(projectId);
  const reorder = useReorderTask(projectId);

  const depthOf = (t: Task) => (t.wbsCode ? t.wbsCode.split(".").length - 1 : 0);
  const siblingsOf = (t: Task) => tasks.filter((x) => x.parentId === t.parentId);

  const reorderTask = (t: Task, direction: TaskMoveDirection) =>
    reorder.mutate({ taskId: t.id, direction });

  /** 缩进：变为同级上一兄弟的子任务（经典 WBS 编辑行为） */
  const indentTask = (t: Task) => {
    const siblings = siblingsOf(t);
    const idx = siblings.findIndex((x) => x.id === t.id);
    if (idx <= 0) return;
    move.mutate({ taskId: t.id, newParentId: siblings[idx - 1].id });
  };

  /** 升级：挂到父级的父级之下（顶层任务的子任务升到顶层） */
  const outdentTask = (t: Task) => {
    if (!t.parentId) return;
    const parent = tasks.find((x) => x.id === t.parentId);
    move.mutate({ taskId: t.id, newParentId: parent?.parentId ?? null });
  };

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
        {tasks.map((t) => {
          const siblings = siblingsOf(t);
          const idx = siblings.findIndex((x) => x.id === t.id);
          return (
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
              <span
                className="w-12 shrink-0 text-right text-xs text-slate-400"
                title="工期（天）"
              >
                {t.durationDays}d
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
              <div className="flex shrink-0 items-center gap-0.5 opacity-0 transition group-hover:opacity-100">
                <RowBtn title="上移" onClick={() => reorderTask(t, "up")} disabled={idx <= 0}>
                  <ChevronUp size={13} />
                </RowBtn>
                <RowBtn
                  title="下移"
                  onClick={() => reorderTask(t, "down")}
                  disabled={idx < 0 || idx >= siblings.length - 1}
                >
                  <ChevronDown size={13} />
                </RowBtn>
                <RowBtn
                  title="缩进（变为上一兄弟的子任务）"
                  onClick={() => indentTask(t)}
                  disabled={idx <= 0}
                >
                  <IndentIncrease size={13} />
                </RowBtn>
                <RowBtn title="升级（回到父级层级）" onClick={() => outdentTask(t)} disabled={!t.parentId}>
                  <IndentDecrease size={13} />
                </RowBtn>
                <RowBtn
                  title="删除任务（子任务将一并删除）"
                  onClick={() => {
                    if (window.confirm(`删除任务「${t.title}」？其子任务将一并删除。`)) {
                      remove.mutate(t.id);
                    }
                  }}
                >
                  <Trash2 size={14} />
                </RowBtn>
              </div>
            </div>
          );
        })}
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
