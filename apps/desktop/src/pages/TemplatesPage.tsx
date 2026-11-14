import { LayoutTemplate } from "lucide-react";

import { Badge } from "../components/ui/Badge";
import { Card } from "../components/ui/Card";

interface TemplateDef {
  name: string;
  desc: string;
  tag: "敏捷" | "预测型" | "混合" | "工件";
}

const TEMPLATES: TemplateDef[] = [
  { name: "软件交付 · 敏捷模板", desc: "迭代管理 + 最小流程集（章程/风险/评审）", tag: "敏捷" },
  { name: "软件交付 · 预测型模板", desc: "WBS 分解 + 甘特排期 + 变更控制", tag: "预测型" },
  { name: "乙方咨询交付", desc: "里程碑验收 + 干系人管理 + 沟通矩阵", tag: "预测型" },
  { name: "设计创意项目", desc: "评审节点 + 素材清单 + 客户确认", tag: "混合" },
  { name: "项目章程", desc: "PMBOK 工件：目标/成功标准/里程碑/职权", tag: "工件" },
  { name: "干系人登记册", desc: "PMBOK 工件：期望/影响力/管理策略", tag: "工件" },
  { name: "风险登记册", desc: "PMBOK 工件：概率×影响/应对策略/责任人", tag: "工件" },
  { name: "RACI 责任矩阵", desc: "PMBOK 工件：谁负责/谁批准/咨询/知情", tag: "工件" },
  { name: "变更请求单", desc: "PMBOK 工件：影响分析/审批流/留痕", tag: "工件" },
  { name: "经验教训登记册", desc: "PMBOK 工件：做得好/做得差/建议", tag: "工件" },
];

const TAG_TONE = {
  敏捷: "green",
  预测型: "blue",
  混合: "amber",
  工件: "gray",
} as const;

export function TemplatesPage() {
  return (
    <div className="mx-auto max-w-6xl px-8 py-6">
      <header className="mb-5">
        <h1 className="flex items-center gap-2 text-lg font-bold text-slate-900">
          <LayoutTemplate size={18} className="text-blue-600" /> 模板与工件库
        </h1>
        <p className="mt-0.5 text-xs text-slate-500">
          按开发方法裁剪的最小流程集 + PMBOK 工件模板（自研撰写）· S3 上线后可直接从模板创建项目
        </p>
      </header>

      <div className="grid grid-cols-3 gap-3">
        {TEMPLATES.map((t) => (
          <Card key={t.name} className="opacity-90">
            <div className="flex items-start justify-between gap-2">
              <div className="text-sm font-semibold text-slate-800">{t.name}</div>
              <Badge tone={TAG_TONE[t.tag]}>{t.tag}</Badge>
            </div>
            <div className="mt-1.5 text-xs text-slate-500">{t.desc}</div>
            <div className="mt-3">
              <Badge tone="blue">S3 上线</Badge>
            </div>
          </Card>
        ))}
      </div>
    </div>
  );
}
