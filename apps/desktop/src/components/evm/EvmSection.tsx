import { useMemo, useState } from "react";
import type { ReactNode } from "react";
import { BarChart3, Clock, LineChart, Plus, Trash2, TrendingUp } from "lucide-react";

import { useBaselines, useCreateTimeLog, useDeleteTimeLog, useEvm, useTimeLogs } from "../../hooks/useProjects";
import { Badge } from "../../components/ui/Badge";
import { Button } from "../../components/ui/Button";
import { Card } from "../../components/ui/Card";
import { Dialog } from "../../components/ui/Dialog";
import { EmptyState } from "../../components/ui/EmptyState";
import { Input, Label, Select, Textarea } from "../../components/ui/Input";
import { Term } from "../ui/Term";
import { toast } from "../../store/toast";
import type { EvmReport, Project, Task, TimeLog } from "../../types";

interface Props {
  project: Project;
  tasks: Task[];
}

function todayStr(): string {
  const d = new Date();
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

/** 指数健康度：≥1 绿，接近 1 琥珀，显著落后红。 */
function indexTone(v: number): "green" | "amber" | "red" {
  if (v >= 1) return "green";
  if (v >= 0.95) return "amber";
  return "red";
}

const toneClass: Record<"green" | "amber" | "red", string> = {
  green: "text-emerald-600",
  amber: "text-amber-600",
  red: "text-red-600",
};

function fmtHours(v: number): string {
  return Number.isInteger(v) ? String(v) : v.toFixed(1);
}

interface LogSubmit {
  projectId: string;
  taskId: string;
  logDate: string;
  hours: number;
  note: string | null;
}

export function EvmSection({ project, tasks }: Props) {
  const projectId = project.id;
  const [baselineId, setBaselineId] = useState<string | null>(null); // null = 最新基线
  const [logOpen, setLogOpen] = useState(false);

  const evm = useEvm(projectId, baselineId);
  const timeLogs = useTimeLogs(projectId);
  const baselines = useBaselines(projectId);
  const createLog = useCreateTimeLog(projectId);
  const deleteLog = useDeleteTimeLog(projectId);

  const taskById = useMemo(() => new Map(tasks.map((t) => [t.id, t])), [tasks]);
  // 只列叶子任务：工时登记发生在工作包层面
  const leafTasks = useMemo(
    () =>
      tasks
        .filter((t) => !tasks.some((c) => c.parentId === t.id))
        .sort((a, b) => (a.wbsCode ?? "").localeCompare(b.wbsCode ?? "", "en", { numeric: true })),
    [tasks],
  );

  return (
    <Card
      title={
        <span className="flex items-center gap-1.5">
          <BarChart3 size={13} className="text-blue-500" /> 挣值管理（EVM）
        </span>
      }
      subtitle="以基线为计划基准、工时台账为实际成本，计算 PV/EV/AC、SPI/CPI 与 EAC 预测（单位：人时）"
    >
      <div className="mb-4 flex flex-wrap items-center gap-2">
        <div className="w-56">
          <Select
            aria-label="选择基线"
            value={baselineId ?? ""}
            onChange={(e) => setBaselineId(e.target.value || null)}
          >
            <option value="">最新基线</option>
            {(baselines.data ?? []).map((b) => (
              <option key={b.id} value={b.id}>
                {b.name}（{b.createdAt.slice(0, 10)}）
              </option>
            ))}
          </Select>
        </div>
        <Button size="sm" variant="secondary" onClick={() => setLogOpen(true)}>
          <Plus size={13} /> 登记工时
        </Button>
        <span className="text-xs text-slate-400">
          SPI/CPI 以"人时"为价值单位；基线后新增的任务按当前预估计价
        </span>
      </div>

      {evm.isLoading ? (
        <div className="py-6 text-center text-xs text-slate-400">加载中…</div>
      ) : evm.isError ? (
        <div className="py-6 text-center text-xs text-red-600">
          挣值数据加载失败：{evm.error.message}
        </div>
      ) : (
        <EvmBody report={evm.data!} />
      )}

      <TimeLogList
        logs={timeLogs.data ?? []}
        taskById={taskById}
        onDelete={(id) => deleteLog.mutate(id)}
        pending={deleteLog.isPending}
      />

      <LogDialog
        open={logOpen}
        onClose={() => setLogOpen(false)}
        tasks={leafTasks}
        projectId={projectId}
        pending={createLog.isPending}
        onSubmit={(input) =>
          createLog.mutate(input, {
            onSuccess: () => {
              setLogOpen(false);
              toast("工时已登记");
            },
          })
        }
      />
    </Card>
  );
}

// ---------- 报表主体 ----------

function EvmBody({ report }: { report: EvmReport }) {
  const s = report.summary;
  const today = todayStr();

  if (!report.hasBaseline && s.ac === 0) {
    return (
      <EmptyState
        icon={<LineChart size={30} />}
        title="还没有基线"
        description="保存一版计划基线，并给任务填写预估工时与起止日期；登记工时后，这里会自动生成 PV/EV/AC 的 S 曲线与 SPI/CPI/EAC 指标。"
      />
    );
  }

  return (
    <div className="space-y-4">
      <div className="grid grid-cols-4 gap-3">
        <MetricCell label={<Term id="bac">BAC 完工预算</Term>} value={`${fmtHours(s.bac)}h`} tone="text-slate-800" />
        <MetricCell label={<Term id="pv">PV 计划价值</Term>} value={`${fmtHours(s.pv)}h`} tone="text-slate-600" />
        <MetricCell label={<Term id="ev">EV 挣值</Term>} value={`${fmtHours(s.ev)}h`} tone="text-emerald-600" />
        <MetricCell label={<Term id="ac">AC 实际成本</Term>} value={`${fmtHours(s.ac)}h`} tone="text-rose-600" />
      </div>
      <div className="grid grid-cols-4 gap-3">
        <MetricCell
          label={<Term id="spi">SPI 进度绩效</Term>}
          value={s.spi.toFixed(2)}
          tone={toneClass[indexTone(s.spi)]}
          hint={s.spi >= 1 ? "进度符合或超前计划" : "进度落后于计划"}
        />
        <MetricCell
          label={<Term id="cpi">CPI 成本绩效</Term>}
          value={s.cpi.toFixed(2)}
          tone={toneClass[indexTone(s.cpi)]}
          hint={s.cpi >= 1 ? "成本效率达标" : "每挣 1 人时花费超预算"}
        />
        <MetricCell
          label={<Term id="eac">EAC 完工估算</Term>}
          value={`${fmtHours(s.eac)}h`}
          tone={s.eac > s.bac ? "text-red-600" : "text-slate-800"}
          hint={`按当前 CPI 推算 · VAC ${fmtHours(s.vac)}h`}
        />
        <MetricCell
          label="完成率（挣值法）"
          value={`${s.percentComplete.toFixed(0)}%`}
          tone="text-blue-600"
          hint={`SV ${fmtHours(s.sv)}h · CV ${fmtHours(s.cv)}h`}
        />
      </div>

      {report.unscheduledHours > 0 && (
        <p className="text-xs text-amber-600">
          基线中有 {fmtHours(report.unscheduledHours)} 人时的任务未排起止日期，未计入 PV 曲线。
        </p>
      )}

      <SCurve report={report} today={today} />
    </div>
  );
}

function MetricCell({
  label,
  value,
  tone,
  hint,
}: {
  label: ReactNode;
  value: string;
  tone: string;
  hint?: string;
}) {
  return (
    <div className="rounded-lg border border-slate-100 bg-slate-50/60 p-3">
      <div className="text-[11px] text-slate-500">{label}</div>
      <div className={`mt-1 text-lg font-bold ${tone}`}>{value}</div>
      {hint && <div className="mt-0.5 text-[10px] leading-tight text-slate-400">{hint}</div>}
    </div>
  );
}

// ---------- S 曲线（手绘 SVG，零依赖） ----------

const W = 760;
const H = 240;
const PAD_L = 44;
const PAD_R = 12;
const PAD_T = 14;
const PAD_B = 26;

function SCurve({ report, today }: { report: EvmReport; today: string }) {
  const { series } = report;
  if (series.length === 0) {
    return (
      <div className="rounded-lg border border-dashed border-slate-200 bg-slate-50/60 p-4 text-center text-xs text-slate-400">
        基线中还没有可排期的任务——给任务填写起止日期与预估工时后生成 S 曲线
      </div>
    );
  }

  const s = report.summary;
  const yMax = Math.max(s.bac, s.pv, s.ev, s.ac, 1);
  const innerW = W - PAD_L - PAD_R;
  const innerH = H - PAD_T - PAD_B;
  const n = series.length;
  const x = (i: number) => PAD_L + (n === 1 ? innerW / 2 : (i / (n - 1)) * innerW);
  const y = (v: number) => PAD_T + innerH - (v / yMax) * innerH;

  const line = (key: "pv" | "ev" | "ac") =>
    series.map((p, i) => `${x(i).toFixed(1)},${y(p[key]).toFixed(1)}`).join(" ");

  const todayIdx = series.findIndex((p) => p.date === today);
  const labelIdx = n > 2 ? [0, Math.floor((n - 1) / 2), n - 1] : series.map((_, i) => i);
  const yTicks = [0, 0.25, 0.5, 0.75, 1].map((f) => yMax * f);
  const dotStep = Math.max(1, Math.floor(n / 12));

  return (
    <div>
      <div className="mb-1 flex items-center gap-4 text-[11px] text-slate-500">
        <span className="flex items-center gap-1">
          <span className="inline-block h-0.5 w-4 bg-slate-400" /> PV 计划
        </span>
        <span className="flex items-center gap-1">
          <span className="inline-block h-0.5 w-4 bg-emerald-500" /> EV 挣值
        </span>
        <span className="flex items-center gap-1">
          <span className="inline-block h-0.5 w-4 bg-rose-500" /> AC 实际
        </span>
        {todayIdx >= 0 && (
          <span className="flex items-center gap-1">
            <span className="inline-block h-3 w-px bg-blue-400" /> 今天
          </span>
        )}
      </div>
      <svg viewBox={`0 0 ${W} ${H}`} className="w-full" role="img" aria-label="EVM S 曲线">
        {yTicks.map((v) => (
          <g key={v}>
            <line x1={PAD_L} y1={y(v)} x2={W - PAD_R} y2={y(v)} stroke="#f1f5f9" />
            <text x={PAD_L - 6} y={y(v) + 3} textAnchor="end" fontSize="9" fill="#94a3b8">
              {fmtHours(Math.round(v * 10) / 10)}
            </text>
          </g>
        ))}
        {todayIdx >= 0 && (
          <line
            x1={x(todayIdx)}
            y1={PAD_T}
            x2={x(todayIdx)}
            y2={PAD_T + innerH}
            stroke="#60a5fa"
            strokeDasharray="3 3"
          />
        )}
        <polyline points={line("pv")} fill="none" stroke="#94a3b8" strokeWidth="1.5" />
        <polyline points={line("ac")} fill="none" stroke="#f43f5e" strokeWidth="1.8" />
        <polyline points={line("ev")} fill="none" stroke="#10b981" strokeWidth="1.8" />
        {labelIdx.map((i) => (
          <text
            key={i}
            x={x(i)}
            y={H - 8}
            textAnchor={i === 0 ? "start" : i === n - 1 ? "end" : "middle"}
            fontSize="9"
            fill="#94a3b8"
          >
            {series[i].date.slice(5)}
          </text>
        ))}
        {series.map(
          (p, i) =>
            i % dotStep === 0 && (
              <circle key={p.date} cx={x(i)} cy={y(p.ev)} r="2" fill="#10b981">
                <title>{`${p.date} · PV ${fmtHours(p.pv)} / EV ${fmtHours(p.ev)} / AC ${fmtHours(p.ac)}`}</title>
              </circle>
            ),
        )}
      </svg>
    </div>
  );
}

// ---------- 工时台账 ----------

function TimeLogList({
  logs,
  taskById,
  onDelete,
  pending,
}: {
  logs: TimeLog[];
  taskById: Map<string, Task>;
  onDelete: (id: string) => void;
  pending: boolean;
}) {
  if (logs.length === 0) {
    return (
      <div className="mt-4 rounded-lg border border-dashed border-slate-200 bg-slate-50/60 p-3 text-center text-xs text-slate-400">
        还没有工时记录——点击"登记工时"记录实际投入，作为 EVM 的 AC 数据源
      </div>
    );
  }
  return (
    <div className="mt-4">
      <div className="mb-1 flex items-center gap-1.5 text-xs font-semibold text-slate-600">
        <Clock size={12} className="text-slate-400" /> 最近工时记录
      </div>
      <div className="divide-y divide-slate-50">
        {logs.map((l) => {
          const task = taskById.get(l.taskId);
          return (
            <div key={l.id} className="flex items-center gap-2 py-1.5 text-sm">
              <span className="w-20 shrink-0 font-mono text-xs text-slate-400">{l.logDate}</span>
              <span className="min-w-0 flex-1 truncate text-slate-700">
                <span className="text-xs text-slate-400">
                  {task?.wbsCode ? `${task.wbsCode} ` : ""}
                </span>
                {task?.title ?? "（任务已删除）"}
                {l.note && <span className="ml-1 text-xs text-slate-400">· {l.note}</span>}
              </span>
              <Badge tone="blue">{fmtHours(l.hours)}h</Badge>
              <button
                aria-label="删除工时记录"
                disabled={pending}
                onClick={() => onDelete(l.id)}
                className="rounded p-1 text-slate-300 hover:bg-red-50 hover:text-red-500 disabled:opacity-40"
              >
                <Trash2 size={13} />
              </button>
            </div>
          );
        })}
      </div>
    </div>
  );
}

function LogDialog({
  open,
  onClose,
  tasks,
  projectId,
  pending,
  onSubmit,
}: {
  open: boolean;
  onClose: () => void;
  tasks: Task[];
  projectId: string;
  pending: boolean;
  onSubmit: (input: LogSubmit) => void;
}) {
  const [taskId, setTaskId] = useState("");
  const [logDate, setLogDate] = useState(todayStr());
  const [hours, setHours] = useState("1");
  const [note, setNote] = useState("");

  const submit = () => {
    const h = Number(hours);
    if (!taskId || !Number.isFinite(h) || h <= 0) return;
    onSubmit({ projectId, taskId, logDate, hours: h, note: note.trim() || null });
  };

  return (
    <Dialog open={open} title="登记工时" onClose={onClose}>
      <div className="space-y-3">
        <div>
          <Label required>任务</Label>
          <Select value={taskId} onChange={(e) => setTaskId(e.target.value)}>
            <option value="">选择任务…</option>
            {tasks.map((t) => (
              <option key={t.id} value={t.id}>
                {t.wbsCode ? `${t.wbsCode} ` : ""}
                {t.title}
              </option>
            ))}
          </Select>
        </div>
        <div className="grid grid-cols-2 gap-3">
          <div>
            <Label required>日期</Label>
            <Input type="date" value={logDate} onChange={(e) => setLogDate(e.target.value)} />
          </div>
          <div>
            <Label required>工时（小时）</Label>
            <Input
              type="number"
              min="0.5"
              max="24"
              step="0.5"
              value={hours}
              onChange={(e) => setHours(e.target.value)}
            />
          </div>
        </div>
        <div>
          <Label>备注</Label>
          <Textarea
            value={note}
            placeholder="做了什么（可选）"
            onChange={(e) => setNote(e.target.value)}
          />
        </div>
        <div className="flex items-center justify-between pt-1">
          <span className="flex items-center gap-1 text-[11px] text-slate-400">
            <TrendingUp size={11} /> 工时计入 EVM 的实际成本（AC）
          </span>
          <div className="flex gap-2">
            <Button variant="ghost" size="sm" onClick={onClose}>
              取消
            </Button>
            <Button size="sm" disabled={pending || !taskId} onClick={submit}>
              保存
            </Button>
          </div>
        </div>
      </div>
    </Dialog>
  );
}
