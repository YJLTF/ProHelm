-- 0003: S3 核心工件 —— 风险登记册、干系人登记册、项目章程、方法论裁剪配置。
-- 概率/影响采用 1~5 标度（评分 = 概率×影响，1~25，显示层计算并着色）；
-- 章程以分节 JSON 存储，S4 AI 起草将按同一结构逐节生成。

ALTER TABLE projects ADD COLUMN tailored TEXT; -- JSON 数组：启用的流程项 key（创建时按开发方法预置）

CREATE TABLE risks (
    id                TEXT PRIMARY KEY,
    project_id        TEXT NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
    risk_no           INTEGER NOT NULL,                -- 项目内自增编号（R1、R2…）
    title             TEXT NOT NULL,
    detail            TEXT,                            -- 描述 / 原因 / 影响说明
    category          TEXT NOT NULL DEFAULT 'technical',
    probability       INTEGER NOT NULL DEFAULT 3,      -- 1~5
    impact            INTEGER NOT NULL DEFAULT 3,      -- 1~5
    response_strategy TEXT NOT NULL DEFAULT 'mitigate', -- avoid | transfer | mitigate | accept | escalate
    response_plan     TEXT,
    owner             TEXT,
    trigger_desc      TEXT,                            -- 预警信号
    status            TEXT NOT NULL DEFAULT 'open',    -- open | occurred | closed
    created_at        TEXT NOT NULL,
    updated_at        TEXT NOT NULL
);

CREATE INDEX idx_risks_project ON risks (project_id);

CREATE TABLE stakeholders (
    id          TEXT PRIMARY KEY,
    project_id  TEXT NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    role        TEXT,
    expectation TEXT,                                -- 期望与诉求
    influence   INTEGER NOT NULL DEFAULT 3,          -- 权力 1~5
    interest    INTEGER NOT NULL DEFAULT 3,          -- 利益/关注 1~5
    notes       TEXT,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE INDEX idx_stakeholders_project ON stakeholders (project_id);

CREATE TABLE charters (
    id         TEXT PRIMARY KEY,
    project_id TEXT NOT NULL UNIQUE REFERENCES projects (id) ON DELETE CASCADE,
    content    TEXT NOT NULL, -- JSON：分节章程（CharterContent）
    updated_at TEXT NOT NULL
);
