import { FileText, Users } from "lucide-react";

import { Card } from "../../components/ui/Card";
import { EmptyState } from "../../components/ui/EmptyState";
import type { Project } from "../../types";

export function InitiationTab({ project }: { project: Project }) {
  return (
    <div className="grid grid-cols-2 gap-4">
      <Card
        title="项目章程（草案）"
        subtitle="对应 PMBOK 8「启动」焦点领域 · S3 上线结构化编辑器，S4 上线 AI 起草"
      >
        {project.description ? (
          <div className="rounded-lg bg-slate-50 p-3 text-sm leading-relaxed text-slate-700">
            {project.description}
          </div>
        ) : (
          <EmptyState
            icon={<FileText size={26} />}
            title="章程待起草"
            description="章程包含：项目目标、成功标准、关键里程碑、主要风险与制约、项目经理职权。"
          />
        )}
      </Card>

      <Card title="干系人登记册" subtitle="S3 上线 · 权力/利益方格将一并呈现">
        <EmptyState
          icon={<Users size={26} />}
          title="干系人管理即将上线"
          description="登记干系人的期望与影响力，自动生成沟通矩阵草稿。"
        />
      </Card>
    </div>
  );
}
