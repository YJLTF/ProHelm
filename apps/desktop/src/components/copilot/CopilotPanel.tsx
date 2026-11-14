import { useEffect, useRef, useState } from "react";
import { useMatch } from "react-router";
import { Channel } from "@tauri-apps/api/core";
import { Send, ShipWheel, X } from "lucide-react";

import { api } from "../../lib/ipc";
import { cn } from "../../lib/format";
import { toastError } from "../../store/toast";
import { useUiStore } from "../../store/ui";

interface ChatMsg {
  role: "user" | "assistant";
  content: string;
  streaming?: boolean;
}

const SAMPLE_QUESTIONS = [
  "什么是关键路径？怎么保护它？",
  "SPI 和 CPI 怎么算？低于 1 怎么办？",
  "风险和问题的区别是什么？",
  "项目启动会要讲哪些内容？",
];

export function CopilotPanel() {
  const open = useUiStore((s) => s.copilotOpen);
  const setOpen = useUiStore((s) => s.setCopilotOpen);
  const match = useMatch("/projects/:id");
  const projectId = match?.params.id ?? null;

  const [messages, setMessages] = useState<ChatMsg[]>([]);
  const [input, setInput] = useState("");
  const [busy, setBusy] = useState(false);
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    scrollRef.current?.scrollTo({ top: scrollRef.current.scrollHeight });
  }, [messages, open]);

  const ask = async () => {
    const question = input.trim();
    if (!question || busy) return;
    setBusy(true);
    setInput("");
    setMessages((m) => [...m, { role: "user", content: question }, { role: "assistant", content: "", streaming: true }]);

    const channel = new Channel<{ event: "delta" | "done"; text?: string }>();
    channel.onmessage = (msg) => {
      if (msg.event === "delta" && msg.text) {
        setMessages((m) => {
          const next = [...m];
          const last = next[next.length - 1];
          if (last?.role === "assistant") {
            next[next.length - 1] = { ...last, content: last.content + msg.text };
          }
          return next;
        });
      } else if (msg.event === "done") {
        setMessages((m) => {
          const next = [...m];
          const last = next[next.length - 1];
          if (last?.role === "assistant") {
            next[next.length - 1] = { ...last, streaming: false };
          }
          return next;
        });
        setBusy(false);
      }
    };

    try {
      await api.coachAsk(channel, question, projectId);
    } catch (e) {
      toastError((e as { message?: string }).message ?? "AI 调用失败");
      setMessages((m) => {
        const next = [...m];
        const last = next[next.length - 1];
        if (last?.role === "assistant" && last.content === "") {
          next[next.length - 1] = { role: "assistant", content: "（调用失败：请检查「设置 → AI 模型」配置与网络）" };
        } else if (last) {
          next[next.length - 1] = { ...last, streaming: false };
        }
        return next;
      });
      setBusy(false);
    }
  };

  return (
    <aside
      className={cn(
        "fixed inset-y-0 right-0 z-40 flex w-96 flex-col border-l border-slate-200 bg-white shadow-xl transition-transform duration-200",
        open ? "translate-x-0" : "translate-x-full",
      )}
    >
      <header className="flex items-center justify-between border-b border-slate-100 px-4 py-3">
        <div className="flex items-center gap-2">
          <div className="flex h-7 w-7 items-center justify-center rounded-lg bg-blue-600 text-white">
            <ShipWheel size={15} />
          </div>
          <div>
            <div className="text-sm font-bold text-slate-800">舵手 · PMP 教练</div>
            <div className="text-[10px] text-slate-400">
              {projectId ? "已附上当前项目上下文" : "全局问答 · BYOK 直连模型"}
            </div>
          </div>
        </div>
        <button
          onClick={() => setOpen(false)}
          aria-label="关闭"
          className="rounded-md px-1.5 py-0.5 text-slate-400 hover:bg-slate-100 hover:text-slate-600"
        >
          <X size={16} />
        </button>
      </header>

      <div ref={scrollRef} className="flex-1 space-y-3 overflow-y-auto px-4 py-3">
        {messages.length === 0 && (
          <div className="space-y-2 pt-6">
            <p className="text-center text-xs text-slate-400">
              试试这些问题（答案基于内置 PMP 知识库，可溯源）：
            </p>
            {SAMPLE_QUESTIONS.map((q) => (
              <button
                key={q}
                onClick={() => setInput(q)}
                className="block w-full rounded-lg border border-slate-200 px-3 py-2 text-left text-xs text-slate-600 hover:border-blue-300 hover:bg-blue-50"
              >
                {q}
              </button>
            ))}
          </div>
        )}
        {messages.map((m, i) => (
          <div
            key={i}
            className={cn(
              "max-w-[85%] rounded-xl px-3 py-2 text-sm leading-relaxed whitespace-pre-wrap",
              m.role === "user"
                ? "ml-auto bg-blue-600 text-white"
                : "bg-slate-100 text-slate-700",
            )}
          >
            {m.content}
            {m.streaming && m.content === "" && <span className="text-slate-400">思考中…</span>}
            {m.streaming && m.content !== "" && (
              <span className="ml-0.5 inline-block h-3.5 w-1.5 animate-pulse bg-slate-400 align-middle" />
            )}
          </div>
        ))}
      </div>

      <footer className="border-t border-slate-100 p-3">
        <div className="flex items-end gap-2">
          <textarea
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter" && !e.shiftKey) {
                e.preventDefault();
                void ask();
              }
            }}
            rows={2}
            placeholder="向 PMP 教练提问…（Enter 发送，Shift+Enter 换行）"
            className="w-full resize-none rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-100"
            disabled={busy}
          />
          <button
            onClick={() => void ask()}
            disabled={busy || !input.trim()}
            aria-label="发送"
            className="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-blue-600 text-white hover:bg-blue-700 disabled:opacity-50"
          >
            <Send size={15} />
          </button>
        </div>
      </footer>
    </aside>
  );
}
