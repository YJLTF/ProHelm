import type { ReactNode } from "react";

interface Props {
  icon?: ReactNode;
  title: string;
  description?: string;
  action?: ReactNode;
}

export function EmptyState({ icon, title, description, action }: Props) {
  return (
    <div className="flex flex-col items-center justify-center rounded-xl border border-dashed border-slate-300 bg-white/60 px-6 py-10 text-center">
      {icon && <div className="mb-2 text-slate-300">{icon}</div>}
      <div className="text-sm font-medium text-slate-700">{title}</div>
      {description && <div className="mt-1 max-w-sm text-xs text-slate-500">{description}</div>}
      {action && <div className="mt-3">{action}</div>}
    </div>
  );
}
