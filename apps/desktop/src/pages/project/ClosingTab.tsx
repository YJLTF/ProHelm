import { Archive, ClipboardCheck, Lightbulb } from "lucide-react";

import { Badge } from "../../components/ui/Badge";
import { Card } from "../../components/ui/Card";
import type { Project } from "../../types";

const CHECKLIST = [
  { label: "交付物验收清单（对照章程成功标准）", sprint: "S3" },
  { label: "合同 / 财务收尾（结算对照预算）", sprint: "S5" },
  { label: "经验教训登记册（AI 辅助复盘起草）", sprint: "S4" },
  { label: "项目归档与模板化沉淀", sprint: "S5" },
];

export function ClosingTab({ project }: { project: Project }) {
  return (
    <Card
      title="收尾清单"
      subtitle={`对应 PMBOK 8「收尾」焦点领域 · ${project.name} 关闭前需逐项确认`}
    >
      <div className="space-y-2">
        {CHECKLIST.map((item) => (
          <div
            key={item.label}
            className="flex items-center justify-between rounded-lg border border-slate-100 bg-white px-3 py-2.5"
          >
            <div className="flex items-center gap-2.5 text-sm text-slate-600">
              {item.label.includes("验收") ? (
                <ClipboardCheck size={15} className="text-slate-300" />
              ) : item.label.includes("经验") ? (
                <Lightbulb size={15} className="text-slate-300" />
              ) : (
                <Archive size={15} className="text-slate-300" />
              )}
              {item.label}
            </div>
            <Badge tone="gray">{item.sprint} 上线</Badge>
          </div>
        ))}
      </div>
    </Card>
  );
}
