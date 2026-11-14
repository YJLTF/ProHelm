import { useEffect, useState } from "react";

import { Button } from "../ui/Button";
import { Card } from "../ui/Card";
import { Label, Textarea } from "../ui/Input";
import { useCharter, useSaveCharter } from "../../hooks/useProjects";
import { toast } from "../../store/toast";
import type { CharterContent } from "../../types";

interface SectionDef {
  key: keyof CharterContent;
  label: string;
  placeholder: string;
}

const SECTIONS: SectionDef[] = [
  { key: "objective", label: "项目目的与目标", placeholder: "为什么做这个项目？要达成什么？" },
  { key: "successCriteria", label: "成功标准（可度量）", placeholder: "例如：UAT 一次通过；上线后崩溃率 < 0.1%" },
  { key: "keyDeliverables", label: "主要交付物", placeholder: "系统 / 文档 / 报告…" },
  { key: "milestones", label: "关键里程碑", placeholder: "W3 需求冻结 / W7 开发完成…" },
  { key: "constraints", label: "制约因素", placeholder: "工期、预算、范围、合规…" },
  { key: "assumptions", label: "假设条件", placeholder: "客户按时提供测试环境与数据…" },
  { key: "highLevelRisks", label: "高层级风险", placeholder: "详细管理见「规划 → 风险登记册」" },
  { key: "pmAuthority", label: "项目经理职权", placeholder: "可动用资源、审批权限、对外承诺边界" },
];

export function CharterEditor({ projectId }: { projectId: string }) {
  const charter = useCharter(projectId);
  const save = useSaveCharter(projectId);
  const [draft, setDraft] = useState<CharterContent | null>(null);

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
      title="项目章程"
      subtitle={
        charter.data?.created
          ? "对应 PMBOK 8「启动」焦点领域 · 首次访问已生成空白章程，S4 接入 AI 逐节起草"
          : `对应 PMBOK 8「启动」焦点领域 · 最近保存 ${
              charter.data?.updatedAt.slice(0, 16).replace("T", " ") ?? "—"
            }`
      }
      actions={
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
      }
    >
      {charter.isLoading || draft === null ? (
        <div className="py-4 text-center text-xs text-slate-400">加载中…</div>
      ) : (
        <div className="grid grid-cols-2 gap-3">
          {SECTIONS.map((s) => (
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
    </Card>
  );
}
