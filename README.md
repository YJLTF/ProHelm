# ProHelm（项舵）

> 基于 PMP 体系的 AI 原生、本地优先桌面项目管理工具 —— 帮中小团队"接得住项目、看得清风险、交得出结果"。

## 产品定位（一句话）

把 **PMP 方法论** + **AI 舵手** + **数据不出本地的桌面应用** 组合到一起，服务 5~50 人团队的项目经理与核心成员。

## 文档导航

| 文档 | 内容 |
|---|---|
| [01-市场调研与PMP体系梳理](docs/01-市场调研与PMP体系梳理.md) | PMBOK 6→7→8 演进与选型结论、市场数据、竞品矩阵、AI 趋势、差异化机会 |
| [02-产品功能设计](docs/02-产品功能设计.md) | 定位与设计原则、用户画像、功能架构、五大焦点领域模块详设、AI 舵手四层能力、MVP 路线图、商业模式 |
| [03-技术选型与架构设计](docs/03-技术选型与架构设计.md) | 总体架构、Tauri 2 选型论证、前端/数据/AI/同步全栈选型、数据模型草案、安全设计、技术风险 |
| [04-开发计划](docs/04-开发计划.md) | 6 个 Sprint 排期、任务 Backlog、工程规范、发布策略、风险登记 |
| [05-团队组建方案](docs/05-团队组建方案.md) | 角色矩阵、配置方案 A/B、AI 智能体工作流、招聘要点、决策机制 |
| [conventions.md](docs/conventions.md) | 代码规范、测试要求、已知边界（AI 与人类共用基准） |

## 当前状态：M1 · S1~S5 已完成，S6（Alpha 发布）主体提前完成 ✅

- Tauri 2 + React 19 + Rust 核心（`helm-core` 业务层 + `helm-ai` AI 引擎）三端可构建
- SQLite 版本化迁移（0001~0006）+ 项目/任务/依赖/风险/干系人/章程/AI 溯源/工时台账 全链路（OpLog 打底）
- **S2**：CPM 关键路径算法、FS/SS/FF/SF 依赖（环检测）、甘特图、WBS 树编辑、计划基线
- **S3**：风险登记册（概率×影响矩阵）、干系人方格、章程编辑器、方法论裁剪器、看板拖拽、行业模板
- **S4**：AI 舵手——BYOK 网关（Key 存 OS 钥匙串）、PMP 教练问答（自研知识库 + 流式输出）、
  章程 AI 起草与 WBS 拆解（草稿→预览→采纳/弃用）、AiArtifact 溯源
- **S5**：SQLite 单文件备份与一键恢复、全量 JSON 数据包导入导出、项目 Markdown 报告与任务 CSV、
  跨项目风险雷达与人员负荷、里程碑标记、tauri build 打包链路验证
- **S6**：挣值管理（EVM）——工时台账、PV/EV/AC、SPI/CPI/EAC、S 曲线与挣值完成率；
  每日自动快照（保留 14 份 + 设置开关）；新建项目向导（空白/行业模板统一流程）
- CI：前端检查构建 + Rust fmt/clippy/test（GitHub Actions）；68 个 Rust 测试全绿

## 开发指南

### 环境要求

- Node ≥ 20.19（含 pnpm 12，可用 `corepack enable`）
- Rust stable（建议 ≥ 1.85）
- Windows：WebView2（系统自带）；macOS：Xcode CLT；Linux：webkit2gtk-4.1

### 常用命令

```bash
pnpm install                # 安装前端依赖
pnpm dev                    # 前端热更新开发（仅 Web 预览）
pnpm --filter @prohelm/desktop tauri dev    # 桌面端开发（完整应用）
pnpm build                  # 前端类型检查 + 构建
pnpm typecheck              # 仅类型检查
cargo check --workspace     # Rust 编译检查
cargo test -p helm-core     # 核心层测试
pnpm --filter @prohelm/desktop tauri build  # 打包安装程序
```

### 目录结构

```
apps/desktop/            # Tauri 桌面端
  src/                   #   React 前端（页面/组件/hooks/IPC 类型化封装）
  src-tauri/             #   Rust 壳（窗口/命令层，业务逻辑不在这里）
crates/helm-core/        # 核心业务层：模型/仓库层/迁移/算法（未来移动端与 CLI 复用）
  migrations/            #   SQLite 版本化迁移（只增不改）
docs/                    # 产品与工程文档（01~05 + conventions）
tools/                   # 辅助脚本（如图标生成）
.github/workflows/       # CI
```

### 架构速记

- **业务逻辑全在 Rust（helm-core）**，前端只做渲染；IPC 走类型化命令（`src/lib/ipc.ts` ↔ `src-tauri/src/commands.rs`）
- 数据层：SQLite（WAL + 外键）+ ULID 主键 + 每次写操作追加 OpLog（M3 同步、S4 撤销的基础）
- UI 主导航 = PMBOK 8 五个焦点领域（启动→规划→执行→监控→收尾），方法论即信息架构
- AI（S4）：BYOK 多模型网关 + 草稿优先 Agent + 自研方法论 RAG；一切 AI 写入先预览可撤销

## 市场窗口

- 中国项目管理软件市场 2026E 约 150.9 亿元，CAGR ~18.5%
- 新版 PMP 考试 2026-07-09 生效（对齐 PMBOK 8），约 65 万中国持证者面临体系切换
- Gartner：2026 年底 40% 企业应用将嵌入 AI Agent
