import type { ReactNode } from "react";
import { useNavigate } from "react-router";

import { kbTermById } from "../../data/kbTerms";

/**
 * 术语引导：虚线下划线 + hover 一句话释义卡片，点击跳转知识库详解。
 * 用法：<Term id="wbs">WBS</Term>（children 省略时显示术语名）。
 */
export function Term({ id, children }: { id: string; children?: ReactNode }) {
  const navigate = useNavigate();
  const t = kbTermById.get(id);
  if (!t) {
    return <>{children ?? id}</>;
  }
  return (
    <span className="group relative inline-block">
      <button
        type="button"
        onClick={(e) => {
          e.stopPropagation();
          e.preventDefault();
          navigate(`/kb?term=${t.id}`);
        }}
        title={t.term}
        className="inline-flex items-baseline gap-0.5 border-b border-dashed border-slate-400/70 text-inherit transition hover:border-blue-500 hover:text-blue-700"
      >
        {children ?? t.term}
        <span
          aria-hidden
          className="inline-block h-3 w-3 shrink-0 translate-y-0.5 rounded-full bg-slate-200 text-center text-[9px] font-bold leading-3 text-slate-500 group-hover:bg-blue-100 group-hover:text-blue-700"
        >
          ?
        </span>
      </button>
      <span
        aria-hidden
        className="pointer-events-none invisible absolute bottom-full left-1/2 z-50 mb-1.5 w-72 -translate-x-1/2 rounded-lg bg-slate-800 p-3 text-left text-xs leading-relaxed text-slate-100 opacity-0 shadow-xl transition group-hover:visible group-hover:opacity-100"
      >
        <span className="block font-semibold text-white">
          {t.term}
          {t.abbr ? ` · ${t.abbr}` : ""}
        </span>
        <span className="mt-1 block text-slate-300">{t.summary}</span>
        <span className="mt-1.5 block text-[10px] text-blue-300">点击查看详解 →</span>
      </span>
    </span>
  );
}
