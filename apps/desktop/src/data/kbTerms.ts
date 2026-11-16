/**
 * PMP 术语速查数据（自研撰写，规避 PMBOK 原文版权——与 helm-ai 教练知识库同一原则）。
 * 结构：id 用于锚点跳转（/kb?term=id）与 Term 引导组件；related 互链。
 */

export type KbCategoryKey =
  | "methodology"
  | "initiation"
  | "planning"
  | "evm"
  | "risk"
  | "execution"
  | "closing";

export const KB_CATEGORIES: Array<{ key: KbCategoryKey; name: string; desc: string }> = [
  { key: "methodology", name: "方法论框架", desc: "PMBOK、生命周期与裁剪" },
  { key: "initiation", name: "启动与干系人", desc: "章程、商业论证、干系人" },
  { key: "planning", name: "规划与进度", desc: "WBS、依赖、关键路径、基线" },
  { key: "evm", name: "成本与挣值", desc: "EVM 全家桶：PV/EV/AC/SPI/CPI" },
  { key: "risk", name: "风险管理", desc: "登记册、矩阵、应对策略" },
  { key: "execution", name: "执行与监控", desc: "看板与流转" },
  { key: "closing", name: "收尾", desc: "验收、经验教训、行政收尾" },
];

export interface KbTerm {
  id: string;
  term: string;
  abbr?: string;
  category: KbCategoryKey;
  /** 一句话定义（hover 卡片与列表摘要） */
  summary: string;
  /** 详解（多段） */
  detail: string[];
  /** 在 ProHelm 中的位置与用法 */
  inProHelm: string;
  related: string[];
}

export const KB_TERMS: KbTerm[] = [
  // ---------- 方法论框架 ----------
  {
    id: "pmbok",
    term: "PMBOK 指南",
    abbr: "PMBOK",
    category: "methodology",
    summary: "PMI 发布的项目管理知识体系，PMP 认证与全球最佳实践的参考基准。",
    detail: [
      "PMBOK（Project Management Body of Knowledge）是项目管理协会（PMI）定期修订的知识体系汇总。第 8 版不再按\"过程\"罗列做法，而是以原则、焦点领域与绩效域组织内容，强调价值交付与方法适配。",
      "对实务的意义：它不是操作手册，而是\"该考虑什么\"的清单。PMP 考试（2026-07 起对齐第 8 版）主要考查对原则与情境的判断，而非背诵术语。"
    ],
    inProHelm: "整个应用的主导航就按 PMBOK 8 的五个焦点领域组织（启动→规划→执行→监控→收尾）；右下角舵手可以回答体系相关问题。",
    related: ["focus-areas", "tailoring", "lifecycle"],
  },
  {
    id: "focus-areas",
    term: "五大焦点领域",
    abbr: "Focus Areas",
    category: "methodology",
    summary: "PMBOK 8 用启动/规划/执行/监控/收尾五个焦点领域取代了旧版的\"过程组\"。",
    detail: [
      "焦点领域回答\"现在这个阶段应该关心什么\"：启动授权与对齐，规划把不确定性变成计划，执行交付成果，监控对比偏差并纠偏，收尾正式关闭并沉淀经验。",
      "与旧版五大过程组一脉相承，但更强调\"领域内的活动可以随生命周期形态伸缩\"——敏捷项目同样有启动与收尾，只是形态更轻。"
    ],
    inProHelm: "每个项目工作区顶部的五个页签即焦点领域，页签内放置该阶段该用的工件（章程/风险/基线/EVM 等）。",
    related: ["pmbok", "lifecycle", "tailoring"],
  },
  {
    id: "lifecycle",
    term: "项目生命周期",
    category: "methodology",
    summary: "项目从启动到收尾经历的阶段序列，形态由开发方法决定。",
    detail: [
      "预测型生命周期按阶段顺序推进（需求→设计→开发→验证→交付），每阶段末有评审关口；敏捷型以固定长度的迭代循环推进，每轮都产出可用增量。",
      "生命周期没有优劣，只有匹配：需求明确、变更昂贵选预测型；需求探索中、反馈价值大选敏捷；多数中小团队的真实形态是混合型。"
    ],
    inProHelm: "创建/编辑项目时选择开发方法即确定生命周期形态，方法论裁剪器据此预置流程集。",
    related: ["predictive", "agile", "hybrid"],
  },
  {
    id: "predictive",
    term: "预测型",
    abbr: "Predictive / 瀑布",
    category: "methodology",
    summary: "范围、进度、成本前期定基线，按计划顺序执行的交付方式。",
    detail: [
      "预测型（俗称瀑布）的优势是计划性强、进度可预测、便于对外承诺；代价是前期必须投入大量规划，且变更成本随项目推进快速上升。",
      "适合需求稳定、合规或验收标准清晰的工程类、实施类项目。"
    ],
    inProHelm: "项目开发方法选「预测型」后，裁剪器会去掉迭代类流程项，重点保留基线与变更控制相关的实践。",
    related: ["lifecycle", "hybrid", "tailoring"],
  },
  {
    id: "agile",
    term: "敏捷型",
    abbr: "Agile",
    category: "methodology",
    summary: "以短周期迭代持续交付增量价值，拥抱变更、依赖反馈。",
    detail: [
      "敏捷不是\"没有计划\"，而是\"滚动式地计划\"：每轮迭代基于最新信息细化范围，通过可运行增量获取真实反馈再调整方向。",
      "对工具的要求从\"详尽的甘特图\"转向\"透明的任务流转与优先级\"。"
    ],
    inProHelm: "开发方法选「敏捷」后，看板流转是主要执行界面；AI 拆解 WBS 时会按增量交付的思路划分任务。",
    related: ["lifecycle", "kanban", "hybrid"],
  },
  {
    id: "hybrid",
    term: "混合型",
    abbr: "Hybrid",
    category: "methodology",
    summary: "预测型的阶段骨架 + 敏捷型的局部迭代，中小团队最常见的形态。",
    detail: [
      "典型混合形态：里程碑、对外承诺与验收用预测型管理；团队日常开发用看板/迭代推进。",
      "混合型的关键是说清楚\"哪些层用哪种方法\"，避免两头都松。"
    ],
    inProHelm: "创建项目时的默认选项；方法论裁剪器允许逐项勾选，把两套实践组合成适合团队的最小集。",
    related: ["predictive", "agile", "tailoring"],
  },
  {
    id: "tailoring",
    term: "方法论裁剪",
    category: "methodology",
    summary: "按项目规模与方法选择性采纳流程与工件——不是每件事都要做，但每件事都要想过。",
    detail: [
      "裁剪（tailoring）是 PMBOK 强调的核心原则之一：5 人内部工具项目和 50 人的对外交付项目，需要的流程重量完全不同。",
      "裁剪要回答两个问题：哪些工件必须要有（章程、风险、基线）？哪些仪式可以简化？结论应与团队达成一致并记录。"
    ],
    inProHelm: "「启动」页签的方法论裁剪器：按开发方法预置最小必要流程集，可逐项增删，随时一键恢复推荐值。",
    related: ["pmbok", "hybrid", "focus-areas"],
  },

  // ---------- 启动与干系人 ----------
  {
    id: "charter",
    term: "项目章程",
    abbr: "Project Charter",
    category: "initiation",
    summary: "正式授权项目存在、任命项目经理并明确资源与目标边界的文件。",
    detail: [
      "章程回答四个问题：为什么做（目标与商业论证）、做成什么样（成功标准与关键交付物）、约束是什么（预算/工期/假设）、项目经理有多大事权。",
      "章程由发起人发布而不是项目经理自己写给自己——它是对外的承诺，也是后续所有变更判断的参照系。",
      "没有章程就开工的项目，最常见的结局是范围与期望漂移。"
    ],
    inProHelm: "「启动」页签章程编辑器：8 节结构化编辑；AI 舵手可基于项目信息与干系人登记册起草初稿，采纳前可逐节修改。",
    related: ["business-case", "success-criteria", "pmbok"],
  },
  {
    id: "business-case",
    term: "商业论证",
    abbr: "Business Case",
    category: "initiation",
    summary: "回答\"值不值得做\"：收益、成本、替代方案与可行性论证。",
    detail: [
      "商业论证在项目启动前编写，是立项决策的依据；它关注的不是\"怎么做\"而是\"为什么做、不做的代价是什么、还有没有更优路径\"。",
      "项目过程中若商业论证失效（收益消失、前提变化），及时止损关闭项目也是正确决策。"
    ],
    inProHelm: "创建项目时的「项目简述」与章程的目标节承载其要点；建议用一句话写清收益，再展开关键数字。",
    related: ["charter", "success-criteria"],
  },
  {
    id: "success-criteria",
    term: "成功标准",
    category: "initiation",
    summary: "判断项目成功与否的量化标准——写得清才验收得动。",
    detail: [
      "成功标准应当可度量（\"上线后月活 5 万\"而非\"提升用户活跃\"），覆盖交付、时间、成本与收益多个维度。",
      "收尾阶段的验收争议，多数源于启动时没有把成功标准写清楚。"
    ],
    inProHelm: "章程的「成功标准」节；收尾页签复盘时逐条对照。",
    related: ["charter", "acceptance"],
  },
  {
    id: "stakeholder",
    term: "干系人",
    abbr: "Stakeholder",
    category: "initiation",
    summary: "能影响项目或被项目影响的个人与组织，早识别早管理。",
    detail: [
      "干系人包括客户、赞助人、高层、团队成员、职能部门，也可能是监管机构或终端用户。",
      "干系人管理的常见失误是\"只盯住的、嗓门大的\"——低权力高利益的用户群体同样能让项目翻车。识别要趁早，管理是持续动作。"
    ],
    inProHelm: "「启动」页签干系人登记册；AI 起草章程时会自动参考登记册中的关键干系人与期望。",
    related: ["stakeholder-register", "power-interest-grid"],
  },
  {
    id: "stakeholder-register",
    term: "干系人登记册",
    category: "initiation",
    summary: "记录干系人身份、期望、影响力与应对策略的结构化清单。",
    detail: [
      "登记册的最低要求：姓名/角色、期望与关切、权力与利益评级、管理策略。",
      "它是活的文档——随项目推进（新部门介入、领导层变动）要持续更新，而不是启动时填一次就归档。"
    ],
    inProHelm: "「启动」页签登记册：以 1~5 标度评价权力与利益，系统自动推导四象限管理策略。",
    related: ["stakeholder", "power-interest-grid"],
  },
  {
    id: "power-interest-grid",
    term: "权力/利益方格",
    abbr: "Power/Interest Grid",
    category: "initiation",
    summary: "按权力×利益把干系人放入四象限，据此分配沟通精力。",
    detail: [
      "四象限策略：高权力高利益 → 重点管理（共决策）；高权力低利益 → 令其满意（少打扰但要安心）；低权力高利益 → 随时告知；低权力低利益 → 监控即可。",
      "方格是动态的：项目进入不同阶段，同一干系人的位置可能移动。"
    ],
    inProHelm: "登记册中的 1~5 评分自动落位四象限并显示策略建议。",
    related: ["stakeholder-register", "stakeholder"],
  },

  // ---------- 规划与进度 ----------
  {
    id: "wbs",
    term: "工作分解结构",
    abbr: "WBS",
    category: "planning",
    summary: "把项目逐层分解为可管理、可分配的工作单元，是进度与成本的骨架。",
    detail: [
      "WBS（Work Breakdown Structure）按可交付成果分解项目，直到每个末级节点责任清晰、工作量可估算。",
      "两条铁律：100% 原则（子节点之和恰好覆盖父节点，不多不少）；WBS 表达的是\"交付什么\"而不是\"怎么做\"（那是活动清单的事）。",
      "WBS 缺失的项目，任务分配与挣值度量都缺少基准。"
    ],
    inProHelm: "「规划」页签列表视图直接编辑 WBS 树：缩进/升级调整层级，编码（1、1.1、1.2.3）自动维护；AI 舵手可从需求文本生成两级草稿。",
    related: ["work-package", "baseline", "cpm"],
  },
  {
    id: "work-package",
    term: "工作包",
    abbr: "Work Package",
    category: "planning",
    summary: "WBS 最底层的工作单元，责任明确、工期与成本可估算。",
    detail: [
      "经验法则（8/80 原则）：工作包工作量以小时到天为量级——太小管理成本高，太大无法跟踪。",
      "每个工作包应有唯一责任人；挣值管理中的预算与实际成本都以工作包为最小单位。"
    ],
    inProHelm: "WBS 的叶子任务即工作包：只有叶子任务参与挣值计价与工时登记，父级行只做汇总。",
    related: ["wbs", "evm"],
  },
  {
    id: "rolling-wave",
    term: "滚动式规划",
    abbr: "Rolling Wave",
    category: "planning",
    summary: "近期分解到细、远期保持粗粒度，随信息增多逐步细化。",
    detail: [
      "对远期工作做精确排期，通常是在为假数据投入管理成本。滚动式规划允许 1~2 个月内的任务精确到天，更远的只列交付物。",
      "它与基线管理并不冲突：基线记录\"当时的计划\"，滚动细化通过变更与重设基线体现。"
    ],
    inProHelm: "远期任务可以先只写标题，近期任务填起止日期与工期；阶段目标达成后保存新基线。",
    related: ["wbs", "baseline"],
  },
  {
    id: "milestone",
    term: "里程碑",
    abbr: "Milestone",
    category: "planning",
    summary: "零工期的重要时间节点——关键交付、评审或决策点。",
    detail: [
      "里程碑是进度\"路标\"：它本身不消耗工作量，但定义了\"到这一天必须证明什么\"。",
      "里程碑通常对应外部承诺（发布、验收、签约），应显著少于任务数量，否则失去路标意义。"
    ],
    inProHelm: "任务表单勾选「里程碑」，甘特图以深色菱形显示；里程碑不计入挣值预算（零工期零预算）。",
    related: ["wbs", "cpm"],
  },
  {
    id: "dependency-types",
    term: "依赖关系",
    abbr: "FS / SS / FF / SF",
    category: "planning",
    summary: "任务间先后约束的四种逻辑，FS（完成后开始）最常用。",
    detail: [
      "FS（Finish-to-Start）：前置完成，后续才能开始——最直觉的顺序约束。",
      "SS（Start-to-Start）：两者同时开始，常用于并行工作；FF（Finish-to-Finish）：同时完成；SF（Start-to-Finish）：少见，如\"接班人开始值班才算交接完成\"。",
      "误用依赖类型是排期失真的常见来源：该用 SS 并行的硬排成 FS 串行，工期凭空翻倍。"
    ],
    inProHelm: "创建任务时选择前置任务（FS）；甘特图绘制依赖连线；系统带环检测，成环会被拒绝。",
    related: ["cpm", "lag-lead", "wbs"],
  },
  {
    id: "lag-lead",
    term: "滞后量与提前量",
    abbr: "Lag / Lead",
    category: "planning",
    summary: "依赖上的时间偏移：滞后是等待，提前是重叠并行。",
    detail: [
      "滞后（lag）：依赖成立后的强制等待，如\"浇筑完成 → 滞后 3 天 → 拆模\"。",
      "提前（lead）：让后续工作提前介入，如\"开发开始 → 提前 2 天 → 测试用例设计开始\"。",
      "合理使用滞后/提前能把串行链条压缩成并行网络，但每一处偏移都要有现实依据。"
    ],
    inProHelm: "依赖的滞后天数字段：正数为滞后；提前量填负数。",
    related: ["dependency-types", "cpm"],
  },
  {
    id: "cpm",
    term: "关键路径法",
    abbr: "CPM",
    category: "planning",
    summary: "计算依赖网络中最长的任务链——它决定项目最短工期。",
    detail: [
      "关键路径上的任务总浮动时间为零：任何一项延误，项目总工期随之延误。",
      "计算过程：正向推算每项任务的最早开始/结束，反向推算最晚开始/结束，两者之差即总浮动时间；浮动为零的任务串成关键路径。",
      "关键路径可能多条，也可能随进度推移而改变——压缩工期时应优先盯关键路径上的任务。"
    ],
    inProHelm: "甘特视图自动计算 CPM 并以红色高亮关键路径，普通任务以蓝色显示；日/周/月切换观察粒度。",
    related: ["float", "dependency-types", "baseline"],
  },
  {
    id: "float",
    term: "浮动时间",
    abbr: "Total Float",
    category: "planning",
    summary: "任务在不影响项目完工日期前提下的最大可推迟量。",
    detail: [
      "总浮动时间 = 最晚开始 − 最早开始。关键路径上的任务浮动为 0。",
      "浮动是共享资源：吃掉某条非关键路径上的浮动，可能让别的任务变成新的关键路径。",
      "管理启示：浮动大的任务可以适当延后腾资源，但要持续监控剩余浮动。"
    ],
    inProHelm: "甘特图中蓝色条形即有浮动的任务（红色为关键路径）；排程数据包含最早/最晚开始与总浮动。",
    related: ["cpm", "milestone"],
  },
  {
    id: "baseline",
    term: "计划基线",
    abbr: "Baseline",
    category: "planning",
    summary: "经批准的进度\"照片\"——保存时的任务与依赖结构快照，绩效对比的基准。",
    detail: [
      "基线的意义：让\"延期了多少、成本超了多少\"有客观参照。没有基线的绩效汇报全是感觉。",
      "基线一经确认不应随意改动；计划确实变更时保存新基线并注明原因，保留版本对比的能力。",
      "常见反模式：进度落后后反复重设基线，让\"延期\"永远不发生。"
    ],
    inProHelm: "「监控」页签保存基线；挣值管理默认对比最新基线，可在下拉中切换历史版本。",
    related: ["evm", "cpm", "spi"],
  },
  {
    id: "scope-creep",
    term: "范围蔓延",
    abbr: "Scope Creep",
    category: "planning",
    summary: "未经变更控制的需求增量悄悄堆进项目，拖垮进度与预算。",
    detail: [
      "范围蔓延的可怕在于\"每一单看都很小\"：加个按钮、多支持一种格式……累积起来就是数周的工作量。",
      "对策不是拒绝变更，而是让变更可见：任何新需求都评估工期影响，走正式确认并更新计划。",
      "与\"镀金\"（gold plating）区分：蔓延来自外部需求，镀金是团队自愿加戏。"
    ],
    inProHelm: "新需求先补进 WBS 并评估工期，必要时保存新基线——变更是否被接受应当看得到。",
    related: ["baseline", "wbs"],
  },

  // ---------- 成本与挣值 ----------
  {
    id: "evm",
    term: "挣值管理",
    abbr: "EVM",
    category: "evm",
    summary: "把计划值、完成的工作价值、实际花费放进同一坐标系，一次回答进度与成本两个问题。",
    detail: [
      "EVM 的三个基础量：PV（按计划该完成多少）、EV（实际完成的工作按预算折算值多少）、AC（实际花了多少）。",
      "由它们派生出四个判断：SV = EV − PV（进度差多少）、CV = EV − AC（成本差多少）、SPI = EV ÷ PV（进度效率）、CPI = EV ÷ AC（成本效率）。",
      "EVM 的独特价值：进度偏差与成本偏差被统一折算成\"钱的量纲\"（或人时），第一次让\"又快又省\"可以同时度量。",
      "使用前提：有基线、任务有预算（工时）、按期登记实际成本。"
    ],
    inProHelm: "「监控」页签 EVM 卡片：S 曲线 + 指标卡 + 工时台账；价值单位为人时。",
    related: ["pv", "ev", "ac", "spi", "cpi", "baseline"],
  },
  {
    id: "pv",
    term: "计划价值",
    abbr: "PV · Planned Value",
    category: "evm",
    summary: "到某时点\"按计划应该完成多少工作\"的预算值。",
    detail: [
      "PV 由基线决定：每项任务把预算工时按起止日期逐日均摊，累计成计划 S 曲线。",
      "PV 是评判进度的尺子——没有 PV 的\"完成率 60%\"无法回答\"是快了还是慢了\"。",
      "统计截止日之后的 PV 仍然继续延伸（计划还没走完），这正是 S 曲线\"未来部分\"的意义。"
    ],
    inProHelm: "由基线任务的起止日期与预估工时自动逐日均摊计算；未排日期的任务不计入 PV 曲线（有提示）。",
    related: ["evm", "baseline"],
  },
  {
    id: "ev",
    term: "挣值",
    abbr: "EV · Earned Value",
    category: "evm",
    summary: "实际完成的工作按计划预算折算的价值——完成即\"挣到\"该任务的预算。",
    detail: [
      "EV 是 EVM 的灵魂：它把\"完成了多少工作\"与\"计划认为它值多少\"绑定，同时反映进度与成本两个维度。",
      "注意 EV 与\"完成率\"的区别：完成率只看数量，EV 还考虑工作的价值权重——先干便宜任务的活，完成率高但挣值未必好看。",
      "EV 永远按基线预算计价，任务中途改预估不影响已挣得的价值。"
    ],
    inProHelm: "任务标记完成即挣得其基线预算工时；基线之后新增的任务按当前预估计价。",
    related: ["evm", "pv", "ac"],
  },
  {
    id: "ac",
    term: "实际成本",
    abbr: "AC · Actual Cost",
    category: "evm",
    summary: "到目前为止实际投入的成本，ProHelm 以登记工时（人时）计。",
    detail: [
      "AC 的准确性取决于记录纪律：按日登记、及时补录。事后补一个大数，曲线会失真。",
      "金额型成本（采购、外包）可按比例折算为人时口径，或与工时并行统计——alpha 版先用人时口径保持简单。"
    ],
    inProHelm: "「监控」页签「登记工时」：选任务、日期、小时数（支持补录）；台账即 AC 数据源，同时累计到任务实际工时与驾驶舱人员负荷。",
    related: ["evm", "ev"],
  },
  {
    id: "bac",
    term: "完工预算",
    abbr: "BAC · Budget at Completion",
    category: "evm",
    summary: "全部计划工作的预算总和，EVM 指标的常客分母。",
    detail: [
      "BAC = 基线内所有叶子任务预算之和。它回答\"这个项目总共值多少工时\"。",
      "EV ÷ BAC = 挣值法完成率；EAC 与 VAC 也都围绕 BAC 推算。",
      "基线变更（增减范围）后 BAC 随之变化——这也是为什么基线要版本化管理。"
    ],
    inProHelm: "基线叶子任务预算工时合计自动计算；项目表单另有金额型 BAC 字段备用（单价换算留后续版本）。",
    related: ["evm", "eac", "baseline"],
  },
  {
    id: "spi",
    term: "进度绩效指数",
    abbr: "SPI · Schedule Performance Index",
    category: "evm",
    summary: "EV ÷ PV：<1 进度落后，>1 超前；工程进度效率的\"汇率\"。",
    detail: [
      "SPI = 0.85 的含义：按当前效率，计划里 10 天的活实际要 11.8 天。",
      "两个注意点：接近完工时 EV 趋近 BAC 而 PV 封顶，SPI 会\"自动变好\"失真，此时应看关键路径；SPI 不区分\"干的活值不值钱\"，要配合 CPI 一起读。",
      "对应的绝对量是 SV = EV − PV，汇报趋势时两者结合更直观。"
    ],
    inProHelm: "EVM 指标卡：≥1 绿色 / 0.95~1 琥珀 / <0.95 红色；配合 S 曲线看趋势而不是只看单点。",
    related: ["cpi", "evm", "cpm"],
  },
  {
    id: "cpi",
    term: "成本绩效指数",
    abbr: "CPI · Cost Performance Index",
    category: "evm",
    summary: "EV ÷ AC：<1 表示每挣 1 人时实际花费超出预算。",
    detail: [
      "CPI = 0.8 的含义：每计划 1 人时的活，实际花了 1.25 人时。",
      "研究经验表明 CPI 在项目早期形成后倾向稳定——CPI < 0.8 的项目很少能完全恢复，早期干预价值最大。",
      "对应绝对量 CV = EV − AC；CPI 还直接用于推算完工估算 EAC。"
    ],
    inProHelm: "EVM 指标卡；EAC = BAC ÷ CPI 由系统按当前效率自动推算。",
    related: ["spi", "eac", "evm"],
  },
  {
    id: "eac",
    term: "完工估算",
    abbr: "EAC · Estimate at Completion",
    category: "evm",
    summary: "按当前绩效推算的最终总花费，超支预警的主指标。",
    detail: [
      "最常用公式：EAC = BAC ÷ CPI（假设剩余工作延续当前成本效率）。",
      "其他情形：原预算已失真时用\"重新自下而上估算\"；效率问题是一次性的（如返工）可用 AC + (BAC − EV)。",
      "EAC 本身不是坏消息——它让你在花超之前 6 周就知道会花超，这是它最大的价值。"
    ],
    inProHelm: "EVM 指标卡：有实际成本数据时按 CPI 法，否则回退 AC + (BAC − EV)；EAC > BAC 时红色提示，VAC 显示缺口金额。",
    related: ["cpi", "bac", "vac"],
  },
  {
    id: "vac",
    term: "完工偏差",
    abbr: "VAC · Variance at Completion",
    category: "evm",
    summary: "BAC − EAC：负值即按当前趋势预计超支的数额。",
    detail: [
      "VAC 把\"会超支\"翻译成\"超多少\"，是与赞助人沟通预算缺口最直接的数字。",
      "VAC 转正需要 CPI 回升——意味着要找到效率提升或范围收缩的抓手，而不是祈祷。"
    ],
    inProHelm: "EVM 指标卡的 EAC 卡片 hint 中展示（按当前 CPI 推算 · VAC x h）。",
    related: ["eac", "cpi", "bac"],
  },

  // ---------- 风险 ----------
  {
    id: "risk-register",
    term: "风险登记册",
    abbr: "Risk Register",
    category: "risk",
    summary: "记录\"可能发生的不确定性\"的活文档：描述、概率、影响、应对与责任人。",
    detail: [
      "风险登记册不是启动时填一次的作业，而是每周看一眼的仪表盘：新风险进来、旧风险关闭、状态持续更新。",
      "一条合格的风险描述 = 风险事件 + 不确定性（\"因 XX 原因，可能导致 XX\"），而不是把已经发生的问题写在里面。",
      "每条风险都应有 owner——没有责任人的风险等于没人管。"
    ],
    inProHelm: "「规划」页签风险登记册：R 编号自动维护、五类应对策略、责任人与预警信号字段；驾驶舱风险雷达跨项目聚合未关闭风险。",
    related: ["risk-score", "risk-strategies", "issue-vs-risk"],
  },
  {
    id: "risk-score",
    term: "概率×影响矩阵",
    abbr: "P×I Matrix",
    category: "risk",
    summary: "概率(1~5) × 影响(1~5) = 1~25 评分，把风险按严重度排序分优先级。",
    detail: [
      "矩阵的意义是区分\"知道就行\"与\"必须行动\"：低分风险留档观察，高分风险进入应对计划并分配资源。",
      "概率与影响的标度要在项目内统一（影响按工时/金额/声誉定档），否则矩阵失去可比性。",
      "评分会随风险应对推进而变化——减轻措施落地后应复评。"
    ],
    inProHelm: "风险登记册的 5×5 热力图；评分 = 概率 × 影响，驾驶舱风险雷达按评分跨项目降序排列。",
    related: ["risk-register", "risk-strategies"],
  },
  {
    id: "risk-strategies",
    term: "应对策略五式",
    category: "risk",
    summary: "威胁的五种应对：规避、转移、减轻、接受、上报。",
    detail: [
      "规避（avoid）：改计划绕开风险源；转移（transfer）：外包/保险把损失转给第三方；减轻（mitigate）：降低概率或影响；接受（accept）：留应急储备不做主动干预；上报（escalate）：超出项目权限交由上层决策。",
      "选择依据是性价比：应对成本 ≤ 预期损失才值得动手。机会型风险（正向不确定）同样有对应策略，alpha 版聚焦威胁侧。"
    ],
    inProHelm: "风险表单的应对策略下拉（默认减轻）+ 应对计划文本字段。",
    related: ["risk-register", "risk-score"],
  },
  {
    id: "risk-trigger",
    term: "触发条件",
    abbr: "Trigger / 预警信号",
    category: "risk",
    summary: "风险即将变成现实的可观察征兆，定义了它才能及时启动应对。",
    detail: [
      "好的触发条件是可观察、可度量的：\"关键开发人员连续两周加班 >20 小时\"比\"团队状态不佳\"有用得多。",
      "触发条件相当于给风险装了传感器——没有它，应对计划只会在损失发生后才被想起。"
    ],
    inProHelm: "风险表单的「预警信号」字段。",
    related: ["risk-register", "issue-vs-risk"],
  },
  {
    id: "issue-vs-risk",
    term: "风险与问题",
    category: "risk",
    summary: "风险是尚未发生的可能性，问题是已经发生的事实；风险发生即转为问题。",
    detail: [
      "两者的管理动作不同：风险做预防与准备，问题做应急与止损。混在一个口径里会导致\"预防性动作\"被\"救火\"挤掉。",
      "风险发生的正确动作：标记状态、启动应急计划、复盘为什么预警没起作用。"
    ],
    inProHelm: "风险状态标记「已发生」后仍留在登记册但以红色标注，驾驶舱雷达同步提示——事实台账与预测台账合一管理。",
    related: ["risk-register", "risk-trigger"],
  },

  // ---------- 执行与监控 ----------
  {
    id: "kanban",
    term: "看板",
    abbr: "Kanban",
    category: "execution",
    summary: "以\"待办→进行中→已完成\"可视化任务流转的执行方法，源自精益生产。",
    detail: [
      "看板的核心不是三列卡片，而是\"让流动可见\"：瓶颈在哪里、什么被积压、什么在等待。",
      "敏捷与混合项目常用看板做日常执行层，配合迭代评审校准方向。",
      "进阶实践是在制品限制（WIP limit）——限制\"进行中\"列的数量，逼团队先完成再开始。"
    ],
    inProHelm: "「执行」页签看板支持拖拽流转（待办→进行中→已完成）；完成率统计与驾驶舱联动。",
    related: ["agile", "wbs"],
  },

  // ---------- 收尾 ----------
  {
    id: "acceptance",
    term: "验收",
    abbr: "Acceptance",
    category: "closing",
    summary: "客户/赞助人对照成功标准确认交付物\"可接受\"的正式动作。",
    detail: [
      "验收争议的根源多数在启动阶段：成功标准没写清、验收流程没约定。",
      "验收应当逐条对照事先定义的标准，形成书面确认——口头\"挺好\"不构成收尾依据。",
      "提前约定验收方式（演示、试运行、测试报告）能省掉收尾阶段的大量扯皮。"
    ],
    inProHelm: "章程的「成功标准」与「关键交付物」节是验收依据；收尾页签复盘对照。",
    related: ["success-criteria", "administrative-closure"],
  },
  {
    id: "lessons-learned",
    term: "经验教训",
    abbr: "Lessons Learned",
    category: "closing",
    summary: "总结\"什么有效、什么无效、下次怎么办\"——不沉淀等于同一个坑掉两次。",
    detail: [
      "经验教训的常见失败形态：收尾会上大家客气一场，纪要没人再看。",
      "让教训可用的关键是写成可执行的变更：\"下次投标前先找法务审模板\"比\"沟通要更早\"有用。",
      "教训属于组织而非个人——沉淀进组织过程资产才叫复利。"
    ],
    inProHelm: "「收尾」页签记录经验教训；配合报告导出沉淀为组织文档。",
    related: ["administrative-closure", "acceptance"],
  },
  {
    id: "administrative-closure",
    term: "行政收尾",
    category: "closing",
    summary: "正式关闭项目的动作：验收确认、合同结清、文档归档、释放资源、宣布关闭。",
    detail: [
      "项目\"做完\"不等于\"收尾完\"：没有行政收尾，团队成员会被\"僵尸项目\"持续占用，账目与合同悬着。",
      "行政收尾的清单感很强：验收文件、财务结清、资料归档、干系人通知、庆功与复盘。",
      "宣布关闭是正式动作——它给团队一个明确的心理终点。"
    ],
    inProHelm: "在「编辑项目」对话框中把项目状态改为「已关闭」，即不再计入进行中与活跃统计；收尾页签提供复盘要点。",
    related: ["acceptance", "lessons-learned"],
  },
];

export const kbTermById: Map<string, KbTerm> = new Map(KB_TERMS.map((t) => [t.id, t]));

/** 搜索过滤：命中术语/缩写/摘要/详解/界面指引任一即返回。 */
export function searchKbTerms(query: string): KbTerm[] {
  const q = query.trim().toLowerCase();
  if (!q) return KB_TERMS;
  return KB_TERMS.filter((t) =>
    [t.term, t.abbr ?? "", t.summary, t.detail.join("\n"), t.inProHelm]
      .join("\n")
      .toLowerCase()
      .includes(q),
  );
}
