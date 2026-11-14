import { useEffect, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { Bot, Check, X } from "lucide-react";

import { Badge } from "../ui/Badge";
import { Button } from "../ui/Button";
import { Dialog } from "../ui/Dialog";
import { api } from "../../lib/ipc";
import { toast, toastError } from "../../store/toast";
import { CHARTER_SECTION_DEFS } from "../../data/charter";
import { EMPTY_CHARTER } from "../../types";
import type { CharterContent } from "../../types";

interface Props {
  projectId: string;
  open: boolean;
  onClose: () => void;
}

/** AI 章程草稿：生成 → 预览 → 采纳（直接保存到章程）/ 弃用 */
export function CharterDraftDialog({ projectId, open, onClose }: Props) {
  const qc = useQueryClient();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [artifactId, setArtifactId] = useState<string | null>(null);
  const [draft, setDraft] = useState<CharterContent | null>(null);
  const [finishing, setFinishing] = useState(false);

  useEffect(() => {
    if (!open) return;
    setLoading(true);
    setError(null);
    setDraft(null);
    setArtifactId(null);
    api
      .generateCharterDraft(projectId)
      .then((out) => {
        setArtifactId(out.artifactId);
        setDraft(out.charter);
      })
      .catch((e) => setError((e as { message?: string }).message ?? "生成失败"))
      .finally(() => setLoading(false));
  }, [open, projectId]);

  const adopt = async () => {
    if (!draft || !artifactId) return;
    setFinishing(true);
    try {
      await api.saveCharter(projectId, draft);
      await api.setArtifactStatus(artifactId, "adopted");
      void qc.invalidateQueries({ queryKey: ["charter", projectId] });
      toast("AI 草稿已保存到章程，可在编辑器中继续修改");
      onClose();
    } catch (e) {
      toastError((e as { message?: string }).message ?? "保存失败");
    } finally {
      setFinishing(false);
    }
  };

  const discard = async () => {
    if (artifactId) {
      try {
        await api.setArtifactStatus(artifactId, "discarded");
      } catch {
        /* 溯源标记失败不影响关闭 */
      }
    }
    onClose();
  };

  const filled = draft
    ? Object.values(draft).filter((v) => v.trim().length > 0).length
    : 0;

  return (
    <Dialog open={open} title={<span className="flex items-center gap-2"><Bot size={16} className="text-blue-600" /> AI 章程草稿</span>} onClose={onClose} className="max-w-xl">
      {loading ? (
        <div className="py-10 text-center text-sm text-slate-500">
          正在基于项目信息与干系人/风险上下文起草章程…
          <div className="mt-2 text-xs text-slate-400">通常需要 10~30 秒</div>
        </div>
      ) : error ? (
        <div className="space-y-3">
          <div className="rounded-lg border border-red-100 bg-red-50 p-3 text-xs text-red-600">
            {error}
          </div>
          <div className="flex justify-end">
            <Button variant="secondary" onClick={onClose}>
              关闭
            </Button>
          </div>
        </div>
      ) : draft ? (
        <div className="space-y-3">
          <div className="flex items-center gap-2 text-xs text-slate-500">
            <Badge tone="blue">{filled}/8 节已生成</Badge>
            <span>请逐节检查后再采纳——AI 产出需要你把关（草稿已记录溯源）</span>
          </div>
          <div className="max-h-80 space-y-3 overflow-y-auto rounded-lg border border-slate-200 p-3">
            {CHARTER_SECTION_DEFS.map((s) => (
              <div key={s.key}>
                <div className="text-xs font-semibold text-slate-600">{s.label}</div>
                <div className="mt-0.5 whitespace-pre-wrap text-sm text-slate-700">
                  {draft[s.key]?.trim() || <span className="text-slate-300">（本节未生成）</span>}
                </div>
              </div>
            ))}
          </div>
          <div className="flex justify-end gap-2">
            <Button variant="secondary" onClick={() => void discard()}>
              <X size={14} /> 弃用
            </Button>
            <Button onClick={() => void adopt()} disabled={finishing || filled === 0}>
              <Check size={14} /> 采纳并保存到章程
            </Button>
          </div>
        </div>
      ) : null}
    </Dialog>
  );
}

// 保持 EMPTY_CHARTER 引用（采纳时兜底使用）
void EMPTY_CHARTER;
