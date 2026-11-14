import { useState } from "react";
import { Link, useParams } from "react-router";
import { ChevronLeft } from "lucide-react";

import { Badge } from "../components/ui/Badge";
import { cn, formatBudget, formatDate, statusTone } from "../lib/format";
import { useProject, useTasks } from "../hooks/useProjects";
import { ClosingTab } from "./project/ClosingTab";
import { ExecutionTab } from "./project/ExecutionTab";
import { InitiationTab } from "./project/InitiationTab";
import { MonitoringTab } from "./project/MonitoringTab";
import { OverviewTab } from "./project/OverviewTab";
import { PlanningTab } from "./project/PlanningTab";
import { PROJECT_METHOD_LABEL, PROJECT_STATUS_LABEL } from "../types";

/** 主导航 = PMBOK 8 五个焦点领域 + 总览（见 docs/02-产品功能设计.md 第 3 章） */
const TABS = [
  { key: "overview", label: "总览", hint: "项目信息与进展" },
  { key: "initiation", label: "启动", hint: "PMBOK 8 焦点领域：章程 / 干系人" },
  { key: "planning", label: "规划", hint: "PMBOK 8 焦点领域：WBS / 进度 / 预算 / 风险计划" },
  { key: "execution", label: "执行", hint: "PMBOK 8 焦点领域：任务流转 / 工时 / 会议" },
  { key: "monitoring", label: "监控", hint: "PMBOK 8 焦点领域：绩效 / 变更控制" },
  { key: "closing", label: "收尾", hint: "PMBOK 8 焦点领域：验收 / 经验教训" },
] as const;

type TabKey = (typeof TABS)[number]["key"];

export function ProjectDetailPage() {
  const { id = "" } = useParams();
  const project = useProject(id);
  const tasks = useTasks(id);
  const [tab, setTab] = useState<TabKey>("overview");

  if (project.isLoading) {
    return <div className="p-8 text-sm text-slate-500">加载中…</div>;
  }
  if (project.isError) {
    return <div className="p-8 text-sm text-red-600">项目加载失败：{project.error.message}</div>;
  }
  const p = project.data;
  if (!p) {
    return (
      <div className="p-8 text-sm text-slate-500">
        项目不存在或已被删除。<Link to="/projects" className="text-blue-600 hover:underline">返回列表</Link>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-6xl px-8 py-6">
      <Link
        to="/projects"
        className="inline-flex items-center gap-1 text-xs text-slate-500 hover:text-slate-700"
      >
        <ChevronLeft size={14} /> 返回项目列表
      </Link>

      <header className="mt-3">
        <h1 className="text-xl font-bold text-slate-900">{p.name}</h1>
        <div className="mt-2 flex flex-wrap items-center gap-2 text-xs text-slate-500">
          <Badge tone="blue">{PROJECT_METHOD_LABEL[p.method]}</Badge>
          <Badge tone={statusTone(p.status)}>{PROJECT_STATUS_LABEL[p.status]}</Badge>
          {p.code && <span className="font-mono">{p.code}</span>}
          <span>
            {formatDate(p.startDate)} → {formatDate(p.targetEndDate)}
          </span>
          <span>{formatBudget(p.bac)}</span>
        </div>
        {p.description && <p className="mt-2 max-w-3xl text-sm text-slate-600">{p.description}</p>}
      </header>

      <div className="mt-5 flex gap-1 border-b border-slate-200">
        {TABS.map((t) => (
          <button
            key={t.key}
            title={t.hint}
            onClick={() => setTab(t.key)}
            className={cn(
              "-mb-px rounded-t-lg border-b-2 px-4 py-2 text-sm font-medium transition-colors",
              tab === t.key
                ? "border-blue-600 text-blue-700"
                : "border-transparent text-slate-500 hover:text-slate-700",
            )}
          >
            {t.label}
          </button>
        ))}
      </div>

      <div className="mt-4 space-y-4 pb-10">
        {tab === "overview" && (
          <OverviewTab project={p} tasks={tasks.data ?? []} tasksLoading={tasks.isLoading} />
        )}
        {tab === "initiation" && <InitiationTab project={p} />}
        {tab === "planning" && (
          <PlanningTab projectId={p.id} tasks={tasks.data ?? []} tasksLoading={tasks.isLoading} />
        )}
        {tab === "execution" && <ExecutionTab projectId={p.id} tasks={tasks.data ?? []} />}
        {tab === "monitoring" && <MonitoringTab tasks={tasks.data ?? []} />}
        {tab === "closing" && <ClosingTab project={p} />}
      </div>
    </div>
  );
}
