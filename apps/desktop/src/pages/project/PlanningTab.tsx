import { useState } from "react";

import { GanttView } from "../../components/tasks/GanttView";
import { TaskList } from "../../components/tasks/TaskList";
import { RiskSection } from "../../components/risks/RiskSection";
import { Card } from "../../components/ui/Card";
import { cn } from "../../lib/format";
import type { Task } from "../../types";

interface Props {
  projectId: string;
  tasks: Task[];
  tasksLoading: boolean;
}

/** 规划页签 = PMBOK 8「规划」焦点领域：任务/WBS + 甘特图（CPM） + 风险登记册 */
export function PlanningTab({ projectId, tasks, tasksLoading }: Props) {
  const [view, setView] = useState<"list" | "gantt">("list");

  return (
    <div className="space-y-4">
      <Card
        title="任务与 WBS"
        subtitle="对应 PMBOK 8「规划」焦点领域 · WBS 编码自动维护（1 / 1.1 / 1.2.3）· 甘特图含关键路径"
        actions={
          <div className="flex items-center gap-1">
            {(
              [
                { key: "list", label: "列表" },
                { key: "gantt", label: "甘特图" },
              ] as const
            ).map((v) => (
              <button
                key={v.key}
                onClick={() => setView(v.key)}
                className={cn(
                  "rounded-md px-2.5 py-1 text-xs font-medium transition-colors",
                  view === v.key
                    ? "bg-blue-600 text-white"
                    : "bg-slate-100 text-slate-600 hover:bg-slate-200",
                )}
              >
                {v.label}
              </button>
            ))}
          </div>
        }
      >
        {view === "list" ? (
          <TaskList projectId={projectId} tasks={tasks} loading={tasksLoading} />
        ) : (
          <GanttView projectId={projectId} />
        )}
      </Card>

      <RiskSection projectId={projectId} />
    </div>
  );
}
