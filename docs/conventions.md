# 开发协作规范（AI 与人类共用）

> 配套：《04-开发计划》§4 工程规范、《05-团队组建方案》§3 AI 工作流。
> 本文件是 AI 智能体生成代码与人工 Review 的共同基准，随项目演进持续补充。

## 1. 语言与文案

- UI 文案：简体中文；技术标识符（代码、提交信息主题、分支名）：英文
- 错误提示面向用户说"人话"：说明发生了什么、该怎么办；禁止堆栈直出用户
- 领域术语遵循 PMBOK 中文惯例：WBS、关键路径、挣值管理（SPI/CPI）、风险登记册、干系人

## 2. 代码规范

### Rust（helm-core / src-tauri）

- 命令层（src-tauri）只做编排与错误转换，业务逻辑进 helm-core
- 可失败路径一律 `Result`，禁止 `unwrap()`/`expect()` 出现在命令与仓库层
- 领域枚举用 `db_enum!` 宏定义（models.rs），数据库 TEXT 与 serde JSON 共用一套取值
- SQL 迁移只增不改（`migrations/NNNN_*.sql`），新表必须带索引
- 写操作必须追加 oplog（`repo::oplog::record`）
- clippy -D warnings 必须全绿；`cargo fmt` 提交前执行

### TypeScript / React（前端）

- 严格模式 + verbatimModuleSyntax：类型导入必须 `import type`
- 与 Rust DTO 对齐的类型集中在 `src/types.ts`，修改模型必须双向同步
- IPC 调用只走 `src/lib/ipc.ts`，组件不得直接 `invoke()`
- 服务端状态用 TanStack Query（hooks 集中在 `src/hooks/`），纯 UI 态用 Zustand
- 组件文件单一职责；宽度冲突用包裹 div 控制，不用 important 覆盖

## 3. 测试要求

- 仓库层 CRUD：增删改查 + 校验失败分支全覆盖
- 算法（CPM/EVM/后续同步合并）：黄金用例 + 快照断言，100% 分支覆盖
- 每个 bug 修复附带回归测试

## 4. 已知边界（有意为之，勿当作 bug 反复修）

- 部分更新用 COALESCE 语义：`null` 表示"不修改"，暂不支持把字段清空（S5 引入 `Set<T>` 语义）
- 依赖编辑目前仅在创建任务/模板中指定单个 FS 前置；SS/FF/SF 需走 API，行内编辑 S5 实现
- 甘特图 readonly：拖拽改期、里程碑标记、工作日历 S5 实现
- 排程按日历天（无工作日历/资源日历），S5 引入资源负荷后升级
- 看板仅跨列流转，列内排序在"规划列表"用上移/下移完成
- 数据安全：备份/数据包导出不含 AI 溯源与本机设置；恢复/导入前自动备份当前库
- EVM（M2 剩余部分）、成员表/RACI/变更控制、每日自动快照与 SQLCipher 开关（S6）按路线图排期
- AI 仅走 OpenAI 兼容协议（Anthropic 原生协议 S5）；PMP 教练知识库检索为关键词评分（S6 升级向量）
- AI 产出一律先落 AiArtifact（draft），采纳/弃用由用户显式操作；AI 永不直接写库
