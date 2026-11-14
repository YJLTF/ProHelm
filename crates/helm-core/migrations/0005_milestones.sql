-- 0005: S5 —— 任务里程碑标记（零工期节点，甘特图以深色菱形语义呈现）。

ALTER TABLE tasks ADD COLUMN is_milestone INTEGER NOT NULL DEFAULT 0;
