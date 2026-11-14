import type { ReactNode } from "react";
import { AlertTriangle, BarChart3, CalendarClock, CheckCircle2, Flag } from "lucide-react";

import { Button } from "../../components/ui/Button";
import { Card } from "../../components/ui/Card";
import { useBaselines, useCreateBaseline } from "../../hooks/useProjects";
import type { Task } from "../../types";

interface Props {
  projectId: string;
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

export function MonitoringTab({ projectId, tasks }: Props) {
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
        subtitle="保存当前 WBS / 依赖结构快照；M2 挣值管理（SPI/CPI）将以基线为对比基准"
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
        title="挣值管理（EVM）仪表盘"
        subtitle="M2 上线：基于基线快照 + 工时记录计算 PV/EV/AC、SPI/CPI、EAC 预测与 S 曲线"
      >
        <div className="flex items-center gap-3 rounded-lg border border-dashed border-slate-200 bg-slate-50/60 p-4">
          <BarChart3 size={22} className="text-slate-300" />
          <p className="text-xs leading-relaxed text-slate-500">
            EVM 是 PMBOK 成本/进度绩效的标准度量：设置基线并记录工时后，这里将呈现
            <span className="mx-1 font-mono text-slate-600">SPI / CPI / EAC</span>
            曲线，并由 AI 舵手给出偏差归因（S4）。当前页面先提供任务级健康信号与基线快照。
          </p>
        </div>
      </Card>
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
