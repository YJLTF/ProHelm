-- 0001: 初始结构 —— 项目、任务、应用设置、操作日志（OpLog）。
-- 设计说明见 docs/03-技术选型与架构设计.md 第 5 章：
-- - 主键统一 ULID（离线创建不冲突，时间有序）；
-- - oplog 记录所有写操作，是 M3 同步引擎与撤销功能的数据基础。

CREATE TABLE projects (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    code            TEXT,
    method          TEXT NOT NULL DEFAULT 'hybrid',   -- predictive | agile | hybrid
    status          TEXT NOT NULL DEFAULT 'planning', -- planning | active | paused | closed
    description     TEXT,
    start_date      TEXT,
    target_end_date TEXT,
    bac             REAL NOT NULL DEFAULT 0,          -- Budget at Completion（M2 EVM 使用）
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

CREATE INDEX idx_projects_updated ON projects (updated_at DESC);

CREATE TABLE tasks (
    id            TEXT PRIMARY KEY,
    project_id    TEXT NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
    parent_id     TEXT REFERENCES tasks (id) ON DELETE CASCADE,
    wbs_code      TEXT,                               -- 自动维护：1、1.1、1.2.3
    title         TEXT NOT NULL,
    notes         TEXT,
    status        TEXT NOT NULL DEFAULT 'todo',       -- todo | in_progress | done
    priority      TEXT NOT NULL DEFAULT 'medium',     -- low | medium | high | urgent
    estimate_hours  REAL NOT NULL DEFAULT 0,
    actual_hours    REAL NOT NULL DEFAULT 0,
    start_date    TEXT,
    end_date      TEXT,
    assignee      TEXT,                               -- S3 引入成员表后改为外键
    sort_order    INTEGER NOT NULL DEFAULT 0,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
);

CREATE INDEX idx_tasks_project ON tasks (project_id);
CREATE INDEX idx_tasks_parent ON tasks (parent_id);
CREATE INDEX idx_tasks_status ON tasks (status);

CREATE TABLE settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE oplog (
    id        TEXT PRIMARY KEY,
    ts        TEXT NOT NULL,
    entity    TEXT NOT NULL,   -- 'project' | 'task'
    entity_id TEXT NOT NULL,
    op        TEXT NOT NULL,   -- 'create' | 'update' | 'delete'
    payload   TEXT NOT NULL,   -- 操作快照（JSON）
    device    TEXT NOT NULL DEFAULT 'local'
);

CREATE INDEX idx_oplog_ts ON oplog (ts);
