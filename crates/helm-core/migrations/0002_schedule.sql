-- 0002: S2 进度能力 —— 任务工期、依赖网络（CPM）、计划基线。
-- 依赖边支持 FS/SS/FF/SF 四类 + 滞后天数，是关键路径算法的输入；
-- baselines 存结构快照（JSON），M2 挣值管理以基线为对比基准。

ALTER TABLE tasks ADD COLUMN duration_days REAL NOT NULL DEFAULT 1;

CREATE TABLE dependencies (
    id             TEXT PRIMARY KEY,
    project_id     TEXT NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
    predecessor_id TEXT NOT NULL REFERENCES tasks (id) ON DELETE CASCADE,
    successor_id   TEXT NOT NULL REFERENCES tasks (id) ON DELETE CASCADE,
    dep_type       TEXT NOT NULL DEFAULT 'fs',  -- fs | ss | ff | sf
    lag_days       REAL NOT NULL DEFAULT 0,
    created_at     TEXT NOT NULL,
    UNIQUE (predecessor_id, successor_id)
);

CREATE INDEX idx_deps_project ON dependencies (project_id);
CREATE INDEX idx_deps_predecessor ON dependencies (predecessor_id);
CREATE INDEX idx_deps_successor ON dependencies (successor_id);

CREATE TABLE baselines (
    id         TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
    name       TEXT NOT NULL,
    snapshot   TEXT NOT NULL, -- JSON：任务 + 依赖完整快照
    created_at TEXT NOT NULL
);

CREATE INDEX idx_baselines_project ON baselines (project_id);
