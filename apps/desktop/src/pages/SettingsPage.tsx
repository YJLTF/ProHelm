import { useEffect, useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { open, save } from "@tauri-apps/plugin-dialog";
import { Bot, Database, Info, ShieldCheck } from "lucide-react";

import { Badge } from "../components/ui/Badge";
import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import { Input, Label, Select } from "../components/ui/Input";
import { api } from "../lib/ipc";
import { toast, toastError } from "../store/toast";
import { AI_KIND_LABEL, AI_STATUS_LABEL } from "../types";
import type { AiArtifactStatusType } from "../types";

const AI_PRESETS = [
  { key: "glm", label: "智谱 GLM", baseUrl: "https://open.bigmodel.cn/api/paas/v4", model: "glm-4-flash" },
  { key: "deepseek", label: "DeepSeek", baseUrl: "https://api.deepseek.com/v1", model: "deepseek-chat" },
  { key: "qwen", label: "通义千问", baseUrl: "https://dashscope.aliyuncs.com/compatible-mode/v1", model: "qwen-plus" },
  { key: "openai", label: "OpenAI", baseUrl: "https://api.openai.com/v1", model: "gpt-4o-mini" },
  { key: "openrouter", label: "OpenRouter", baseUrl: "https://openrouter.ai/api/v1", model: "openai/gpt-4o-mini" },
  { key: "ollama", label: "Ollama（本地）", baseUrl: "http://localhost:11434/v1", model: "qwen2.5:7b" },
];

export function SettingsPage() {
  const qc = useQueryClient();
  const settings = useQuery({ queryKey: ["ai-settings"], queryFn: api.getAiSettings });
  const artifacts = useQuery({ queryKey: ["ai-artifacts"], queryFn: () => api.listAiArtifacts(null) });

  const [presetKey, setPresetKey] = useState("");
  const [baseUrl, setBaseUrl] = useState("");
  const [model, setModel] = useState("");
  const [apiKey, setApiKey] = useState("");
  const [saving, setSaving] = useState(false);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<{ ok: boolean; text: string } | null>(null);

  useEffect(() => {
    if (settings.data) {
      setBaseUrl(settings.data.baseUrl ?? "");
      setModel(settings.data.model ?? "");
    }
  }, [settings.data]);

  const applyPreset = (key: string) => {
    setPresetKey(key);
    const preset = AI_PRESETS.find((p) => p.key === key);
    if (preset) {
      setBaseUrl(preset.baseUrl);
      setModel(preset.model);
    }
  };

  const save = async () => {
    setSaving(true);
    setTestResult(null);
    try {
      await api.setAiSettings(baseUrl, model, apiKey ? apiKey : null);
      void qc.invalidateQueries({ queryKey: ["ai-settings"] });
      setApiKey("");
      toast("AI 设置已保存（Key 已写入系统钥匙串）");
    } catch (e) {
      toastError((e as { message?: string }).message ?? "保存失败");
    } finally {
      setSaving(false);
    }
  };

  const test = async () => {
    setTesting(true);
    setTestResult(null);
    try {
      const reply = await api.testAiConnection();
      setTestResult({ ok: true, text: reply });
    } catch (e) {
      setTestResult({ ok: false, text: (e as { message?: string }).message ?? "连接失败" });
    } finally {
      setTesting(false);
    }
  };

  const dirty =
    baseUrl !== (settings.data?.baseUrl ?? "") ||
    model !== (settings.data?.model ?? "") ||
    apiKey.length > 0;

  return (
    <div className="mx-auto max-w-3xl px-8 py-6">
      <header className="mb-5">
        <h1 className="text-lg font-bold text-slate-900">设置</h1>
        <p className="mt-0.5 text-xs text-slate-500">AI 模型、数据与隐私均在本地管理</p>
      </header>

      <div className="space-y-4">
        <Card
          title={
            <span className="flex items-center gap-2">
              <Bot size={15} className="text-blue-600" /> AI 模型（舵手 · BYOK）
            </span>
          }
          subtitle="自带 API Key，直连模型供应商；Key 存系统钥匙串，不经过任何第三方服务器"
        >
          <div className="space-y-3">
            <div>
              <Label>供应商预设</Label>
              <Select
                value={presetKey}
                onChange={(e) => applyPreset(e.target.value)}
              >
                <option value="">— 自定义 —</option>
                {AI_PRESETS.map((p) => (
                  <option key={p.key} value={p.key}>
                    {p.label}
                  </option>
                ))}
              </Select>
            </div>
            <div>
              <Label>接口地址（OpenAI 兼容）</Label>
              <Input
                value={baseUrl}
                onChange={(e) => setBaseUrl(e.target.value)}
                placeholder="https://api.example.com/v1"
              />
            </div>
            <div>
              <Label>模型名称</Label>
              <Input
                value={model}
                onChange={(e) => setModel(e.target.value)}
                placeholder="例如：glm-4-flash"
              />
            </div>
            <div>
              <Label>API Key</Label>
              <Input
                type="password"
                value={apiKey}
                onChange={(e) => setApiKey(e.target.value)}
                placeholder={
                  settings.data?.hasApiKey
                    ? "已配置（写入系统钥匙串）——留空则不修改"
                    : "粘贴 Key，保存后写入系统钥匙串"
                }
              />
            </div>
            <div className="flex items-center gap-2">
              <Button onClick={() => void save()} disabled={saving || !dirty || !baseUrl.trim() || !model.trim()}>
                {saving ? "保存中…" : "保存设置"}
              </Button>
              <Button variant="secondary" onClick={() => void test()} disabled={testing}>
                {testing ? "测试中…" : "测试连接"}
              </Button>
              {settings.data?.hasApiKey && <Badge tone="green">Key 已配置</Badge>}
            </div>
            {testResult && (
              <div
                className={`rounded-lg border p-3 text-xs ${
                  testResult.ok
                    ? "border-emerald-100 bg-emerald-50 text-emerald-700"
                    : "border-red-100 bg-red-50 text-red-600"
                }`}
              >
                {testResult.ok ? `连接成功，模型回复：${testResult.text}` : `连接失败：${testResult.text}`}
              </div>
            )}
            <p className="text-xs leading-relaxed text-slate-500">
              隐私承诺：默认脱敏外发（项目名/人名/客户名→代号，S5 完善规则配置）；外发范围可配置；一切
              AI 写入先预览、可拒绝、可撤销。当前仅支持 OpenAI 兼容协议（覆盖 GLM/DeepSeek/Qwen/
              OpenRouter/Ollama 等），Anthropic 原生协议 S5 接入。
            </p>
          </div>
        </Card>

        <Card
          title={
            <span className="flex items-center gap-2">
              <Database size={15} className="text-blue-600" /> 数据与备份
            </span>
          }
          subtitle="本地优先 · 数据不出设备 · 备份为 SQLite 单文件（VACUUM INTO）"
        >
          <DataSection />
        </Card>

        <Card
          title="AI 使用记录（溯源）"
          subtitle="每一次 AI 生成（问答/章程/WBS）都记录在本地，采纳与弃用留痕"
        >
          {artifacts.isLoading ? (
            <div className="py-2 text-center text-xs text-slate-400">加载中…</div>
          ) : (artifacts.data?.length ?? 0) === 0 ? (
            <div className="py-2 text-center text-xs text-slate-400">还没有 AI 生成记录</div>
          ) : (
            <div className="max-h-64 divide-y divide-slate-50 overflow-y-auto">
              {artifacts.data!.slice(0, 20).map((a) => (
                <div key={a.id} className="flex items-center gap-2 py-2 text-xs">
                  <Badge tone={a.kind === "coach" ? "blue" : "amber"}>{AI_KIND_LABEL[a.kind]}</Badge>
                  <span className="min-w-0 flex-1 truncate text-slate-600">{a.inputSummary}</span>
                  <span className="shrink-0 font-mono text-[10px] text-slate-400">{a.model}</span>
                  <Badge tone={a.status === "adopted" ? "green" : a.status === "discarded" ? "gray" : "amber"}>
                    {AI_STATUS_LABEL[a.status]}
                  </Badge>
                  {a.status === "draft" && (
                    <button
                      onClick={() => {
                        void api
                          .setArtifactStatus(a.id, "discarded" satisfies AiArtifactStatusType)
                          .then(() => qc.invalidateQueries({ queryKey: ["ai-artifacts"] }));
                      }}
                      className="shrink-0 text-slate-300 hover:text-red-500"
                      title="标记为弃用"
                    >
                      弃用
                    </button>
                  )}
                </div>
              ))}
            </div>
          )}
        </Card>

        <TelemetryCard />

        <Card
          title={
            <span className="flex items-center gap-2">
              <Info size={15} className="text-blue-600" /> 关于
            </span>
          }
        >
          <div className="space-y-2 text-sm text-slate-600">
            <p className="flex items-center gap-2">
              <ShieldCheck size={15} className="text-emerald-500" />
              ProHelm（项舵）v0.1.0-alpha · 基于 PMP 体系（PMBOK 8）的项目管理桌面应用
            </p>
            <p className="text-xs leading-relaxed text-slate-500">
              方法论基线：PMBOK 第八版「5 焦点领域 + 7 绩效域」；经典工具（WBS / 关键路径 /
              挣值管理）作为功能落地。PMP 教练知识库为自研撰写，仅引用公开方法论框架概念。
            </p>
          </div>
        </Card>
      </div>
    </div>
  );
}

function DataSection() {
  const qc = useQueryClient();
  const backups = useQuery({ queryKey: ["backups"], queryFn: api.listBackups });
  const autoEnabled = useQuery({
    queryKey: ["auto-backup-enabled"],
    queryFn: api.getAutoBackupEnabled,
  });
  const [busy, setBusy] = useState(false);

  const toggleAuto = async (enabled: boolean) => {
    try {
      await api.setAutoBackupEnabled(enabled);
      void qc.invalidateQueries({ queryKey: ["auto-backup-enabled"] });
      toast(enabled ? "已开启每日自动快照" : "已关闭每日自动快照");
    } catch (e) {
      toastError((e as { message?: string }).message ?? "设置失败");
    }
  };

  const create = async () => {
    setBusy(true);
    try {
      const info = await api.createBackup();
      void qc.invalidateQueries({ queryKey: ["backups"] });
      toast(`备份完成：${info.fileName}`);
    } catch (e) {
      toastError((e as { message?: string }).message ?? "备份失败");
    } finally {
      setBusy(false);
    }
  };

  const restore = async (fileName: string) => {
    if (!window.confirm(`恢复备份「${fileName}」将覆盖当前全部数据（恢复前会自动再备份一次）。继续？`)) {
      return;
    }
    setBusy(true);
    try {
      await api.restoreBackup(fileName);
      void qc.invalidateQueries();
      toast("已恢复备份并刷新数据");
    } catch (e) {
      toastError((e as { message?: string }).message ?? "恢复失败");
    } finally {
      setBusy(false);
    }
  };

  const remove = async (fileName: string) => {
    if (!window.confirm(`删除备份「${fileName}」？`)) return;
    try {
      await api.deleteBackup(fileName);
      void qc.invalidateQueries({ queryKey: ["backups"] });
    } catch (e) {
      toastError((e as { message?: string }).message ?? "删除失败");
    }
  };

  const exportAll = async () => {
    try {
      const path = await save({
        defaultPath: `prohelm-export-${new Date().toISOString().slice(0, 10)}.json`,
        filters: [{ name: "ProHelm 数据包", extensions: ["json"] }],
      });
      if (!path) return;
      await api.exportSnapshotTo(path);
      toast("全量数据包已导出（不含 AI 溯源与本机设置）");
    } catch (e) {
      toastError((e as { message?: string }).message ?? "导出失败");
    }
  };

  const importAll = async () => {
    try {
      const path = await open({
        multiple: false,
        filters: [{ name: "ProHelm 数据包", extensions: ["json"] }],
      });
      if (!path) return;
      if (
        !window.confirm(
          "导入将覆盖当前全部项目数据（导入前会自动备份当前库）。继续？",
        )
      ) {
        return;
      }
      await api.createBackup();
      const stats = await api.importSnapshotFrom(path as string);
      void qc.invalidateQueries();
      toast(
        `导入完成：${stats.projects} 项目 / ${stats.tasks} 任务 / ${stats.risks} 风险 / ${stats.stakeholders} 干系人`,
      );
    } catch (e) {
      toastError((e as { message?: string }).message ?? "导入失败");
    }
  };

  return (
    <div className="space-y-3 text-sm text-slate-600">
      <label className="flex cursor-pointer items-center gap-2.5 rounded-lg border border-slate-100 bg-slate-50/60 px-3 py-2.5">
        <input
          type="checkbox"
          className="h-4 w-4 accent-blue-600"
          checked={autoEnabled.data ?? true}
          disabled={autoEnabled.isLoading}
          onChange={(e) => void toggleAuto(e.target.checked)}
        />
        <span className="min-w-0 flex-1">
          <span className="block text-xs font-medium text-slate-700">每日自动快照</span>
          <span className="block text-[11px] text-slate-400">
            每天首次启动应用时自动备份整个数据库，保留最近 14 份（prohelm-auto-*）
          </span>
        </span>
      </label>

      <div className="flex flex-wrap gap-2">
        <Button size="sm" onClick={() => void create()} disabled={busy}>
          立即备份
        </Button>
        <Button size="sm" variant="secondary" onClick={() => void exportAll()}>
          导出全量数据包
        </Button>
        <Button size="sm" variant="secondary" onClick={() => void importAll()}>
          导入数据包
        </Button>
      </div>

      <div>
        <div className="mb-1 text-xs font-semibold text-slate-500">备份历史</div>
        {backups.isLoading ? (
          <div className="py-2 text-center text-xs text-slate-400">加载中…</div>
        ) : (backups.data?.length ?? 0) === 0 ? (
          <div className="py-2 text-center text-xs text-slate-400">还没有备份</div>
        ) : (
          <div className="max-h-56 divide-y divide-slate-50 overflow-y-auto rounded-lg border border-slate-100">
            {backups.data!.map((b) => (
              <div key={b.fileName} className="flex items-center gap-2 px-3 py-2 text-xs">
                {b.fileName.startsWith("prohelm-auto-") && (
                  <span className="shrink-0 rounded bg-blue-50 px-1.5 py-0.5 text-[10px] text-blue-600">
                    自动
                  </span>
                )}
                <span className="min-w-0 flex-1 truncate font-mono text-slate-700">
                  {b.fileName}
                </span>
                <span className="shrink-0 text-slate-400">
                  {(b.sizeBytes / 1024).toFixed(0)} KB
                </span>
                <span className="shrink-0 text-slate-400">{b.createdAt}</span>
                <button
                  onClick={() => void restore(b.fileName)}
                  className="shrink-0 text-blue-600 hover:underline"
                >
                  恢复
                </button>
                <button
                  onClick={() => void remove(b.fileName)}
                  className="shrink-0 text-red-500 hover:underline"
                >
                  删除
                </button>
              </div>
            ))}
          </div>
        )}
      </div>

      <p className="text-xs leading-relaxed text-slate-500">
        数据文件位置：%APPDATA%\com.prohelm.desktop\prohelm.db（Windows）。恢复与导入会先自动备份当前库；
        数据包不含 AI 溯源与本机设置。每日自动快照已上线（见上方开关）；SQLCipher 整库加密按路线图排期。
      </p>
    </div>
  );
}

/** 内测诊断统计：本地聚合，主动复制反馈；无任何自动上报。 */
function TelemetryCard() {
  const t = useQuery({ queryKey: ["telemetry"], queryFn: api.telemetry });

  const copy = async () => {
    if (!t.data) return;
    try {
      await navigator.clipboard.writeText(JSON.stringify(t.data, null, 2));
      toast("诊断信息已复制，可粘贴到内测反馈渠道");
    } catch {
      toastError("复制失败，请手动截图本卡片");
    }
  };

  const cells: Array<{ label: string; value: string | number }> = t.data
    ? [
        { label: "激活（≥1 项目 +5 任务）", value: t.data.activated ? "✅" : "—" },
        { label: "累计活跃天数", value: t.data.activeDays },
        { label: "近 7 天活跃", value: t.data.activeDaysLast7 },
        { label: "创建项目", value: t.data.projectsCreated },
        { label: "创建任务", value: t.data.tasksCreated },
        { label: "登记风险", value: t.data.risksCreated },
        { label: "保存基线", value: t.data.baselinesCreated },
        { label: "登记工时", value: t.data.timeLogsCreated },
        { label: "AI 生成 / 采纳", value: `${t.data.aiGenerations} / ${t.data.aiAdopted}` },
      ]
    : [];

  return (
    <Card
      title="使用统计（内测诊断）"
      subtitle="仅统计操作次数与活跃天数，不含任何内容文本；数据留在本机，由你决定是否反馈"
    >
      {t.isLoading ? (
        <div className="py-2 text-center text-xs text-slate-400">加载中…</div>
      ) : t.isError ? (
        <div className="py-2 text-center text-xs text-red-600">统计加载失败：{t.error.message}</div>
      ) : (
        <div className="space-y-3">
          <div className="grid grid-cols-3 gap-2">
            {cells.map((c) => (
              <div key={c.label} className="rounded-lg border border-slate-100 bg-slate-50/60 px-2.5 py-2">
                <div className="text-[10px] text-slate-500">{c.label}</div>
                <div className="mt-0.5 text-sm font-semibold text-slate-700">{c.value}</div>
              </div>
            ))}
          </div>
          <div className="flex items-center justify-between">
            <span className="text-[11px] text-slate-400">
              首次使用：{t.data?.firstSeenAt?.slice(0, 10) ?? "—"} · 最近写入：
              {t.data?.lastActiveAt?.slice(0, 16).replace("T", " ") ?? "—"}
            </span>
            <Button size="sm" variant="secondary" onClick={() => void copy()}>
              复制诊断信息
            </Button>
          </div>
        </div>
      )}
    </Card>
  );
}
