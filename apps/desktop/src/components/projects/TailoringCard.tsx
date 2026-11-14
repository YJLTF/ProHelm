import { useEffect, useState } from "react";

import { Button } from "../ui/Button";
import { Card } from "../ui/Card";
import { useApplyRecommendedTailoring, useSetTailored } from "../../hooks/useProjects";
import { cn } from "../../lib/format";
import { toast } from "../../store/toast";
import { TAILORING_GROUPS } from "../../data/templates";
import { PROJECT_METHOD_LABEL } from "../../types";
import type { Project } from "../../types";

export function TailoringCard({ project }: { project: Project }) {
  const setTailored = useSetTailored(project.id);
  const applyRecommended = useApplyRecommendedTailoring(project.id);
  const [selected, setSelected] = useState<string[]>(project.tailored ?? []);

  useEffect(() => {
    setSelected(project.tailored ?? []);
  }, [project.tailored]);

  const dirty =
    JSON.stringify([...selected].sort()) !==
    JSON.stringify([...(project.tailored ?? [])].sort());

  const toggle = (key: string) =>
    setSelected((s) => (s.includes(key) ? s.filter((k) => k !== key) : [...s, key]));

  return (
    <Card
      title="方法论裁剪器"
      subtitle={`按「${PROJECT_METHOD_LABEL[project.method]}」预置 · PMBOK 8 过程非规范性、可裁剪`}
      actions={
        <>
          <Button
            size="sm"
            variant="ghost"
            disabled={applyRecommended.isPending}
            onClick={() =>
              applyRecommended.mutate(undefined, {
                onSuccess: () => toast("已重置为推荐裁剪"),
              })
            }
          >
            重置推荐
          </Button>
          <Button
            size="sm"
            disabled={!dirty || setTailored.isPending}
            onClick={() =>
              setTailored.mutate(selected, { onSuccess: () => toast("裁剪已保存") })
            }
          >
            保存裁剪
          </Button>
        </>
      }
    >
      {TAILORING_GROUPS.map((group) => (
        <div key={group.focus} className="mb-3 last:mb-0">
          <div className="mb-1.5 text-xs font-semibold text-slate-500">{group.focus}</div>
          <div className="flex flex-wrap gap-1.5">
            {group.items.map((item) => (
              <button
                key={item.key}
                onClick={() => toggle(item.key)}
                className={cn(
                  "rounded-full border px-2.5 py-1 text-xs font-medium transition-colors",
                  selected.includes(item.key)
                    ? "border-blue-600 bg-blue-600 text-white"
                    : "border-slate-300 bg-white text-slate-500 hover:border-slate-400 hover:text-slate-700",
                )}
              >
                {item.label}
              </button>
            ))}
          </div>
        </div>
      ))}
      {project.tailored === null && (
        <p className="mt-2 text-xs text-slate-400">
          该项目尚未定制裁剪——点击「重置推荐」按开发方法自动生成最小必要流程集。
        </p>
      )}
    </Card>
  );
}
