import { useState } from "react";
import { Pencil, Plus, Trash2, Users } from "lucide-react";

import { Badge } from "../ui/Badge";
import { Button } from "../ui/Button";
import { Card } from "../ui/Card";
import { Dialog } from "../ui/Dialog";
import { EmptyState } from "../ui/EmptyState";
import { Term } from "../ui/Term";
import { Input, Label, Select, Textarea } from "../ui/Input";
import {
  useCreateStakeholder,
  useDeleteStakeholder,
  useStakeholders,
  useUpdateStakeholder,
} from "../../hooks/useProjects";
import { STAKEHOLDER_STRATEGY_LABEL, stakeholderStrategy } from "../../types";
import type { CreateStakeholderInput, Stakeholder, UpdateStakeholderInput } from "../../types";
import { toast } from "../../store/toast";
import { cn } from "../../lib/format";

interface Props {
  projectId: string;
}

const LEVELS = [1, 2, 3, 4, 5];

interface FormState {
  name: string;
  role: string;
  expectation: string;
  influence: number;
  interest: number;
  notes: string;
}

const EMPTY: FormState = { name: "", role: "", expectation: "", influence: 3, interest: 3, notes: "" };

const QUADRANTS: Array<{
  key: "hh" | "hl" | "lh" | "ll";
  hint: string;
  strategy: string;
  chipTone: string;
}> = [
  { key: "hh", hint: "高权力 · 高利益", strategy: "重点管理", chipTone: "bg-rose-50 text-rose-700 border-rose-200" },
  { key: "hl", hint: "高权力 · 低利益", strategy: "令其满意", chipTone: "bg-amber-50 text-amber-700 border-amber-200" },
  { key: "lh", hint: "低权力 · 高利益", strategy: "随时告知", chipTone: "bg-blue-50 text-blue-700 border-blue-200" },
  { key: "ll", hint: "低权力 · 低利益", strategy: "监控即可", chipTone: "bg-slate-50 text-slate-500 border-slate-200" },
];

function quadrantOf(s: Stakeholder): "hh" | "hl" | "lh" | "ll" {
  if (s.influence >= 4 && s.interest >= 4) return "hh";
  if (s.influence >= 4) return "hl";
  if (s.interest >= 4) return "lh";
  return "ll";
}

export function StakeholderSection({ projectId }: Props) {
  const stakeholders = useStakeholders(projectId);
  const create = useCreateStakeholder(projectId);
  const update = useUpdateStakeholder(projectId);
  const remove = useDeleteStakeholder(projectId);
  const [creating, setCreating] = useState(false);
  const [editing, setEditing] = useState<Stakeholder | null>(null);
  const [form, setForm] = useState<FormState>(EMPTY);

  const openDialog = (initial: Stakeholder | null) => {
    setForm(
      initial
        ? {
            name: initial.name,
            role: initial.role ?? "",
            expectation: initial.expectation ?? "",
            influence: initial.influence,
            interest: initial.interest,
            notes: initial.notes ?? "",
          }
        : EMPTY,
    );
    setEditing(initial);
    setCreating(initial === null);
  };
  const closeDialog = () => {
    setCreating(false);
    setEditing(null);
  };
  const dialogOpen = creating || editing !== null;

  const submit = () => {
    if (!form.name.trim()) {
      toast("请填写干系人姓名");
      return;
    }
    if (editing) {
      const payload: UpdateStakeholderInput = {
        id: editing.id,
        name: form.name,
        role: form.role || null,
        expectation: form.expectation || null,
        influence: form.influence,
        interest: form.interest,
        notes: form.notes || null,
      };
      update.mutate(payload, { onSuccess: () => { toast("干系人已更新"); closeDialog(); } });
    } else {
      const payload: CreateStakeholderInput = {
        projectId,
        name: form.name,
        role: form.role || null,
        expectation: form.expectation || null,
        influence: form.influence,
        interest: form.interest,
        notes: form.notes || null,
      };
      create.mutate(payload, { onSuccess: () => { toast("干系人已登记"); closeDialog(); } });
    }
  };

  const list = stakeholders.data ?? [];

  return (
    <Card
      title={<Term id="stakeholder-register">干系人登记册</Term>}
      subtitle="对应 PMBOK 8 干系人绩效域 · 权力/利益方格（1~5 标度，≥4 为高）"
      actions={
        <Button size="sm" onClick={() => openDialog(null)}>
          <Plus size={14} /> 登记干系人
        </Button>
      }
    >
      {stakeholders.isLoading ? (
        <div className="py-4 text-center text-xs text-slate-400">加载中…</div>
      ) : list.length === 0 ? (
        <EmptyState
          icon={<Users size={26} />}
          title="还没有登记干系人"
          description="识别所有影响项目或被项目影响的人：甲方、发起人、终端用户、监管方…"
          action={
            <Button size="sm" onClick={() => openDialog(null)}>
              登记第一位干系人
            </Button>
          }
        />
      ) : (
        <div className="flex flex-col gap-4 lg:flex-row">
          {/* 权力/利益方格 */}
          <div className="grid shrink-0 grid-cols-2 gap-2 lg:w-96">
            {QUADRANTS.map((q) => {
              const members = list.filter((s) => quadrantOf(s) === q.key);
              return (
                <div key={q.key} className="rounded-lg border border-slate-200 p-2">
                  <div className="mb-1.5 text-[10px] font-medium text-slate-400">{q.hint}</div>
                  <div className={cn("mb-1.5 inline-block rounded px-1.5 py-0.5 text-[10px] font-semibold", q.chipTone)}>
                    {q.strategy}
                  </div>
                  <div className="flex flex-wrap gap-1">
                    {members.length === 0 ? (
                      <span className="text-[10px] text-slate-300">—</span>
                    ) : (
                      members.map((s) => (
                        <button
                          key={s.id}
                          title={`${s.name}（权 ${s.influence} / 利 ${s.interest}）点击编辑`}
                          onClick={() => openDialog(s)}
                          className={cn("rounded border px-1.5 py-0.5 text-xs", q.chipTone)}
                        >
                          {s.name}
                        </button>
                      ))
                    )}
                  </div>
                </div>
              );
            })}
          </div>

          {/* 列表 */}
          <div className="min-w-0 flex-1 space-y-1.5">
            {list.map((s) => {
              const strategy = stakeholderStrategy(s.influence, s.interest);
              return (
                <div
                  key={s.id}
                  className="group flex items-center gap-2 rounded-lg border border-slate-100 bg-white px-3 py-2 hover:border-slate-200"
                >
                  <button
                    className="shrink-0 text-sm font-medium text-slate-800 hover:text-blue-700"
                    onClick={() => openDialog(s)}
                  >
                    {s.name}
                  </button>
                  {s.role && <span className="shrink-0 text-xs text-slate-400">{s.role}</span>}
                  <span
                    className="min-w-0 flex-1 truncate text-xs text-slate-500"
                    title={s.expectation ?? ""}
                  >
                    {s.expectation ?? "—"}
                  </span>
                  <span className="shrink-0 font-mono text-xs text-slate-400">
                    权{s.influence}/利{s.interest}
                  </span>
                  <Badge tone={strategy === "manage_closely" ? "red" : strategy === "keep_satisfied" ? "amber" : strategy === "keep_informed" ? "blue" : "gray"}>
                    {STAKEHOLDER_STRATEGY_LABEL[strategy]}
                  </Badge>
                  <div className="flex shrink-0 items-center gap-0.5 opacity-0 transition group-hover:opacity-100">
                    <button
                      title="编辑"
                      onClick={() => openDialog(s)}
                      className="rounded p-0.5 text-slate-300 hover:bg-slate-100 hover:text-slate-600"
                    >
                      <Pencil size={13} />
                    </button>
                    <button
                      title="删除"
                      onClick={() => {
                        if (window.confirm(`删除干系人「${s.name}」？`)) remove.mutate(s.id);
                      }}
                      className="rounded p-0.5 text-slate-300 hover:bg-slate-100 hover:text-red-500"
                    >
                      <Trash2 size={13} />
                    </button>
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      )}

      <Dialog
        open={dialogOpen}
        title={editing ? `编辑干系人` : "登记干系人"}
        onClose={closeDialog}
        className="max-w-lg"
      >
        <form
          className="space-y-3"
          onSubmit={(e) => {
            e.preventDefault();
            submit();
          }}
        >
          <div className="grid grid-cols-2 gap-3">
            <div>
              <Label required>姓名 / 主体</Label>
              <Input
                value={form.name}
                onChange={(e) => setForm((f) => ({ ...f, name: e.target.value }))}
                placeholder="例如：甲方项目负责人"
                autoFocus
              />
            </div>
            <div>
              <Label>角色</Label>
              <Input
                value={form.role}
                onChange={(e) => setForm((f) => ({ ...f, role: e.target.value }))}
                placeholder="发起人 / 评审人 / 使用方…"
              />
            </div>
          </div>
          <div className="grid grid-cols-2 gap-3">
            <div>
              <Label>权力（1~5）</Label>
              <Select
                value={String(form.influence)}
                onChange={(e) => setForm((f) => ({ ...f, influence: Number(e.target.value) }))}
              >
                {LEVELS.map((l) => (
                  <option key={l} value={l}>
                    {l}
                  </option>
                ))}
              </Select>
            </div>
            <div>
              <Label>利益/关注（1~5）</Label>
              <Select
                value={String(form.interest)}
                onChange={(e) => setForm((f) => ({ ...f, interest: Number(e.target.value) }))}
              >
                {LEVELS.map((l) => (
                  <option key={l} value={l}>
                    {l}
                  </option>
                ))}
              </Select>
            </div>
          </div>
          <div>
            <Label>期望与诉求</Label>
            <Textarea
              value={form.expectation}
              onChange={(e) => setForm((f) => ({ ...f, expectation: e.target.value }))}
              placeholder="TA 关心什么、期望从项目得到什么"
            />
          </div>
          <div>
            <Label>备注</Label>
            <Input
              value={form.notes}
              onChange={(e) => setForm((f) => ({ ...f, notes: e.target.value }))}
              placeholder="沟通偏好、注意事项等"
            />
          </div>
          <div className="flex justify-end gap-2 pt-1">
            <Button type="button" variant="secondary" onClick={closeDialog}>
              取消
            </Button>
            <Button type="submit" disabled={create.isPending || update.isPending}>
              {editing ? "保存修改" : "登记"}
            </Button>
          </div>
        </form>
      </Dialog>
    </Card>
  );
}
