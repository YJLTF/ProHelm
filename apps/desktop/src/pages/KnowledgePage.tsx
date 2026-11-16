import { useEffect, useMemo, useRef, useState } from "react";
import { useSearchParams } from "react-router";
import { BookOpen, Compass, Search } from "lucide-react";

import { Badge } from "../components/ui/Badge";
import { EmptyState } from "../components/ui/EmptyState";
import { Input } from "../components/ui/Input";
import {
  KB_CATEGORIES,
  KB_TERMS,
  kbTermById,
  searchKbTerms,
} from "../data/kbTerms";
import type { KbCategoryKey, KbTerm } from "../data/kbTerms";
import { cn } from "../lib/format";

/** 高亮状态的自动清除时长（毫秒）。 */
const HIGHLIGHT_MS = 2600;

export function KnowledgePage() {
  const [params, setParams] = useSearchParams();
  const activeTerm = params.get("term");
  const [category, setCategory] = useState<KbCategoryKey | "all">("all");
  const [query, setQuery] = useState("");
  const [highlightId, setHighlightId] = useState<string | null>(null);
  const cardRefs = useRef<Map<string, HTMLElement>>(new Map());

  const terms = useMemo(() => {
    const hit = searchKbTerms(query);
    return category === "all" ? hit : hit.filter((t) => t.category === category);
  }, [query, category]);

  // 从工具界面跳转进来：定位并高亮目标词条
  useEffect(() => {
    if (!activeTerm) return;
    const t = kbTermById.get(activeTerm);
    if (t) {
      setCategory("all");
      setQuery("");
    }
    const el = cardRefs.current.get(activeTerm);
    if (el) {
      el.scrollIntoView({ behavior: "smooth", block: "start" });
      setHighlightId(activeTerm);
      const timer = window.setTimeout(() => setHighlightId(null), HIGHLIGHT_MS);
      return () => window.clearTimeout(timer);
    }
  }, [activeTerm]);

  const jumpTo = (id: string) => setParams({ term: id });

  return (
    <div className="mx-auto max-w-5xl px-8 py-6">
      <header className="mb-4">
        <h1 className="flex items-center gap-2 text-lg font-bold text-slate-900">
          <BookOpen size={18} className="text-blue-600" /> PMP 术语速查
        </h1>
        <p className="mt-0.5 text-xs text-slate-500">
          工具中遇到的名词与方法不理解？虚线下划线的词悬停即有一句话释义，点击进入详解。
          共 {KB_TERMS.length} 条 · 自研讲解，持续扩充
        </p>
      </header>

      <div className="mb-4 flex flex-col gap-3">
        <div className="relative w-full max-w-md">
          <Search
            size={14}
            className="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-slate-400"
          />
          <Input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="搜索术语、缩写或概念（如 挣值、WBS、SPI）"
            className="pl-8"
          />
        </div>
        <div className="flex flex-wrap gap-1.5">
          <CategoryChip
            active={category === "all"}
            onClick={() => setCategory("all")}
            label={`全部 (${KB_TERMS.length})`}
          />
          {KB_CATEGORIES.map((c) => (
            <CategoryChip
              key={c.key}
              active={category === c.key}
              onClick={() => setCategory(c.key)}
              label={`${c.name} (${KB_TERMS.filter((t) => t.category === c.key).length})`}
              title={c.desc}
            />
          ))}
        </div>
      </div>

      {terms.length === 0 ? (
        <EmptyState
          icon={<Search size={30} />}
          title="没有匹配的术语"
          description="换个关键词试试，或到舵手（右下角）直接提问——知识库持续扩充中。"
        />
      ) : (
        <div className="grid grid-cols-2 gap-3">
          {terms.map((t) => (
            <TermCard
              key={t.id}
              term={t}
              highlighted={highlightId === t.id}
              onJump={jumpTo}
              refFn={(el) => {
                if (el) cardRefs.current.set(t.id, el);
                else cardRefs.current.delete(t.id);
              }}
            />
          ))}
        </div>
      )}
    </div>
  );
}

function CategoryChip({
  active,
  onClick,
  label,
  title,
}: {
  active: boolean;
  onClick: () => void;
  label: string;
  title?: string;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      title={title}
      className={cn(
        "rounded-full border px-3 py-1 text-xs font-medium transition",
        active
          ? "border-blue-600 bg-blue-600 text-white"
          : "border-slate-200 bg-white text-slate-600 hover:border-slate-300 hover:text-slate-800",
      )}
    >
      {label}
    </button>
  );
}

function TermCard({
  term,
  highlighted,
  onJump,
  refFn,
}: {
  term: KbTerm;
  highlighted: boolean;
  onJump: (id: string) => void;
  refFn: (el: HTMLElement | null) => void;
}) {
  const category = KB_CATEGORIES.find((c) => c.key === term.category);
  return (
    <article
      ref={refFn}
      className={cn(
        "scroll-mt-4 rounded-xl border bg-white p-4 shadow-sm transition",
        highlighted
          ? "border-blue-500 ring-2 ring-blue-200"
          : "border-slate-200",
      )}
    >
      <div className="flex items-start justify-between gap-2">
        <div className="min-w-0">
          <div className="flex items-baseline gap-1.5">
            <span className="text-sm font-bold text-slate-800">{term.term}</span>
            {term.abbr && (
              <span className="truncate text-xs text-slate-400">{term.abbr}</span>
            )}
          </div>
        </div>
        <Badge tone="blue">{category?.name}</Badge>
      </div>

      <p className="mt-2 text-xs font-medium leading-relaxed text-slate-700">
        {term.summary}
      </p>
      <div className="mt-2 space-y-1.5">
        {term.detail.map((p, i) => (
          <p key={i} className="text-xs leading-relaxed text-slate-500">
            {p}
          </p>
        ))}
      </div>

      <div className="mt-3 flex items-start gap-1.5 rounded-lg border border-blue-100 bg-blue-50/60 px-2.5 py-2">
        <Compass size={12} className="mt-0.5 shrink-0 text-blue-500" />
        <p className="text-[11px] leading-relaxed text-blue-700">
          <span className="font-semibold">在 ProHelm 中：</span>
          {term.inProHelm}
        </p>
      </div>

      {term.related.length > 0 && (
        <div className="mt-2.5 flex flex-wrap items-center gap-1.5">
          <span className="text-[10px] text-slate-400">相关：</span>
          {term.related.map((rid) => {
            const rt = kbTermById.get(rid);
            if (!rt) return null;
            return (
              <button
                key={rid}
                type="button"
                onClick={() => onJump(rid)}
                className="rounded-full border border-slate-200 bg-slate-50 px-2 py-0.5 text-[11px] text-slate-600 transition hover:border-blue-300 hover:text-blue-700"
              >
                {rt.term}
              </button>
            );
          })}
        </div>
      )}
    </article>
  );
}
