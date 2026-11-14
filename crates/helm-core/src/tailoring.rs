//! 方法论裁剪：流程项清单与按开发方法的推荐预设。
//!
//! PMBOK 8 强调 40 个过程"非规范性、可裁剪"（见 docs/01 第 2 章）——
//! 本模块是"最小必要流程集"的唯一事实源：创建项目时预置，用户可在
//! 「启动 → 裁剪器」中增删（前端展示标签与这里的 key 对应）。

use crate::models::ProjectMethod;

/// (key, 标签)。key 为存储标识；标签供 UI 显示。
pub const ALL_ITEMS: &[(&str, &str)] = &[
    // 启动
    ("charter", "项目章程"),
    ("stakeholder", "干系人登记册"),
    // 规划
    ("wbs", "WBS 分解"),
    ("schedule", "排期与关键路径"),
    ("budget", "预算与估算"),
    ("risk", "风险登记册"),
    ("communication", "沟通计划"),
    ("procurement", "采购计划"),
    ("quality", "质量计划"),
    // 执行
    ("kanban", "任务看板"),
    ("timesheet", "工时记录"),
    ("meeting", "会议与行动项"),
    // 监控
    ("evm", "挣值管理"),
    ("change_control", "变更控制"),
    ("issue", "问题登记册"),
    // 收尾
    ("acceptance", "验收清单"),
    ("lessons", "经验教训登记册"),
];

pub fn is_valid_key(key: &str) -> bool {
    ALL_ITEMS.iter().any(|(k, _)| *k == key)
}

/// 按开发方法返回推荐裁剪（"最小必要流程集"）。
pub fn preset_for_method(method: ProjectMethod) -> Vec<&'static str> {
    let keys: &[&str] = match method {
        // 预测型：完整计划驱动流程（不含敏捷看板）
        ProjectMethod::Predictive => &[
            "charter",
            "stakeholder",
            "wbs",
            "schedule",
            "budget",
            "risk",
            "communication",
            "procurement",
            "quality",
            "timesheet",
            "meeting",
            "evm",
            "change_control",
            "issue",
            "acceptance",
            "lessons",
        ],
        // 敏捷：轻计划、重执行
        ProjectMethod::Agile => &[
            "charter",
            "stakeholder",
            "wbs",
            "schedule",
            "risk",
            "kanban",
            "meeting",
            "lessons",
        ],
        // 混合：计划骨架 + 敏捷执行
        ProjectMethod::Hybrid => &[
            "charter",
            "stakeholder",
            "wbs",
            "schedule",
            "budget",
            "risk",
            "kanban",
            "timesheet",
            "meeting",
            "change_control",
            "acceptance",
            "lessons",
        ],
    };
    keys.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_are_valid_and_distinct() {
        for method in [
            ProjectMethod::Predictive,
            ProjectMethod::Agile,
            ProjectMethod::Hybrid,
        ] {
            let preset = preset_for_method(method);
            assert!(!preset.is_empty());
            for key in &preset {
                assert!(is_valid_key(key), "预设含未知 key: {key}");
            }
            let unique: std::collections::HashSet<_> = preset.iter().collect();
            assert_eq!(unique.len(), preset.len(), "预设存在重复 key");
        }
    }
}
