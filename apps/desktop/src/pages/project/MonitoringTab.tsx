import type { ReactNode } from "react";
import { save } from "@tauri-apps/plugin-dialog";
import {
  AlertTriangle,
  CalendarClock,
  CheckCircle2,
  FileDown,
  Flag,
  Table2,
} from "lucide-react";

import { EvmSection } from "../../components/evm/EvmSection";
import { Button } from "../../components/ui/Button";
import { Card } from "../../components/ui/Card";
import { api } from "../../lib/ipc";
import { useBaselines, useCreateBaseline } from "../../hooks/useProjects";
import { toast, toastError } from "../../store/toast";
import type { Project, Task } from "../../types";

interface Props {
  project: Project;
  tasks: Task[];
}

function todayStr(): string {
  const d = new Date();
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

function addDays(iso: string, days: number): string {
  const d = new Date(`${iso}T00:00:00`);
  d.setDate(d.getDate() + days);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

export function MonitoringTab({ project, tasks }: Props) {
  const projectId = project.id;
  const today = todayStr();
  const weekLater = addDays(today, 7);
  const open = tasks.filter((t) => t.status !== "done");
  const overdue = open.filter((t) => t.endDate && t.endDate < today).length;
  const dueSoon = open.filter((t) => t.endDate && t.endDate >= today && t.endDate <= weekLater).length;
  const done = tasks.length - open.length;
  const pct = tasks.length > 0 ? Math.round((done / tasks.length) * 100) : 0;

  const baselines = useBaselines(projectId);
  const createBaseline = useCreateBaseline(projectId);

  return (
    <div className="space-y-4">
      <div className="grid grid-cols-3 gap-3">
        <MetricCard
          icon={<AlertTriangle size={15} />}
          label="逾期任务"
          value={overdue}
          tone={overdue > 0 ? "text-red-600" : "text-slate-700"}
        />
        <MetricCard
          icon={<CalendarClock size={15} />}
          label="七天内到期"
          value={dueSoon}
          tone={dueSoon > 0 ? "text-amber-600" : "text-slate-700"}
        />
        <MetricCard
          icon={<CheckCircle2 size={15} />}
          label="完成率"
          value={`${pct}%`}
          tone="text-emerald-600"
        />
      </div>

      <Card
        title="计划基线"
        subtitle="保存当前 WBS / 依赖结构快照，作为挣值管理（EVM）与进度对比的计划基准"
      >
        <div className="mb-3">
          <Button
            size="sm"
            variant="secondary"
            disabled={createBaseline.isPending}
            onClick={() =>
              createBaseline.mutate(`基线 ${(baselines.data?.length ?? 0) + 1}`)
            }
          >
            <Flag size={13} /> 保存当前为基线
          </Button>
        </div>
        {baselines.isLoading ? (
          <div className="py-2 text-center text-xs text-slate-400">加载中…</div>
        ) : (baselines.data?.length ?? 0) === 0 ? (
          <div className="py-2 text-center text-xs text-slate-400">
            暂无基线——建议在排期确认后立即保存第一版基线
          </div>
        ) : (
          <div className="divide-y divide-slate-50">
            {baselines.data!.map((b) => (
              <div key={b.id} className="flex items-center justify-between py-2 text-sm">
                <span className="text-slate-700">{b.name}</span>
                <span className="font-mono text-xs text-slate-400">
                  {b.createdAt.slice(0, 16).replace("T", " ")}
                </span>
              </div>
            ))}
          </div>
        )}
      </Card>

      <Card
        title="报告与导出"
        subtitle="项目报告（Markdown，含章程/干系人/风险/任务进度/基线）与任务清单 CSV"
      >
        <div className="flex gap-2">
          <Button
            size="sm"
            variant="secondary"
            onClick={() =>
              void (async () => {
                try {
                  const path = await save({
                    defaultPath: `项目报告-${project.name}.md`,
                    filters: [{ name: "Markdown", extensions: ["md"] }],
                  });
                  if (!path) return;
                  await api.exportProjectReportTo(projectId, path);
                  toast("项目报告已导出");
                } catch (e) {
                  toastError((e as { message?: string }).message ?? "导出失败");
                }
              })()
            }
          >
            <FileDown size={13} /> 导出项目报告
          </Button>
          <Button
            size="sm"
            variant="secondary"
            onClick={() =>
              void (async () => {
                try {
                  const path = await save({
                    defaultPath: `任务清单-${project.name}.csv`,
                    filters: [{ name: "CSV", extensions: ["csv"] }],
                  });
                  if (!path) return;
                  await api.exportTasksCsvTo(projectId, path);
                  toast("任务 CSV 已导出");
                } catch (e) {
                  toastError((e as { message?: string }).message ?? "导出失败");
                }
              })()
            }
          >
            <Table2 size={13} /> 导出任务 CSV
          </Button>
        </div>
      </Card>

      <EvmSection project={project} tasks={tasks} />
    </div>
  );
}

function MetricCard({
  icon,
  label,
  value,
  tone,
}: {
  icon: ReactNode;
  label: string;
  value: string | number;
  tone: string;
}) {
  return (
    <div className="rounded-xl border border-slate-200 bg-white p-4 shadow-sm">
      <div className="flex items-center justify-between text-xs text-slate-500">
        {label}
        <span className="text-slate-300">{icon}</span>
      </div>
      <div className={`mt-1.5 text-xl font-bold ${tone}`}>{value}</div>
    </div>
  );
}
