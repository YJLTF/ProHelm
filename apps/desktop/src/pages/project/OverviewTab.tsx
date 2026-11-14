import { CheckCircle2, Circle, CircleDashed } from "lucide-react";

import { Badge } from "../../components/ui/Badge";
import { Card } from "../../components/ui/Card";
import { formatDate } from "../../lib/format";
import { TASK_STATUS_LABEL } from "../../types";
import type { Project, Task } from "../../types";

interface Props {
  project: Project;
  tasks: Task[];
  tasksLoading: boolean;
}

export function OverviewTab({ project, tasks, tasksLoading }: Props) {
  const done = tasks.filter((t) => t.status === "done").length;
  const inProgress = tasks.filter((t) => t.status === "in_progress").length;
  const pct = tasks.length > 0 ? Math.round((done / tasks.length) * 100) : 0;

  const upcoming = tasks
    .filter((t) => t.status !== "done" && t.endDate)
    .sort((a, b) => (a.endDate! < b.endDate! ? -1 : 1))
    .slice(0, 5);

  return (
    <div className="grid grid-cols-2 gap-4">
      <Card title="任务进展" subtitle={tasksLoading ? "统计中…" : `共 ${tasks.length} 个任务`}>
        <div className="mb-2 flex items-center gap-2">
          <div className="h-2.5 flex-1 overflow-hidden rounded-full bg-slate-100">
            <div
              className="h-full rounded-full bg-blue-600 transition-all"
              style={{ width: `${pct}%` }}
            />
          </div>
          <span className="text-sm font-semibold text-slate-700">{pct}%</span>
        </div>
        <div className="flex gap-2 text-xs text-slate-500">
          <Badge>
            {TASK_STATUS_LABEL.todo} {tasks.length - done - inProgress}
          </Badge>
          <Badge tone="blue">
            {TASK_STATUS_LABEL.in_progress} {inProgress}
          </Badge>
          <Badge tone="green">
            {TASK_STATUS_LABEL.done} {done}
          </Badge>
        </div>
      </Card>

      <Card title="关键信息">
        <dl className="grid grid-cols-2 gap-x-4 gap-y-2 text-sm">
          <div>
            <dt className="text-xs text-slate-400">开始日期</dt>
            <dd className="text-slate-700">{formatDate(project.startDate)}</dd>
          </div>
          <div>
            <dt className="text-xs text-slate-400">目标完成</dt>
            <dd className="text-slate-700">{formatDate(project.targetEndDate)}</dd>
          </div>
          <div>
            <dt className="text-xs text-slate-400">总预算（BAC）</dt>
            <dd className="text-slate-700">{project.bac > 0 ? `¥${project.bac.toLocaleString("zh-CN")}` : "未设置"}</dd>
          </div>
          <div>
            <dt className="text-xs text-slate-400">已产生实际成本</dt>
            <dd className="text-slate-700">M2 接入工时后计算（AC）</dd>
          </div>
        </dl>
      </Card>

      <Card title="近期到期任务" className="col-span-2">
        {upcoming.length === 0 ? (
          <div className="py-3 text-center text-xs text-slate-400">没有设置了截止日期的未完成任务</div>
        ) : (
          <div className="divide-y divide-slate-50">
            {upcoming.map((t) => (
              <div key={t.id} className="flex items-center gap-3 py-2 text-sm">
                {t.status === "in_progress" ? (
                  <CircleDashed size={14} className="shrink-0 text-blue-500" />
                ) : (
                  <Circle size={14} className="shrink-0 text-slate-300" />
                )}
                <span className="font-mono text-xs text-slate-400">{t.wbsCode}</span>
                <span className="min-w-0 flex-1 truncate text-slate-700">{t.title}</span>
                {t.assignee && <span className="text-xs text-slate-500">{t.assignee}</span>}
                <span className="text-xs text-slate-400">{formatDate(t.endDate)}</span>
                <CheckCircle2 size={13} className="text-slate-200" />
              </div>
            ))}
          </div>
        )}
      </Card>
    </div>
  );
}
