-- 0006: S6 挣值管理（EVM）—— 工时台账与任务完成时间。
-- - time_log 是 AC（实际成本，以人时计）的唯一台账；tasks.actual_hours 为其派生缓存
--   （登记/删除工时后由仓库层重算，保持人员负荷等既有统计兼容）；
-- - tasks.completed_at 记录任务转入 done 的时刻，是时间相位 EV（挣值）曲线的数据基础；
--   旧数据该列为 NULL，EVM 汇总时按"已完成"计入、时序上落在截止日。

CREATE TABLE time_log (
    id         TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
    task_id    TEXT NOT NULL REFERENCES tasks (id) ON DELETE CASCADE,
    log_date   TEXT NOT NULL,                -- YYYY-MM-DD（工时归属日）
    hours      REAL NOT NULL,
    note       TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_time_log_project ON time_log (project_id);
CREATE INDEX idx_time_log_task ON time_log (task_id);
CREATE INDEX idx_time_log_date ON time_log (log_date);

ALTER TABLE tasks ADD COLUMN completed_at TEXT;
