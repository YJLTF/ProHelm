import { useState } from "react";
import { Bot } from "lucide-react";

import { WbsDraftDialog } from "../../components/ai/WbsDraftDialog";
import { GanttView } from "../../components/tasks/GanttView";
import { TaskList } from "../../components/tasks/TaskList";
import { RiskSection } from "../../components/risks/RiskSection";
import { Card } from "../../components/ui/Card";
import { cn } from "../../lib/format";
import type { Project, Task } from "../../types";

interface Props {
  project: Project;
  tasks: Task[];
  tasksLoading: boolean;
}

/** 规划页签 = PMBOK 8「规划」焦点领域：任务/WBS + 甘特图（CPM） + 风险登记册 */
export function PlanningTab({ project, tasks, tasksLoading }: Props) {
  const [view, setView] = useState<"list" | "gantt">("list");
  const [aiOpen, setAiOpen] = useState(false);

  return (
    <div className="space-y-4">
      <Card
        title="任务与 WBS"
        subtitle="对应 PMBOK 8「规划」焦点领域 · WBS 编码自动维护（1 / 1.1 / 1.2.3）· 甘特图含关键路径"
        actions={
          <div className="flex items-center gap-1.5">
            <button
              onClick={() => setAiOpen(true)}
              title="AI 基于需求描述生成 WBS 草稿（草稿→预览→采纳）"
              className="flex items-center gap-1 rounded-md bg-slate-800 px-2.5 py-1 text-xs font-medium text-white hover:bg-slate-700"
            >
              <Bot size={13} /> AI 拆解
            </button>
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
          <TaskList projectId={project.id} tasks={tasks} loading={tasksLoading} />
        ) : (
          <GanttView projectId={project.id} />
        )}
      </Card>

      <RiskSection projectId={project.id} />

      <WbsDraftDialog
        projectId={project.id}
        open={aiOpen}
        defaultRequirements={project.description ?? ""}
        onClose={() => setAiOpen(false)}
      />
    </div>
  );
}
