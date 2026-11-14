import { useEffect, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { Bot, Check, X } from "lucide-react";

import { Badge } from "../ui/Badge";
import { Button } from "../ui/Button";
import { Dialog } from "../ui/Dialog";
import { Label, Textarea } from "../ui/Input";
import { api } from "../../lib/ipc";
import { toast, toastError } from "../../store/toast";
import type { WbsDraftItem, WbsDraftOut } from "../../types";

interface Props {
  projectId: string;
  open: boolean;
  defaultRequirements: string;
  onClose: () => void;
}

/** AI WBS 拆解：填写需求 → 生成草稿（两级）→ 勾选采纳创建任务 */
export function WbsDraftDialog({ projectId, open, defaultRequirements, onClose }: Props) {
  const qc = useQueryClient();
  const [requirements, setRequirements] = useState(defaultRequirements);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<WbsDraftOut | null>(null);
  const [checked, setChecked] = useState<Set<number>>(new Set());
  const [finishing, setFinishing] = useState(false);

  useEffect(() => {
    if (!open) return;
    setRequirements(defaultRequirements);
    setError(null);
    setResult(null);
    setChecked(new Set());
    if (defaultRequirements.trim().length > 0) {
      void generate(defaultRequirements);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open]);

  const generate = async (req: string) => {
    if (!req.trim()) {
      toastError("请先填写需求描述");
      return;
    }
    setLoading(true);
    setError(null);
    try {
      const out: WbsDraftOut = await api.generateWbsDraft(projectId, req);
      setResult(out);
      setChecked(new Set(out.items.map((_, i) => i)));
    } catch (e) {
      setError((e as { message?: string }).message ?? "生成失败");
    } finally {
      setLoading(false);
    }
  };

  const toggle = (i: number) =>
    setChecked((s) => {
      const next = new Set(s);
      if (next.has(i)) next.delete(i);
      else next.add(i);
      return next;
    });

  const adopt = async () => {
    if (!result) return;
    setFinishing(true);
    try {
      let created = 0;
      for (const [i, item] of result.items.entries()) {
        if (!checked.has(i)) continue;
        const parent = await api.createTask({
          projectId,
          title: item.title,
          durationDays: item.durationDays,
        });
        created += 1;
        for (const child of item.children ?? []) {
          await api.createTask({
            projectId,
            title: child.title,
            durationDays: child.durationDays,
            parentId: parent.id,
          });
          created += 1;
        }
      }
      if (result.artifactId) {
        try {
          await api.setArtifactStatus(result.artifactId, "adopted");
        } catch {
          /* 溯源标记失败不阻塞 */
        }
      }
      void qc.invalidateQueries({ queryKey: ["tasks", projectId] });
      void qc.invalidateQueries({ queryKey: ["schedule", projectId] });
      void qc.invalidateQueries({ queryKey: ["dashboard-stats"] });
      toast(`已创建 ${created} 个任务，可在列表中调整层级与依赖`);
      onClose();
    } catch (e) {
      toastError((e as { message?: string }).message ?? "创建任务失败");
    } finally {
      setFinishing(false);
    }
  };

  const selectedCount = result
    ? result.items.filter((_, i) => checked.has(i)).reduce((n, it) => n + 1 + (it.children?.length ?? 0), 0)
    : 0;

  return (
    <Dialog
      open={open}
      title={
        <span className="flex items-center gap-2">
          <Bot size={16} className="text-blue-600" /> AI 拆解 WBS
        </span>
      }
      onClose={onClose}
      className="max-w-xl"
    >
      <div className="space-y-3">
        <div>
          <Label>需求描述（粘一段需求/目标即可）</Label>
          <Textarea
            rows={4}
            value={requirements}
            onChange={(e) => setRequirements(e.target.value)}
            placeholder="例如：为一家连锁餐饮品牌开发会员小程序，包含注册登录、积分商城、门店核销、数据看板四个模块，6 周内上线。"
          />
        </div>
        <div className="flex items-center justify-between">
          <Button size="sm" onClick={() => void generate(requirements)} disabled={loading}>
            {loading ? "生成中…" : result ? "重新生成" : "生成草稿"}
          </Button>
          <span className="text-xs text-slate-400">通常需要 10~30 秒 · 草稿已记录溯源</span>
        </div>

        {error && (
          <div className="rounded-lg border border-red-100 bg-red-50 p-3 text-xs text-red-600">{error}</div>
        )}

        {result && (
          <div className="space-y-2">
            <div className="flex items-center gap-2 text-xs text-slate-500">
              <Badge tone="blue">已选 {selectedCount} 个任务</Badge>
              <span>勾选要采纳的顶层工作包（子任务随父级一起创建）</span>
            </div>
            <div className="max-h-72 space-y-1.5 overflow-y-auto rounded-lg border border-slate-200 p-2">
              {result.items.map((item: WbsDraftItem, i: number) => (
                <div key={i} className="rounded-lg bg-slate-50 px-2.5 py-2">
                  <label className="flex items-center gap-2 text-sm text-slate-700">
                    <input
                      type="checkbox"
                      checked={checked.has(i)}
                      onChange={() => toggle(i)}
                      className="accent-blue-600"
                    />
                    <span className="font-medium">{item.title}</span>
                    <span className="ml-auto text-xs text-slate-400">{item.durationDays}d</span>
                  </label>
                  {(item.children?.length ?? 0) > 0 && (
                    <div className="mt-1 space-y-0.5 pl-6">
                      {item.children.map((c, j) => (
                        <div key={j} className="text-xs text-slate-500">
                          {c.title} · {c.durationDays}d
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              ))}
            </div>
            <div className="flex justify-end gap-2">
              <Button variant="secondary" onClick={onClose}>
                <X size={14} /> 关闭
              </Button>
              <Button onClick={() => void adopt()} disabled={finishing || checked.size === 0}>
                <Check size={14} /> 采纳并创建任务
              </Button>
            </div>
          </div>
        )}
      </div>
    </Dialog>
  );
}
