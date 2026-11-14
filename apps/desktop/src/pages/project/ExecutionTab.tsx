import { ArrowLeft, ArrowRight } from "lucide-react";

import { Badge } from "../../components/ui/Badge";
import { Card } from "../../components/ui/Card";
import { EmptyState } from "../../components/ui/EmptyState";
import { useUpdateTask } from "../../hooks/useProjects";
import { cn, formatDate, priorityTone } from "../../lib/format";
import { TASK_PRIORITY_LABEL, TASK_STATUS_LABEL } from "../../types";
import type { Task, TaskStatus } from "../../types";

interface Props {
  projectId: string;
  tasks: Task[];
}

const FLOW: TaskStatus[] = ["todo", "in_progress", "done"];

const COLUMNS: Array<{ key: TaskStatus; label: string; accent: string }> = [
  { key: "todo", label: TASK_STATUS_LABEL.todo, accent: "bg-slate-400" },
  { key: "in_progress", label: TASK_STATUS_LABEL.in_progress, accent: "bg-blue-500" },
  { key: "done", label: TASK_STATUS_LABEL.done, accent: "bg-emerald-500" },
];

export function ExecutionTab({ projectId, tasks }: Props) {
  const update = useUpdateTask(projectId);

  const move = (task: Task, dir: -1 | 1) => {
    const idx = FLOW.indexOf(task.status);
    const next = FLOW[idx + dir];
    if (next) update.mutate({ id: task.id, status: next });
  };

  return (
    <Card
      title="任务看板"
      subtitle="对应 PMBOK 8「执行」焦点领域 · 拖拽流转 S3 接入 dnd-kit，当前用箭头按钮流转"
    >
      <div className="grid grid-cols-3 gap-3">
        {COLUMNS.map((col) => {
          const items = tasks.filter((t) => t.status === col.key);
          return (
            <div key={col.key} className="rounded-xl bg-slate-50 p-2.5">
              <div className="mb-2 flex items-center gap-2 px-1">
                <span className={cn("h-2 w-2 rounded-full", col.accent)} />
                <span className="text-xs font-semibold text-slate-600">{col.label}</span>
                <span className="text-xs text-slate-400">{items.length}</span>
              </div>
              <div className="space-y-2">
                {items.length === 0 ? (
                  <div className="rounded-lg border border-dashed border-slate-200 py-4 text-center text-[11px] text-slate-300">
                    空
                  </div>
                ) : (
                  items.map((t) => {
                    const idx = FLOW.indexOf(t.status);
                    return (
                      <div
                        key={t.id}
                        className="rounded-lg border border-slate-200 bg-white p-2.5 shadow-sm"
                      >
                        <div className="flex items-center gap-1.5">
                          <span className="font-mono text-[10px] text-slate-400">{t.wbsCode}</span>
                          <Badge tone={priorityTone(t.priority)}>
                            {TASK_PRIORITY_LABEL[t.priority]}
                          </Badge>
                        </div>
                        <div className="mt-1 line-clamp-2 text-sm text-slate-700">{t.title}</div>
                        <div className="mt-1.5 flex items-center justify-between text-[11px] text-slate-400">
                          <span>{t.assignee ?? "未分配"}</span>
                          <span>{formatDate(t.endDate)}</span>
                        </div>
                        <div className="mt-2 flex justify-end gap-1">
                          {idx > 0 && (
                            <button
                              title={`移回${COLUMNS[idx - 1].label}`}
                              onClick={() => move(t, -1)}
                              className="rounded p-1 text-slate-400 hover:bg-slate-100 hover:text-slate-600"
                            >
                              <ArrowLeft size={13} />
                            </button>
                          )}
                          {idx < FLOW.length - 1 && (
                            <button
                              title={`推进到${COLUMNS[idx + 1].label}`}
                              onClick={() => move(t, 1)}
                              className="rounded p-1 text-slate-400 hover:bg-slate-100 hover:text-blue-600"
                            >
                              <ArrowRight size={13} />
                            </button>
                          )}
                        </div>
                      </div>
                    );
                  })
                )}
              </div>
            </div>
          );
        })}
      </div>
      {tasks.length === 0 && (
        <div className="mt-3">
          <EmptyState
            title="没有任务"
            description="先在「规划」页签创建任务，再回到这里流转。"
          />
        </div>
      )}
    </Card>
  );
}
