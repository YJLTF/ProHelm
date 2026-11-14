//! PMP 教练知识库（自研撰写，仅引用公开方法论框架的概念与名称）。
//!
//! v1 检索 = 关键词评分（question 是否包含条目 keywords / 标题词）；
//! S6 升级为本地向量检索（sqlite-vec）。条目正文为撰写内容的精炼版，
//! 面向"中小团队项目经理怎么用"的实践视角。

pub struct KbEntry {
    pub id: &'static str,
    pub title: &'static str,
    /// 空格分隔的检索关键词（用户提问中包含即计分）
    pub keywords: &'static str,
    pub body: &'static str,
}

pub const KNOWLEDGE: &[KbEntry] = &[
    KbEntry {
        id: "kb-pmbok8",
        title: "PMBOK 第八版总览",
        keywords: "PMBOK 第八版 框架 体系 标准 pmbok",
        body: "PMBOK 第八版（2025 年 11 月发布）由 6 项原则、7 个绩效域和 5 个焦点领域（含 40 个非规范性过程）构成。焦点领域保留启动/规划/执行/监控/收尾的实践骨架，绩效域（治理、范围、进度、财务、干系人、资源、风险）提供健康度视角。基于第 8 版的新 PMP 考试自 2026-07-09 起生效。",
    },
    KbEntry {
        id: "kb-principles",
        title: "六项原则",
        keywords: "原则 整体视角 聚焦价值 嵌入质量 负责任领导 可持续性 赋权团队",
        body: "六项原则：整体视角（项目是组织系统的一部分）、聚焦价值（关注成果而非任务清单）、嵌入质量（一开始就做对）、负责任地领导（对决策和结果负责）、整合可持续性（长期影响）、建设赋权团队（让团队能自主决策）。",
    },
    KbEntry {
        id: "kb-domains",
        title: "七个绩效域",
        keywords: "绩效域 治理 范围 进度 财务 干系人 资源 风险 健康度",
        body: "七个绩效域是项目的七个健康度视角：治理（规则与决策流程）、范围（交付什么边界）、进度（时间线与依赖）、财务（预算与价值）、干系人（参与与关系）、资源（人与物）、风险（不确定性）。建议每周按七个域各问一遍'有没有恶化'。",
    },
    KbEntry {
        id: "kb-focus",
        title: "五个焦点领域",
        keywords: "焦点领域 启动 规划 执行 监控 收尾 过程 非规范性",
        body: "五个焦点领域（启动、规划、执行、监控与控制、收尾）覆盖 40 个非规范性过程——可以按项目规模裁剪。中小团队从'最小必要流程集'开始（章程+干系人+WBS+排期+风险+看板），缺什么补什么。",
    },
    KbEntry {
        id: "kb-charter",
        title: "项目章程",
        keywords: "章程 charter 启动 授权 成功标准",
        body: "章程是正式授权项目存在的文件，核心要素：项目目的与目标、可度量的成功标准、关键里程碑、预算、高层级风险、假设与制约、主要干系人、项目经理职权（能调动什么资源、能对外承诺什么）。章程要短，一页能说完最好；它与合同/需求一起是后续所有判断的基准。",
    },
    KbEntry {
        id: "kb-stakeholder",
        title: "干系人识别与权力/利益方格",
        keywords: "干系人 权力 利益 方格 识别 策略 登记",
        body: "先穷举干系人（客户、发起人、终端用户、监管方、被影响部门），登记期望与诉求，再按权力/利益方格定策略：高权力高利益→重点管理（周沟通）；高权力低利益→令其满意（关键节点汇报）；低权力高利益→随时告知；低权力低利益→监控即可。方格位置随项目阶段会变，定期回顾。",
    },
    KbEntry {
        id: "kb-wbs",
        title: "WBS 工作分解结构",
        keywords: "WBS 分解 工作包 100% 原则 层级",
        body: "WBS 按可交付成果逐层分解，叶子节点是可分配、可估算的工作包。100% 原则：子节点之和等于父节点范围，不多不少；不放'准备''协助'这类过程性条目。经验法则：工作包 ≤ 80 小时/2 周，超过就继续拆。WBS 是排期、预算、责任分配的共同基础。",
    },
    KbEntry {
        id: "kb-estimate",
        title: "工期估算方法",
        keywords: "估算 类比 参数 三点 PERT 工期 乐观 悲观",
        body: "常用估算：类比法（找历史相似项目）、参数法（单位工作量×数量）、三点估算（乐观 O / 最可能 M / 悲观 P，PERT 期望 = (O + 4M + P) / 6）。给估算留缓冲的正确位置是项目层的应急储备，而不是每项任务都偷偷加水分。",
    },
    KbEntry {
        id: "kb-cpm",
        title: "关键路径法（CPM）",
        keywords: "关键路径 CPM 网络图 顺推 逆推 浮动 总浮动",
        body: "关键路径法：按依赖关系顺推算出每项任务的最早开始/结束（ES/EF），逆推算出最晚开始/结束（LS/LF）；总浮动时间 = LS - ES。浮动为零的任务串起来就是关键路径——它决定项目最短工期。管理重点是保护关键路径上的任务，非关键任务可以借用浮动时间。",
    },
    KbEntry {
        id: "kb-crash",
        title: "赶工与快速跟进",
        keywords: "赶工 快速跟进 压缩 工期 进度 加资源 并行",
        body: "进度落后两种压缩手段：赶工=给关键路径任务加资源换工期（成本上升，边际收益递减）；快速跟进=把原本串行的活动部分并行（风险上升，返工概率增加）。优先压缩关键路径上的活动；压缩后重算关键路径，它可能转移。",
    },
    KbEntry {
        id: "kb-budget",
        title: "预算与储备",
        keywords: "预算 成本 储备 应急 管理储备 BAC 成本基准",
        body: "成本基准 = 各活动成本之和 + 应急储备（应对已识别风险，PM 可动用）；总预算 = 成本基准 + 管理储备（应对未知-未知，动用需管理层批准）。BAC（完工预算）是挣值分析的基准值。",
    },
    KbEntry {
        id: "kb-evm",
        title: "挣值管理（EVM）核心公式",
        keywords: "挣值 EVM SPI CPI EV PV AC 偏差 绩效",
        body: "三个基础值：PV（计划价值）、EV（已完成工作的计划价值）、AC（实际成本）。偏差：SV = EV - PV（进度），CV = EV - AC（成本）；指数：SPI = EV / PV，CPI = EV / AC。SPI < 1 进度落后，CPI < 1 成本超支。CPI 具有稳定性——今天 CPI 0.8，往往整个项目都是 0.8 左右，必须尽早干预。",
    },
    KbEntry {
        id: "kb-eac",
        title: "完工预测（EAC/ETC/TCPI）",
        keywords: "EAC ETC TCPI 完工估算 预测 总成本",
        body: "典型偏差持续时：EAC = BAC / CPI；按计划继续且不再偏差：EAC = AC + (BAC - EV)；重新自下而上估算：EAC = AC + 重估的 ETC。TCPI = (BAC - EV) / (BAC - AC)，即'剩余工作要达到的效率'——TCPI > 1.2 通常意味着原预算已不现实，尽早上报而不是硬扛。",
    },
    KbEntry {
        id: "kb-risk-process",
        title: "风险管理过程",
        keywords: "风险管理 识别 定性 定量 应对 监督",
        body: "风险管理循环：识别（写入风险登记册，越早越多越好）→ 定性分析（概率×影响矩阵排序）→ 定量分析（大项目才需要：预期货币价值 EMV、蒙特卡洛）→ 规划应对 → 实施应对 → 监督（新增、恶化、失效都要更新）。风险登记册是活文档，不是立项一次性作业。",
    },
    KbEntry {
        id: "kb-risk-strategy",
        title: "风险应对策略",
        keywords: "应对 规避 转移 减轻 接受 上报 威胁 机会",
        body: "对威胁：规避（改计划消除）、转移（保险/外包/合同条款）、减轻（降低概率或影响）、接受（主动接受留应急储备，或被动接受不作为）、上报（超出项目权限）。对机会：开拓（确保抓住）、分享（与合作方共赢）、提高（放大概率或影响）、接受。每条风险选一个主策略并写清具体措施。",
    },
    KbEntry {
        id: "kb-risk-register",
        title: "风险登记册要素",
        keywords: "风险登记册 登记册 要素 触发 预警 责任人",
        body: "每条风险包含：描述（起因-事件-影响三段式）、类别、概率与影响评分、评分（概率×影响）、应对策略与具体计划、责任人、预警信号（出现什么迹象说明正在发生）、状态（打开/已发生/已关闭）。预警信号是风险管理的灵魂——没有触发条件的应对计划不会被执行。",
    },
    KbEntry {
        id: "kb-change",
        title: "整体变更控制",
        keywords: "变更 控制 变更请求 CCB 基线 审批",
        body: "变更控制流程：提交变更请求（书面）→ 影响分析（范围/工期/成本/质量波及面）→ 决策（小项目 PM+客户即可，正式项目走变更控制委员会 CCB）→ 更新基线与计划 → 通知所有受影响干系人。核心纪律：口头变更不算数，一切变更留痕；拒绝也是决策，同样要记录。",
    },
    KbEntry {
        id: "kb-issue",
        title: "问题与风险的区别",
        keywords: "问题 issue 风险 区别 升级",
        body: "风险是不确定的、可能发生的事；问题（Issue）是已经发生、正在影响项目的事。风险一旦发生：从风险登记册转入问题管理，预设的应对计划变成行动项。问题解决后要问：它是当初登记过的风险吗？为什么没拦住？——这是经验教训的直接来源。",
    },
    KbEntry {
        id: "kb-communication",
        title: "沟通管理",
        keywords: "沟通 计划 矩阵 频率 报告 5C",
        body: "沟通矩阵回答五个问题：谁需要什么信息、什么时候、什么方式、发给谁、由谁发。原则：给高管一页纸结论，给团队细节清单；坏消息要早说。5C：正确（Correct）、简洁（Concise）、清晰（Clear）、连贯（Coherent）、可控（Controlling）。",
    },
    KbEntry {
        id: "kb-quality",
        title: "质量管理",
        keywords: "质量 计划 检查 评审 缺陷 预防",
        body: "质量要融入过程而不是事后检验：预防成本（评审、培训）远低于缺陷成本（返工、召回）。实操：每个交付物定义'完成的标准'（DoD）、用检查表执行、评审节点前移。质量计划回答：标准是什么、谁检查、何时检查、不达标怎么办。",
    },
    KbEntry {
        id: "kb-agile",
        title: "敏捷方法要点",
        keywords: "敏捷 迭代 sprint 燃尽 回顾 待办",
        body: "敏捷核心是固定节奏的小批量交付：待办梳理 → 迭代规划 → 每日站会 → 迭代评审（演示可用的东西）→ 回顾（改进流程）。燃尽图看剩余量趋势而非绝对值。敏捷不等于不做计划——它把计划粒度从'一次做全'改为'滚动细化'。",
    },
    KbEntry {
        id: "kb-hybrid",
        title: "混合方法",
        keywords: "混合 预测 敏捷 裁剪 方法选择",
        body: "混合方法 = 骨架用预测型（对甲方：里程碑、验收、合同），执行用敏捷（对团队：迭代、看板）。选择依据：需求稳定的部分预测，易变的部分敏捷。中小团队最常见的误区是二选一站队，实际按交付物性质混合裁剪最务实。",
    },
    KbEntry {
        id: "kb-tailoring",
        title: "方法论裁剪",
        keywords: "裁剪 最小 必要 流程 项目规模",
        body: "裁剪 = 按项目规模、复杂度、干系人要求选择'最小必要流程集'。判据：这个流程产出的信息会被谁、在什么决策里使用？没人用的流程就是流程税。小团队起步配置：章程、干系人、WBS、排期、风险登记册、看板；变更控制和挣值在多甲方/长周期项目上再加。",
    },
    KbEntry {
        id: "kb-milestone",
        title: "里程碑管理",
        keywords: "里程碑 节点 零工期 阶段 验收",
        body: "里程碑是零工期的重要时点（需求冻结、评审通过、验收签字），是对甲方汇报的最直观语言。管理要点：里程碑要少而关键（一页放得下）；每个里程碑绑定明确的通过标准；里程碑连续偏差两次就要系统性归因，而不是逐个解释。",
    },
    KbEntry {
        id: "kb-baseline",
        title: "计划基线",
        keywords: "基线 baseline 快照 对比 批准",
        body: "基线 = 经批准的计划快照（范围/进度/成本三条基线）。执行中所有'偏差''绩效'都是相对基线而言——没有基线的'超期''超支'只是感觉。纪律：基线只随正式变更走，日常波动不刷基线，否则对比失去意义。",
    },
    KbEntry {
        id: "kb-meeting",
        title: "会议与行动项",
        keywords: "会议 纪要 行动项 站会 三问",
        body: "每个会议必须产出：结论 + 行动项（责任人 + 期限 + 验收方式），否则等于没开。每日站会三问：昨天完成了什么、今天计划做什么、有什么阻塞。阻塞超过 24 小时未解决就升级。会议时长：站会 ≤15 分钟，评审 ≤1 小时。",
    },
    KbEntry {
        id: "kb-timesheet",
        title: "工时与实际成本记录",
        keywords: "工时 记录 成本 AC 实际 填报",
        body: "及时记录工时的意义：形成实际成本（AC），让挣值分析有输入；暴露估算偏差，喂给下一个项目的估算。纪律：周清（每周五补齐当周工时）；滞后记录让偏差发现变晚一整周。异常工时（某项任务远超估算）要当场追问原因。",
    },
    KbEntry {
        id: "kb-lessons",
        title: "经验教训登记册",
        keywords: "经验教训 复盘 lessons 收尾 归档",
        body: "收尾复盘的结构化三问：什么做得好（保持）、什么做得差（避免）、下次建议（行动）。关键在于'事中记录'：风险发生、变更审批、返工发生的当下就记一笔，收尾时汇总——只靠回忆的复盘会美化记忆。",
    },
    KbEntry {
        id: "kb-closing",
        title: "项目收尾要点",
        keywords: "收尾 验收 移交 归档 释放 合同",
        body: "收尾清单：范围确认（对照章程成功标准逐条验收）→ 交付物移交与签收 → 合同/财务收尾（结算对照预算）→ 经验教训归档 → 团队资源释放与反馈 → 项目归档（可复用为模板）。不要跳过验收直接进入'下一件事'，未验收的项目永远占着心智能量。",
    },
    KbEntry {
        id: "kb-scope-creep",
        title: "范围蔓延与镀金",
        keywords: "范围蔓延 镀金 creep scope 加需求",
        body: "范围蔓延（Scope Creep）= 未经变更控制的需求累加；镀金（Gold Plating）= 团队主动超出范围给'惊喜'。两者都消耗工期预算却不被承认。防御：WBS 基线 + 书面变更控制 + 每个交付物明确的验收标准 + 对'顺手做了'说不。",
    },
    KbEntry {
        id: "kb-kickoff",
        title: "项目启动会",
        keywords: "启动会 kickoff 会议 开工",
        body: "启动会的目的是统一预期：项目目标与成功标准、范围边界（不做什么同样重要）、关键角色与分工、沟通方式与节奏、里程碑计划。邀请全部关键干系人，会后发出会议纪要作为第一份基线记录。启动会开了不等于对齐了——会前一对一摸底关键干系人才是真正的对齐。",
    },
    KbEntry {
        id: "kb-weekly-report",
        title: "项目周报要点",
        keywords: "周报 报告 汇报 进度",
        body: "周报三段式：1）进度对比基线（里程碑达成、SPI/关键路径变化）；2）风险与问题变化（新增、恶化、关闭）；3）下周计划与需要的支持（把'求资源'变成正式请求）。给甲方的周报永远先说结论和需要决策的事，细节放附录。",
    },
    KbEntry {
        id: "kb-resource-level",
        title: "资源负荷与平衡",
        keywords: "资源 负荷 过载 平衡 增援",
        body: "资源负荷热力图暴露'某人同时被 3 个项目压满'。平衡手段按优先级：调整非关键路径任务时间（利用浮动）、增援或替换、降低该任务的并行度、最后才是缩减范围。过载的人产出质量最先崩，宁可排期诚实也不要假装能做完。",
    },
    KbEntry {
        id: "kb-team-tuckman",
        title: "团队发展阶梯（塔克曼）",
        keywords: "团队 形成 震荡 规范 成熟 塔克曼 冲突",
        body: "团队发展五阶段：形成（客气）→ 震荡（冲突暴露）→ 规范（建立规则）→ 成熟（自组织）→ 解散。震荡期的冲突是正常的、甚至是必要的——PM 的任务是促成规则与信任，而不是压制分歧。新成员加入会让团队退回前一阶段。",
    },
    KbEntry {
        id: "kb-overview",
        title: "项舵使用总览",
        keywords: "项舵 prohelm 使用 怎么用 入门 流程",
        body: "项舵推荐用法：从模板库创建项目（或手工创建）→ 确认方法论裁剪 → 填写章程 → 登记干系人 → WBS 分解与排期（甘特图看关键路径）→ 保存计划基线 → 执行期在看板流转与记录工时 → 每周看监控页签健康信号 → 收尾按清单验收与复盘。",
    },
];

/// 简单评分检索（v1）：统计提问中命中的关键词，按得分降序取 top_k。
/// S6 升级为本地向量检索（sqlite-vec）。
pub fn retrieve(query: &str, top_k: usize) -> Vec<&'static KbEntry> {
    let q = query.trim();
    if q.is_empty() {
        return Vec::new();
    }
    let mut scored: Vec<(i64, &KbEntry)> = KNOWLEDGE
        .iter()
        .map(|e| {
            let mut score = 0i64;
            for kw in e.keywords.split_whitespace() {
                if q.contains(kw) {
                    score += 2;
                }
            }
            // 标题整体出现在提问中给高分
            if q.contains(e.title) {
                score += 3;
            }
            // 正文命中的关键词（弱信号）
            if e.keywords
                .split_whitespace()
                .any(|kw| kw.chars().count() >= 2 && e.body.contains(kw) && q.contains(kw))
            {
                score += 1;
            }
            (score, e)
        })
        .filter(|(score, _)| *score > 0)
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.id.cmp(b.1.id)));
    if scored.is_empty() {
        // 兜底：返回总览条目，保证教练永远有上下文
        if let Some(overview) = KNOWLEDGE.iter().find(|e| e.id == "kb-overview") {
            return vec![overview];
        }
    }
    scored.into_iter().take(top_k).map(|(_, e)| e).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retrieve_hits_relevant_entries() {
        let hits = retrieve("什么是关键路径？浮动时间怎么算", 3);
        assert!(!hits.is_empty());
        assert!(hits.iter().any(|e| e.id == "kb-cpm"));

        let hits = retrieve("SPI CPI 公式是什么", 3);
        assert!(hits.iter().any(|e| e.id == "kb-evm"));

        let hits = retrieve("变更控制流程怎么走", 3);
        assert!(hits.iter().any(|e| e.id == "kb-change"));
    }

    #[test]
    fn retrieve_falls_back_to_overview() {
        let hits = retrieve("今天天气怎么样", 3);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].id, "kb-overview");
    }

    #[test]
    fn knowledge_entries_have_unique_ids() {
        let mut ids: Vec<_> = KNOWLEDGE.iter().map(|e| e.id).collect();
        ids.sort_unstable();
        let len = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), len, "知识条目 id 存在重复");
    }
}
