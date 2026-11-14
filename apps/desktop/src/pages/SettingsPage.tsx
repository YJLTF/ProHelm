import { Bot, Database, Info, ShieldCheck } from "lucide-react";

import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import { Input, Label, Select } from "../components/ui/Input";

const PROVIDERS = ["GLM（智谱）", "DeepSeek", "通义千问 Qwen", "OpenAI", "Anthropic", "Ollama（本地）", "OpenRouter"];

export function SettingsPage() {
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
          subtitle="S4 上线：自带 API Key，直连模型供应商；密钥存系统钥匙串，不经过任何第三方服务器"
        >
          <div className="space-y-3 opacity-70">
            <div>
              <Label>模型供应商</Label>
              <Select disabled defaultValue="">
                <option value="">选择供应商（S4 上线后可配置）</option>
                {PROVIDERS.map((p) => (
                  <option key={p} value={p}>
                    {p}
                  </option>
                ))}
              </Select>
            </div>
            <div>
              <Label>API Key</Label>
              <Input disabled placeholder="密钥将保存在系统钥匙串（Win 凭据管理器 / macOS Keychain）" />
            </div>
            <p className="text-xs leading-relaxed text-slate-500">
              隐私承诺：默认脱敏外发（项目名/人名/客户名替换为代号）；外发范围可配置；一切 AI
              写入先预览、可拒绝、可撤销。
            </p>
          </div>
        </Card>

        <Card
          title={
            <span className="flex items-center gap-2">
              <Database size={15} className="text-blue-600" /> 数据与备份
            </span>
          }
          subtitle="本地优先 · 数据不出设备"
        >
          <div className="space-y-3 text-sm text-slate-600">
            <p>
              数据文件：
              <span className="mx-1 rounded bg-slate-100 px-1.5 py-0.5 font-mono text-xs">
                Windows: %APPDATA%\com.prohelm.desktop\prohelm.db
              </span>
              <span className="mx-1 rounded bg-slate-100 px-1.5 py-0.5 font-mono text-xs">
                macOS: ~/Library/Application Support/com.prohelm.desktop/
              </span>
            </p>
            <p className="text-xs text-slate-500">
              S5 上线：每日自动快照、一键备份/恢复、JSON 全量包 / CSV / Markdown 报告导出、SQLCipher
              整库加密开关。
            </p>
            <div className="flex gap-2">
              <Button variant="secondary" size="sm" disabled>
                立即备份（S5）
              </Button>
              <Button variant="secondary" size="sm" disabled>
                导出数据（S5）
              </Button>
            </div>
          </div>
        </Card>

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
              挣值管理）作为功能落地。模板与知识库为自研撰写，仅引用公开方法论框架概念。
            </p>
          </div>
        </Card>
      </div>
    </div>
  );
}
