import { useEffect, useRef, useState } from "react";
import Gantt from "frappe-gantt";
import "frappe-gantt/dist/frappe-gantt.css";

import { EmptyState } from "../ui/EmptyState";
import { cn } from "../../lib/format";
import { useSchedule } from "../../hooks/useProjects";

const VIEW_MODES = ["Day", "Week", "Month"] as const;
type ViewMode = (typeof VIEW_MODES)[number];

const VIEW_LABEL: Record<ViewMode, string> = {
  Day: "日",
  Week: "周",
  Month: "月",
};

interface Props {
  projectId: string;
}

export function GanttView({ projectId }: Props) {
  const schedule = useSchedule(projectId);
  const [mode, setMode] = useState<ViewMode>("Week");
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const el = containerRef.current;
    const data = schedule.data;
    if (!el || !data || data.tasks.length === 0) return;

    el.innerHTML = "";
    const ganttTasks = data.tasks.map((t) => ({
      id: t.taskId,
      name: `${t.wbsCode ?? ""} ${t.isMilestone ? "◆ " : ""}${t.title}`.trim(),
      start: t.startDate,
      end: t.endDate,
      progress: 0,
      dependencies: t.predecessorIds.join(", ") || undefined,
      custom_class: t.critical ? "bar-critical" : "bar-float",
    }));
    new Gantt(el, ganttTasks, {
      view_mode: mode,
      date_format: "YYYY-MM-DD",
      readonly: true,
    });
    return () => {
      el.innerHTML = "";
    };
  }, [schedule.data, mode]);

  if (schedule.isLoading) {
    return <div className="py-6 text-center text-xs text-slate-400">排程计算中…</div>;
  }
  if (schedule.isError) {
    return (
      <div className="rounded-lg border border-red-100 bg-red-50 p-3 text-xs text-red-600">
        排程计算失败：{schedule.error.message}
      </div>
    );
  }
  const s = schedule.data;
  if (!s || s.tasks.length === 0) {
    return (
      <EmptyState
        title="暂无可排程的任务"
        description="先在任务列表中创建任务并设置工期，甘特图将按依赖关系自动排布。"
      />
    );
  }

  const criticalCount = s.tasks.filter((t) => t.critical).length;

  return (
    <div>
      <div className="mb-3 flex flex-wrap items-center gap-2 text-xs text-slate-500">
        <span className="rounded bg-slate-100 px-2 py-1">
          锚点日 <span className="font-mono">{s.anchorDate}</span>
        </span>
        <span className="rounded bg-slate-100 px-2 py-1">
          总工期 <span className="font-semibold">{s.durationDays}</span> 天
        </span>
        <span className="flex items-center gap-1.5 rounded bg-slate-100 px-2 py-1">
          <span className="h-2.5 w-4 rounded bg-rose-600" /> 关键路径 {criticalCount} 个任务
        </span>
        <div className="ml-auto flex items-center gap-1">
          {VIEW_MODES.map((m) => (
            <button
              key={m}
              onClick={() => setMode(m)}
              className={cn(
                "rounded px-2 py-1 font-medium transition-colors",
                mode === m
                  ? "bg-blue-600 text-white"
                  : "bg-slate-100 text-slate-600 hover:bg-slate-200",
              )}
            >
              {VIEW_LABEL[m]}
            </button>
          ))}
        </div>
      </div>
      <div className="overflow-x-auto rounded-lg border border-slate-200 bg-white p-2">
        <div ref={containerRef} />
      </div>
      <p className="mt-2 text-[11px] text-slate-400">
        排程 = 锚点日 + 日历天偏移（暂不含工作日历）；拖动已禁用，改期请编辑任务工期与依赖。
      </p>
    </div>
  );
}
