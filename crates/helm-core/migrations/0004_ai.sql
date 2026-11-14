-- 0004: S4 AI 溯源 —— 每一次 AI 生成（教练问答 / 章程草稿 / WBS 草稿）都记录
-- 输入摘要、输出全文与采纳状态，支撑"可溯源、可拒绝、可撤销"的产品铁律。
-- project_id 可为 NULL（全局教练问答不归属项目）。

CREATE TABLE ai_artifacts (
    id            TEXT PRIMARY KEY,
    project_id    TEXT REFERENCES projects (id) ON DELETE CASCADE,
    kind          TEXT NOT NULL, -- coach | charter_draft | wbs_draft
    provider      TEXT NOT NULL,
    model         TEXT NOT NULL,
    input_summary TEXT NOT NULL, -- 输入摘要（提问 / 需求描述前 200 字符）
    output        TEXT NOT NULL, -- 输出全文（JSON 或文本）
    status        TEXT NOT NULL DEFAULT 'draft', -- draft | adopted | discarded
    created_at    TEXT NOT NULL
);

CREATE INDEX idx_ai_artifacts_project ON ai_artifacts (project_id);
CREATE INDEX idx_ai_artifacts_created ON ai_artifacts (created_at DESC);
