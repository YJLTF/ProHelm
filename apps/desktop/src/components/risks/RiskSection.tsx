import { useState } from "react";
import { Pencil, Plus, Trash2, TriangleAlert } from "lucide-react";

import { Badge } from "../ui/Badge";
import { Button } from "../ui/Button";
import { Card } from "../ui/Card";
import { Dialog } from "../ui/Dialog";
import { EmptyState } from "../ui/EmptyState";
import { Term } from "../ui/Term";
import { Input, Label, Select, Textarea } from "../ui/Input";
import {
  useCreateRisk,
  useDeleteRisk,
  useRisks,
  useUpdateRisk,
} from "../../hooks/useProjects";
import {
  RESPONSE_STRATEGY_LABEL,
  RISK_CATEGORY_LABEL,
  RISK_STATUS_LABEL,
  riskScoreTone,
} from "../../types";
import type {
  CreateRiskInput,
  Risk,
  RiskCategory,
  RiskStatusType,
  ResponseStrategy,
  UpdateRiskInput,
} from "../../types";
import { toast } from "../../store/toast";

interface Props {
  projectId: string;
}

const LEVELS = [1, 2, 3, 4, 5];

interface FormState {
  title: string;
  detail: string;
  category: RiskCategory;
  probability: number;
  impact: number;
  responseStrategy: ResponseStrategy;
  responsePlan: string;
  owner: string;
  triggerDesc: string;
}

const EMPTY: FormState = {
  title: "",
  detail: "",
  category: "technical",
  probability: 3,
  impact: 3,
  responseStrategy: "mitigate",
  responsePlan: "",
  owner: "",
  triggerDesc: "",
};

export function RiskSection({ projectId }: Props) {
  const risks = useRisks(projectId);
  const create = useCreateRisk(projectId);
  const update = useUpdateRisk(projectId);
  const remove = useDeleteRisk(projectId);
  const [creating, setCreating] = useState(false);
  const [editing, setEditing] = useState<Risk | null>(null);
  const [form, setForm] = useState<FormState>(EMPTY);

  const openDialog = (initial: Risk | null) => {
    if (initial) {
      setForm({
        title: initial.title,
        detail: initial.detail ?? "",
        category: initial.category,
        probability: initial.probability,
        impact: initial.impact,
        responseStrategy: initial.responseStrategy,
        responsePlan: initial.responsePlan ?? "",
        owner: initial.owner ?? "",
        triggerDesc: initial.triggerDesc ?? "",
      });
    } else {
      setForm(EMPTY);
    }
    setEditing(initial);
    setCreating(initial === null);
  };
  const closeDialog = () => {
    setCreating(false);
    setEditing(null);
  };
  const dialogOpen = creating || editing !== null;

  const submit = () => {
    if (!form.title.trim()) {
      toast("请填写风险标题");
      return;
    }
    if (editing) {
      const payload: UpdateRiskInput = {
        id: editing.id,
        title: form.title,
        detail: form.detail || null,
        category: form.category,
        probability: form.probability,
        impact: form.impact,
        responseStrategy: form.responseStrategy,
        responsePlan: form.responsePlan || null,
        owner: form.owner || null,
        triggerDesc: form.triggerDesc || null,
      };
      update.mutate(payload, { onSuccess: () => { toast("风险已更新"); closeDialog(); } });
    } else {
      const payload: CreateRiskInput = {
        projectId,
        title: form.title,
        detail: form.detail || null,
        category: form.category,
        probability: form.probability,
        impact: form.impact,
        responseStrategy: form.responseStrategy,
        responsePlan: form.responsePlan || null,
        owner: form.owner || null,
        triggerDesc: form.triggerDesc || null,
      };
      create.mutate(payload, { onSuccess: () => { toast("风险已登记"); closeDialog(); } });
    }
  };

  const list = risks.data ?? [];
  const openCount = list.filter((r) => r.status !== "closed").length;

  return (
    <Card
      title={<Term id="risk-register">风险登记册</Term>}
      subtitle={`对应 PMBOK 8 风险绩效域 · 概率×影响（1~5 标度）· ${openCount} 个未关闭`}
      actions={
        <Button size="sm" onClick={() => openDialog(null)}>
          <Plus size={14} /> 登记风险
        </Button>
      }
    >
      {risks.isLoading ? (
        <div className="py-4 text-center text-xs text-slate-400">加载中…</div>
      ) : list.length === 0 ? (
        <EmptyState
          icon={<TriangleAlert size={26} />}
          title="还没有登记风险"
          description="从 WBS 与章程出发识别风险：先用矩阵定位概率与影响，再选择应对策略。"
          action={
            <Button size="sm" onClick={() => openDialog(null)}>
              登记第一条风险
            </Button>
          }
        />
      ) : (
        <div className="flex flex-col gap-4 lg:flex-row">
          <MatrixHeat risks={list} />
          <div className="min-w-0 flex-1 space-y-1.5">
            {list.map((r) => (
              <div
                key={r.id}
                className="group flex items-center gap-2 rounded-lg border border-slate-100 bg-white px-3 py-2 hover:border-slate-200"
              >
                <span className="w-9 shrink-0 font-mono text-xs text-slate-400">
                  R{r.riskNo}
                </span>
                <button
                  className="min-w-0 flex-1 truncate text-left text-sm text-slate-800 hover:text-blue-700"
                  title={r.detail ?? r.title}
                  onClick={() => openDialog(r)}
                >
                  {r.title}
                </button>
                <Badge tone={riskScoreTone(r.probability * r.impact)}>
                  {r.probability * r.impact} 分
                </Badge>
                <Badge>{RISK_CATEGORY_LABEL[r.category]}</Badge>
                <Badge tone="blue">{RESPONSE_STRATEGY_LABEL[r.responseStrategy]}</Badge>
                {r.owner && <span className="shrink-0 text-xs text-slate-500">{r.owner}</span>}
                <div className="w-20 shrink-0">
                  <Select
                    className="px-2 py-1 text-xs"
                    value={r.status}
                    onChange={(e) =>
                      update.mutate({ id: r.id, status: e.target.value as RiskStatusType })
                    }
                  >
                    {(Object.keys(RISK_STATUS_LABEL) as RiskStatusType[]).map((s) => (
                      <option key={s} value={s}>
                        {RISK_STATUS_LABEL[s]}
                      </option>
                    ))}
                  </Select>
                </div>
                <div className="flex shrink-0 items-center gap-0.5 opacity-0 transition group-hover:opacity-100">
                  <button
                    title="编辑"
                    onClick={() => openDialog(r)}
                    className="rounded p-0.5 text-slate-300 hover:bg-slate-100 hover:text-slate-600"
                  >
                    <Pencil size={13} />
                  </button>
                  <button
                    title="删除"
                    onClick={() => {
                      if (window.confirm(`删除风险「${r.title}」？`)) remove.mutate(r.id);
                    }}
                    className="rounded p-0.5 text-slate-300 hover:bg-slate-100 hover:text-red-500"
                  >
                    <Trash2 size={13} />
                  </button>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      <Dialog
        open={dialogOpen}
        title={editing ? `编辑风险 R${editing.riskNo}` : "登记风险"}
        onClose={closeDialog}
        className="max-w-xl"
      >
        <form
          className="space-y-3"
          onSubmit={(e) => {
            e.preventDefault();
            submit();
          }}
        >
          <div>
            <Label required>风险标题</Label>
            <Input
              value={form.title}
              onChange={(e) => setForm((f) => ({ ...f, title: e.target.value }))}
              placeholder="例如：需求蔓延导致工期失守"
              autoFocus
            />
          </div>
          <div>
            <Label>描述 / 原因 / 影响</Label>
            <Textarea
              value={form.detail}
              onChange={(e) => setForm((f) => ({ ...f, detail: e.target.value }))}
              placeholder="风险由什么引起、一旦发生影响什么"
            />
          </div>
          <div className="grid grid-cols-3 gap-3">
            <div>
              <Label>类别</Label>
              <Select
                value={form.category}
                onChange={(e) => setForm((f) => ({ ...f, category: e.target.value as RiskCategory }))}
              >
                {(Object.keys(RISK_CATEGORY_LABEL) as RiskCategory[]).map((c) => (
                  <option key={c} value={c}>
                    {RISK_CATEGORY_LABEL[c]}
                  </option>
                ))}
              </Select>
            </div>
            <div>
              <Label>概率（1~5）</Label>
              <Select
                value={String(form.probability)}
                onChange={(e) => setForm((f) => ({ ...f, probability: Number(e.target.value) }))}
              >
                {LEVELS.map((l) => (
                  <option key={l} value={l}>
                    {l}
                  </option>
                ))}
              </Select>
            </div>
            <div>
              <Label>影响（1~5）</Label>
              <Select
                value={String(form.impact)}
                onChange={(e) => setForm((f) => ({ ...f, impact: Number(e.target.value) }))}
              >
                {LEVELS.map((l) => (
                  <option key={l} value={l}>
                    {l}
                  </option>
                ))}
              </Select>
            </div>
          </div>
          <div className="grid grid-cols-2 gap-3">
            <div>
              <Label>应对策略</Label>
              <Select
                value={form.responseStrategy}
                onChange={(e) =>
                  setForm((f) => ({ ...f, responseStrategy: e.target.value as ResponseStrategy }))
                }
              >
                {(Object.keys(RESPONSE_STRATEGY_LABEL) as ResponseStrategy[]).map((s) => (
                  <option key={s} value={s}>
                    {RESPONSE_STRATEGY_LABEL[s]}
                  </option>
                ))}
              </Select>
            </div>
            <div>
              <Label>责任人</Label>
              <Input
                value={form.owner}
                onChange={(e) => setForm((f) => ({ ...f, owner: e.target.value }))}
                placeholder="谁盯这个风险"
              />
            </div>
          </div>
          <div>
            <Label>应对计划</Label>
            <Textarea
              value={form.responsePlan}
              onChange={(e) => setForm((f) => ({ ...f, responsePlan: e.target.value }))}
              placeholder="具体怎么规避 / 减轻 / 转移"
            />
          </div>
          <div>
            <Label>预警信号（触发条件）</Label>
            <Input
              value={form.triggerDesc}
              onChange={(e) => setForm((f) => ({ ...f, triggerDesc: e.target.value }))}
              placeholder="出现什么迹象说明风险正在发生"
            />
          </div>
          <div className="flex items-center justify-between pt-1">
            <Badge tone={riskScoreTone(form.probability * form.impact)}>
              评分 {form.probability}×{form.impact} = {form.probability * form.impact}
            </Badge>
            <div className="flex gap-2">
              <Button type="button" variant="secondary" onClick={closeDialog}>
                取消
              </Button>
              <Button type="submit" disabled={create.isPending || update.isPending}>
                {editing ? "保存修改" : "登记"}
              </Button>
            </div>
          </div>
        </form>
      </Dialog>
    </Card>
  );
}

/** 5×5 概率×影响热力矩阵：格子显示落在该组合的风险数量 */
function MatrixHeat({ risks }: { risks: Risk[] }) {
  const tone = (score: number) =>
    score >= 15
      ? "bg-rose-500/85 text-white"
      : score >= 8
        ? "bg-amber-400/85 text-slate-800"
        : "bg-emerald-400/70 text-slate-800";
  const count = (p: number, i: number) =>
    risks.filter((r) => r.probability === p && r.impact === i).length;

  return (
    <div className="shrink-0">
      <div className="mb-1 text-center text-[10px] text-slate-400">影响 →</div>
      <div className="grid grid-cols-6 gap-0.5 text-center text-[10px]">
        <div />
        {LEVELS.map((i) => (
          <div key={`h${i}`} className="text-slate-400">
            {i}
          </div>
        ))}
        {[...LEVELS].reverse().map((p) => (
          <Row key={p} p={p} count={count} tone={tone} />
        ))}
      </div>
      <div className="mt-1 text-center text-[10px] text-slate-400">← 概率</div>
    </div>
  );
}

function Row({
  p,
  count,
  tone,
}: {
  p: number;
  count: (p: number, i: number) => number;
  tone: (score: number) => string;
}) {
  return (
    <>
      <div className="flex items-center justify-end pr-1 text-slate-400">{p}</div>
      {LEVELS.map((i) => {
        const c = count(p, i);
        return (
          <div
            key={`${p}-${i}`}
            title={`概率 ${p} × 影响 ${i} = ${p * i} 分${c > 0 ? `（${c} 个风险）` : ""}`}
            className={`flex h-8 w-10 items-center justify-center rounded ${tone(p * i)} ${
              c > 0 ? "font-bold ring-2 ring-slate-700/60" : "opacity-60"
            }`}
          >
            {c > 0 ? c : ""}
          </div>
        );
      })}
    </>
  );
}
