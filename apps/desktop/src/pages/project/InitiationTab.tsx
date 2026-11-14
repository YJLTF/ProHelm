import { Users } from "lucide-react";

import { CharterEditor } from "../../components/charter/CharterEditor";
import { StakeholderSection } from "../../components/stakeholders/StakeholderSection";
import { TailoringCard } from "../../components/projects/TailoringCard";
import type { Project } from "../../types";

/** 启动页签 = PMBOK 8「启动」焦点领域：章程 → 干系人 → 方法论裁剪 */
export function InitiationTab({ project }: { project: Project }) {
  return (
    <div className="space-y-4">
      <CharterEditor projectId={project.id} />
      <StakeholderSection projectId={project.id} />
      <TailoringCard project={project} />
      <p className="flex items-center gap-1.5 text-[11px] text-slate-400">
        <Users size={12} />
        章程与干系人信息是 S4 AI 起草（风险识别、沟通计划）的主要上下文来源。
      </p>
    </div>
  );
}
