import { Link } from "react-router";
import { useQuery } from "@tanstack/react-query";
import type { LucideIcon } from "lucide-react";
import {
  AlertTriangle,
  CalendarClock,
  Compass,
  FolderKanban,
  Rocket,
  TrendingUp,
  TriangleAlert,
  Users,
} from "lucide-react";

import { ProjectCard } from "../components/projects/ProjectCard";
import { Card } from "../components/ui/Card";
import { EmptyState } from "../components/ui/EmptyState";
import { Badge } from "../components/ui/Badge";
import { Button } from "../components/ui/Button";
import { api } from "../lib/ipc";
import { useDashboardStats, useProjects } from "../hooks/useProjects";
import { riskScoreTone } from "../types";
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
  const radar = useQuery({ queryKey: ["risk-radar"], queryFn: api.riskRadar });
  const workload = useQuery({ queryKey: ["workload"], queryFn: api.workload });

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

      <div className="mt-6 grid grid-cols-2 gap-3">
        <Card
          title={
            <span className="flex items-center gap-1.5">
              <TriangleAlert size={13} className="text-rose-500" /> 风险雷达（跨项目）
            </span>
          }
          subtitle="未关闭风险按评分降序 · 对应 PMBOK 8 风险绩效域"
        >
          {radar.isLoading ? (
            <div className="py-2 text-center text-xs text-slate-400">加载中…</div>
          ) : (radar.data?.length ?? 0) === 0 ? (
            <div className="py-2 text-center text-xs text-slate-400">没有未关闭的风险</div>
          ) : (
            <div className="divide-y divide-slate-50">
              {radar.data!.map((r) => (
                <Link
                  key={r.riskId}
                  to={`/projects/${r.projectId}`}
                  className="flex items-center gap-2 py-2 text-sm hover:text-blue-700"
                >
                  <Badge tone={riskScoreTone(r.probability * r.impact)}>
                    {r.probability * r.impact}
                  </Badge>
                  <span className="min-w-0 flex-1 truncate text-slate-700">
                    <span className="text-xs text-slate-400">R{r.riskNo} {r.projectName} · </span>
                    {r.title}
                  </span>
                  {r.owner && <span className="shrink-0 text-xs text-slate-500">{r.owner}</span>}
                  {r.status === "occurred" && <Badge tone="red">已发生</Badge>}
                </Link>
              ))}
            </div>
          )}
        </Card>

        <Card
          title={
            <span className="flex items-center gap-1.5">
              <Users size={13} className="text-blue-500" /> 人员负荷
            </span>
          }
          subtitle="按负责人聚合 · 逾期任务标红"
        >
          {workload.isLoading ? (
            <div className="py-2 text-center text-xs text-slate-400">加载中…</div>
          ) : (workload.data?.length ?? 0) === 0 ? (
            <div className="py-2 text-center text-xs text-slate-400">
              还没有分配负责人（在任务表单中填写）
            </div>
          ) : (
            <div className="divide-y divide-slate-50">
              {workload.data!.map((w) => (
                <div key={w.assignee} className="flex items-center gap-2 py-2 text-sm">
                  <span className="min-w-0 flex-1 truncate text-slate-700">{w.assignee}</span>
                  <span className="text-xs text-slate-400">
                    {w.projectCount} 项目 · {w.taskCount} 任务 · {w.estimateHours}h
                  </span>
                  {w.overdue > 0 && <Badge tone="red">逾期 {w.overdue}</Badge>}
                </div>
              ))}
            </div>
          )}
        </Card>
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
