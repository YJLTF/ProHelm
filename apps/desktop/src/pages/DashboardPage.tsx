import { Link } from "react-router";
import type { LucideIcon } from "lucide-react";
import {
  AlertTriangle,
  CalendarClock,
  Compass,
  FolderKanban,
  Rocket,
  TrendingUp,
} from "lucide-react";

import { ProjectCard } from "../components/projects/ProjectCard";
import { EmptyState } from "../components/ui/EmptyState";
import { Button } from "../components/ui/Button";
import { useDashboardStats, useProjects } from "../hooks/useProjects";
import type { DashboardStats } from "../types";

interface StatDef {
  key: keyof DashboardStats;
  label: string;
  hint: string;
  icon: LucideIcon;
  tone: string;
}

const statDefs: StatDef[] = [
  { key: "projectCount", label: "项目总数", hint: "全部项目", icon: FolderKanban, tone: "text-slate-800" },
  { key: "activeProjects", label: "进行中", hint: "状态为进行中的项目", icon: Rocket, tone: "text-emerald-600" },
  { key: "overdueTasks", label: "逾期任务", hint: "已过截止日且未完成", icon: AlertTriangle, tone: "text-red-600" },
  { key: "dueSoonTasks", label: "七天内到期", hint: "即将到期的任务", icon: CalendarClock, tone: "text-amber-600" },
];

export function DashboardPage() {
  const stats = useDashboardStats();
  const projects = useProjects();

  if (stats.isLoading || projects.isLoading) {
    return <div className="p-8 text-sm text-slate-500">加载中…</div>;
  }
  if (stats.isError) {
    return <div className="p-8 text-sm text-red-600">驾驶舱数据加载失败：{stats.error.message}</div>;
  }

  const s = stats.data!;
  const projectList = projects.data ?? [];
  const activeProjects = projectList.filter((p) => p.status === "active");

  return (
    <div className="mx-auto max-w-6xl px-8 py-6">
      <header className="mb-5 flex items-center gap-2">
        <Compass size={18} className="text-blue-600" />
        <h1 className="text-lg font-bold text-slate-900">驾驶舱</h1>
        <span className="text-xs text-slate-400">跨项目健康度总览 · 数据实时来自本地</span>
      </header>

      <div className="grid grid-cols-4 gap-3">
        {statDefs.map(({ key, label, hint, icon: Icon, tone }) => (
          <div key={key} className="rounded-xl border border-slate-200 bg-white p-4 shadow-sm">
            <div className="flex items-center justify-between">
              <span className="text-xs font-medium text-slate-500">{label}</span>
              <Icon size={15} className="text-slate-300" />
            </div>
            <div className={`mt-2 text-2xl font-bold ${tone}`}>{s[key]}</div>
            <div className="mt-1 text-[11px] text-slate-400">{hint}</div>
          </div>
        ))}
      </div>

      <div className="mt-6 mb-2 flex items-center gap-2">
        <TrendingUp size={15} className="text-slate-400" />
        <h2 className="text-sm font-semibold text-slate-700">进行中的项目</h2>
        <span className="text-xs text-slate-400">{activeProjects.length} 个</span>
      </div>

      {projectList.length === 0 ? (
        <EmptyState
          icon={<FolderKanban size={32} />}
          title="还没有项目"
          description="创建第一个项目，ProHelm 将按 PMBOK 焦点领域引导你完成启动、规划、执行、监控与收尾。"
          action={
            <Link to="/projects">
              <Button size="sm">前往项目页创建</Button>
            </Link>
          }
        />
      ) : activeProjects.length === 0 ? (
        <EmptyState
          title="没有进行中的项目"
          description="所有项目都处于规划中 / 已暂停 / 已关闭状态。创建任务会自动激活规划中的项目。"
        />
      ) : (
        <div className="grid grid-cols-3 gap-3">
          {activeProjects.map((p) => (
            <ProjectCard key={p.id} project={p} />
          ))}
        </div>
      )}
    </div>
  );
}
