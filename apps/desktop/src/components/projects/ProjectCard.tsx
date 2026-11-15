import { useState } from "react";
import { Link } from "react-router";
import { Pencil } from "lucide-react";

import { ProjectEditDialog } from "./ProjectEditDialog";
import { Badge } from "../ui/Badge";
import { formatBudget, formatDate, statusTone } from "../../lib/format";
import { PROJECT_METHOD_LABEL, PROJECT_STATUS_LABEL } from "../../types";
import type { Project } from "../../types";

export function ProjectCard({ project }: { project: Project }) {
  const [editOpen, setEditOpen] = useState(false);
  return (
    <>
      <Link
        to={`/projects/${project.id}`}
        className="block rounded-xl border border-slate-200 bg-white p-4 shadow-sm transition hover:border-blue-300 hover:shadow-md"
      >
        <div className="flex items-start justify-between gap-2">
          <div className="min-w-0">
            <div className="truncate text-sm font-semibold text-slate-800">{project.name}</div>
            <div className="mt-0.5 text-xs text-slate-400">{project.code || "未编号"}</div>
          </div>
          <div className="flex shrink-0 items-center gap-1.5">
            <Badge tone={statusTone(project.status)}>{PROJECT_STATUS_LABEL[project.status]}</Badge>
            <button
              aria-label="编辑项目信息"
              title="编辑项目信息与状态"
              onClick={(e) => {
                e.preventDefault();
                e.stopPropagation();
                setEditOpen(true);
              }}
              className="rounded p-1 text-slate-300 transition hover:bg-slate-100 hover:text-slate-600"
            >
              <Pencil size={13} />
            </button>
          </div>
        </div>
        <div className="mt-3 flex flex-wrap items-center gap-1.5">
          <Badge tone="blue">{PROJECT_METHOD_LABEL[project.method]}</Badge>
          <span className="text-xs text-slate-500">
            {formatDate(project.startDate)} → {formatDate(project.targetEndDate)}
          </span>
        </div>
        <div className="mt-1.5 text-xs text-slate-500">{formatBudget(project.bac)}</div>
      </Link>

      <ProjectEditDialog project={project} open={editOpen} onClose={() => setEditOpen(false)} />
    </>
  );
}
