import { useEffect, useState } from "react";
import { Bot } from "lucide-react";

import { CharterDraftDialog } from "../ai/CharterDraftDialog";
import { Button } from "../ui/Button";
import { Card } from "../ui/Card";
import { Label, Textarea } from "../ui/Input";
import { Term } from "../ui/Term";
import { useCharter, useSaveCharter } from "../../hooks/useProjects";
import { toast } from "../../store/toast";
import { CHARTER_SECTION_DEFS } from "../../data/charter";
import type { CharterContent } from "../../types";

export function CharterEditor({ projectId }: { projectId: string }) {
  const charter = useCharter(projectId);
  const save = useSaveCharter(projectId);
  const [draft, setDraft] = useState<CharterContent | null>(null);
  const [aiDialogOpen, setAiDialogOpen] = useState(false);

  useEffect(() => {
    if (charter.data) setDraft(charter.data.charter);
  }, [charter.data]);

  const dirty =
    draft !== null &&
    charter.data !== undefined &&
    JSON.stringify(draft) !== JSON.stringify(charter.data.charter);

  const setSection = (key: keyof CharterContent, value: string) =>
    setDraft((d) => (d ? { ...d, [key]: value } : d));

  return (
    <Card
      title={<Term id="charter">项目章程</Term>}
      subtitle={
        charter.data?.created
          ? "对应 PMBOK 8「启动」焦点领域 · 首次访问已生成空白章程，可用 AI 起草后逐节修改"
          : `对应 PMBOK 8「启动」焦点领域 · 最近保存 ${
              charter.data?.updatedAt.slice(0, 16).replace("T", " ") ?? "—"
            }`
      }
      actions={
        <>
          <Button
            size="sm"
            variant="secondary"
            onClick={() => setAiDialogOpen(true)}
            disabled={charter.isLoading}
            title="AI 基于项目信息与干系人/风险上下文起草章程草稿"
          >
            <Bot size={14} /> AI 起草
          </Button>
          <Button
            size="sm"
            disabled={!dirty || save.isPending}
            onClick={() => {
              if (!draft) return;
              save.mutate(draft, { onSuccess: () => toast("章程已保存") });
            }}
          >
            保存章程
          </Button>
        </>
      }
    >
      {charter.isLoading || draft === null ? (
        <div className="py-4 text-center text-xs text-slate-400">加载中…</div>
      ) : (
        <div className="grid grid-cols-2 gap-3">
          {CHARTER_SECTION_DEFS.map((s) => (
            <div key={s.key} className={s.key === "objective" ? "col-span-2" : undefined}>
              <Label>{s.label}</Label>
              <Textarea
                rows={3}
                value={draft[s.key]}
                onChange={(e) => setSection(s.key, e.target.value)}
                placeholder={s.placeholder}
              />
            </div>
          ))}
        </div>
      )}

      <CharterDraftDialog
        projectId={projectId}
        open={aiDialogOpen}
        onClose={() => setAiDialogOpen(false)}
      />
    </Card>
  );
}
