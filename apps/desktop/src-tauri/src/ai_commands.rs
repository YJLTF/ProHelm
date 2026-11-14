//! AI 命令：设置与连接测试、舵手问答（流式）、章程/WBS 草稿生成、溯源记录。
//!
//! 产品铁律的落实位置：一切草稿先落 AiArtifact（status=draft），采纳/弃用
//! 由前端显式调用 set_artifact_status——AI 永远不直接改库。

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;
use tauri::ipc::Channel;

use helm_ai::AiProviderConfig;
use helm_ai::gateway::{self, Message};
use helm_core::models::{AiArtifactKind, AiArtifactStatus, CharterContent};
use helm_core::repo::{ai_artifacts, projects, risks, settings, stakeholders};

use crate::AppState;
use crate::commands::CommandError;

type CmdResult<T> = Result<T, CommandError>;

// ---------- 提示词（system） ----------

const COACH_SYSTEM: &str = "你是「项舵 ProHelm」内置的 PMP 项目管理教练，服务对象是中小团队的项目经理。回答规则：1) 优先基于给定的知识条目作答，并注明引用的条目名称；2) 知识条目未覆盖时，可给出通用项目管理最佳实践，但须注明『超出内置知识库』；3) 回答简洁、分点、可执行，直接给做法和例子；4) 全中文。";

const CHARTER_SYSTEM: &str = "你是资深项目管理专家。根据项目信息起草项目章程。只输出一个 JSON 对象，不要任何解释、前后缀或代码围栏。字段全部为中文文本字符串：objective（项目目的与目标）、successCriteria（可度量的成功标准）、keyDeliverables（主要交付物，顿号分隔）、milestones（关键里程碑，带相对周次）、constraints（制约因素）、assumptions（假设条件）、highLevelRisks（3~5 条高层级风险，分号分隔）、pmAuthority（项目经理职权）。内容要具体、可执行，贴合项目信息，不要空话。";

const WBS_SYSTEM: &str = "你是资深项目管理专家。根据需求描述生成 WBS 分解草稿。只输出一个 JSON 数组，不要解释或代码围栏。元素格式：{\"title\":\"任务名\",\"durationDays\":数字,\"children\":[{\"title\":\"子任务名\",\"durationDays\":数字}]}。要求：顶层 4~8 个、最多两级、覆盖需求的主要交付路径、工期 0.5~15 天、全中文、可执行。";

// ---------- 共享构件 ----------

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiSettingsView {
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub has_api_key: bool,
}

fn from_ai(e: helm_ai::AiError) -> CommandError {
    CommandError {
        message: e.to_string(),
    }
}

fn load_ai_config(db: &helm_core::db::Db) -> CmdResult<AiProviderConfig> {
    let base_url = settings::get(&db.conn, "ai.base_url")?
        .filter(|s| !s.trim().is_empty())
        .ok_or(CommandError {
            message: "AI 尚未配置：请先在「设置 → AI 模型」填写接口地址与模型，并保存 API Key"
                .into(),
        })?;
    let model = settings::get(&db.conn, "ai.model")?
        .filter(|s| !s.trim().is_empty())
        .ok_or(CommandError {
            message: "AI 尚未配置：缺少模型名称".into(),
        })?;
    let api_key = helm_ai::keys::load_api_key()
        .map_err(from_ai)?
        .filter(|s| !s.is_empty())
        .ok_or(CommandError {
            message: "AI 尚未配置：缺少 API Key（保存在系统钥匙串）".into(),
        })?;
    Ok(AiProviderConfig {
        base_url,
        api_key,
        model,
    })
}

// ---------- 设置 ----------

#[tauri::command]
pub fn get_ai_settings(state: State<AppState>) -> CmdResult<AiSettingsView> {
    let db = state.db.lock().map_err(|_| lock_error())?;
    let has_api_key = helm_ai::keys::load_api_key()
        .map_err(from_ai)?
        .is_some_and(|k| !k.is_empty());
    Ok(AiSettingsView {
        base_url: settings::get(&db.conn, "ai.base_url")?,
        model: settings::get(&db.conn, "ai.model")?,
        has_api_key,
    })
}

/// `api_key`: None = 不修改；Some("") = 删除；Some(k) = 保存。
#[tauri::command]
pub fn set_ai_settings(
    state: State<AppState>,
    base_url: String,
    model: String,
    api_key: Option<String>,
) -> CmdResult<AiSettingsView> {
    let trimmed = base_url.trim().to_string();
    if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
        return Err(CommandError {
            message: "接口地址必须以 http(s):// 开头".into(),
        });
    }
    if model.trim().is_empty() {
        return Err(CommandError {
            message: "模型名称不能为空".into(),
        });
    }
    {
        let db = state.db.lock().map_err(|_| lock_error())?;
        settings::set(&db.conn, "ai.base_url", trimmed.trim())?;
        settings::set(&db.conn, "ai.model", model.trim())?;
    }
    if let Some(key) = api_key {
        helm_ai::keys::store_api_key(&key).map_err(from_ai)?;
    }
    get_ai_settings(state)
}

#[tauri::command]
pub async fn test_ai_connection(state: State<'_, AppState>) -> CmdResult<String> {
    let cfg = {
        let db = state.db.lock().map_err(|_| lock_error())?;
        load_ai_config(&db)?
    };
    let reply = gateway::chat(&cfg, &[Message::user("请只回复四个字：连接成功")])
        .await
        .map_err(from_ai)?;
    Ok(reply)
}

fn lock_error() -> CommandError {
    CommandError {
        message: "数据库连接不可用".into(),
    }
}

// ---------- 舵手问答（流式） ----------

/// 流式事件：delta 增量文本 / done 结束。
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum CopilotEvent {
    Delta { text: String },
    Done,
}

#[tauri::command]
pub async fn coach_ask(
    state: State<'_, AppState>,
    channel: Channel<CopilotEvent>,
    question: String,
    project_id: Option<String>,
) -> CmdResult<String> {
    if question.trim().is_empty() {
        return Err(CommandError {
            message: "问题不能为空".into(),
        });
    }

    // 1. 同步段：读配置 + 检索知识库 + 组装上下文（不跨 await 持锁）
    let (cfg, messages) = {
        let db = state.db.lock().map_err(|_| lock_error())?;
        let cfg = load_ai_config(&db)?;
        let entries = helm_ai::kb::retrieve(&question, 3);
        let kb_block = if entries.is_empty() {
            "（知识库未命中相关条目，请基于通用实践回答并注明）".to_string()
        } else {
            entries
                .iter()
                .map(|e| format!("【{}】{}", e.title, e.body))
                .collect::<Vec<_>>()
                .join("\n---\n")
        };
        let mut user_text = format!("知识库检索结果：\n{kb_block}\n\n用户问题：{question}");
        if let Some(pid) = &project_id
            && let Ok(p) = projects::get(&db.conn, pid)
        {
            let task_count: i64 = db
                .conn
                .query_row(
                    "SELECT COUNT(*) FROM tasks WHERE project_id = ?1",
                    [&p.id],
                    |r| r.get(0),
                )
                .unwrap_or(0);
            user_text.push_str(&format!(
                "\n\n【当前项目上下文】名称：{}；开发方法：{:?}；状态：{:?}；任务数：{}",
                p.name, p.method, p.status, task_count
            ));
        }
        (
            cfg,
            vec![Message::system(COACH_SYSTEM), Message::user(user_text)],
        )
    };

    // 2. 流式调用（不持锁）
    let channel_for_stream = channel.clone();
    let mut on_delta = move |d: &str| {
        let _ = channel_for_stream.send(CopilotEvent::Delta {
            text: d.to_string(),
        });
    };
    let answer = gateway::chat_stream(&cfg, &messages, &mut on_delta)
        .await
        .map_err(from_ai)?;

    // 3. 溯源落库
    let artifact_id = {
        let db = state.db.lock().map_err(|_| lock_error())?;
        ai_artifacts::create(
            &db.conn,
            AiArtifactKind::Coach,
            project_id.as_deref(),
            "openai-compatible",
            &cfg.model,
            &question,
            &answer,
        )?
        .id
    };
    let _ = channel.send(CopilotEvent::Done);
    Ok(artifact_id)
}

// ---------- 章程草稿 ----------

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CharterDraftOut {
    pub artifact_id: String,
    pub charter: CharterContent,
}

fn project_context_block(db: &helm_core::db::Db, project_id: &str) -> CmdResult<String> {
    let conn = &db.conn;
    let p = projects::get(conn, project_id)?;
    let mut block = format!(
        "项目名称：{}\n开发方法：{:?}\n项目状态：{:?}\n计划周期：{} ~ {}\n总预算：{}",
        p.name,
        p.method,
        p.status,
        p.start_date.as_deref().unwrap_or("未设置"),
        p.target_end_date.as_deref().unwrap_or("未设置"),
        if p.bac > 0.0 {
            format!("¥{:.0}", p.bac)
        } else {
            "未设置".to_string()
        },
    );
    if let Some(desc) = &p.description {
        block.push_str(&format!("\n项目简述：{desc}"));
    }
    let stakeholders = stakeholders::list_by_project(conn, project_id)?;
    if !stakeholders.is_empty() {
        let lines: Vec<String> = stakeholders
            .iter()
            .take(8)
            .map(|s| {
                format!(
                    "- {}（{}）：{}",
                    s.name,
                    s.role.as_deref().unwrap_or("角色未填"),
                    s.expectation.as_deref().unwrap_or("")
                )
            })
            .collect();
        block.push_str(&format!("\n主要干系人：\n{}", lines.join("\n")));
    }
    let risk_list = risks::list_by_project(conn, project_id)?;
    if !risk_list.is_empty() {
        let lines: Vec<String> = risk_list
            .iter()
            .take(5)
            .map(|r| format!("- {}（评分 {}）", r.title, r.probability * r.impact))
            .collect();
        block.push_str(&format!("\n已登记风险：\n{}", lines.join("\n")));
    }
    Ok(block)
}

#[tauri::command]
pub async fn generate_charter_draft(
    state: State<'_, AppState>,
    project_id: String,
) -> CmdResult<CharterDraftOut> {
    let (cfg, user_prompt) = {
        let db = state.db.lock().map_err(|_| lock_error())?;
        let cfg = load_ai_config(&db)?;
        let context = project_context_block(&db, &project_id)?;
        (cfg, format!("请为以下项目起草章程：\n{context}"))
    };

    let value: Value = gateway::chat_json(&cfg, CHARTER_SYSTEM, &user_prompt)
        .await
        .map_err(from_ai)?;
    let charter: CharterContent =
        serde_json::from_value(value.clone()).map_err(|e| CommandError {
            message: format!("AI 输出无法解析为章程结构: {e}"),
        })?;
    if [
        charter.objective.as_str(),
        charter.success_criteria.as_str(),
        charter.key_deliverables.as_str(),
        charter.milestones.as_str(),
    ]
    .iter()
    .all(|s| s.trim().is_empty())
    {
        return Err(CommandError {
            message: "AI 返回了空章程，请重试或更换模型".into(),
        });
    }

    let output = serde_json::to_string(&charter)?;
    let artifact_id = {
        let db = state.db.lock().map_err(|_| lock_error())?;
        ai_artifacts::create(
            &db.conn,
            AiArtifactKind::CharterDraft,
            Some(&project_id),
            "openai-compatible",
            &cfg.model,
            &project_id,
            &output,
        )?
        .id
    };
    Ok(CharterDraftOut {
        artifact_id,
        charter,
    })
}

// ---------- WBS 拆解草稿 ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WbsDraftItem {
    pub title: String,
    pub duration_days: f64,
    #[serde(default)]
    pub children: Vec<WbsDraftItem>,
}

fn sanitize_wbs(items: Vec<WbsDraftItem>) -> Vec<WbsDraftItem> {
    fn fix(item: WbsDraftItem, depth: usize) -> Option<WbsDraftItem> {
        let title = item.title.trim().to_string();
        if title.is_empty() {
            return None;
        }
        let duration = if item.duration_days.is_finite() {
            item.duration_days.clamp(0.5, 30.0)
        } else {
            1.0
        };
        let children: Vec<WbsDraftItem> = if depth == 0 {
            item.children
                .into_iter()
                .filter_map(|c| fix(c, depth + 1))
                .take(8)
                .collect()
        } else {
            Vec::new()
        };
        Some(WbsDraftItem {
            title,
            duration_days: duration,
            children,
        })
    }
    items
        .into_iter()
        .filter_map(|i| fix(i, 0))
        .take(10)
        .collect()
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WbsDraftOut {
    pub artifact_id: String,
    pub items: Vec<WbsDraftItem>,
}

#[tauri::command]
pub async fn generate_wbs_draft(
    state: State<'_, AppState>,
    project_id: String,
    requirements: String,
) -> CmdResult<WbsDraftOut> {
    if requirements.trim().is_empty() {
        return Err(CommandError {
            message: "请先填写需求描述".into(),
        });
    }
    let (cfg, user_prompt) = {
        let db = state.db.lock().map_err(|_| lock_error())?;
        let cfg = load_ai_config(&db)?;
        let context = project_context_block(&db, &project_id)?;
        (
            cfg,
            format!(
                "项目信息：\n{context}\n\n需求描述：\n{}",
                requirements.trim()
            ),
        )
    };

    let value: Value = gateway::chat_json(&cfg, WBS_SYSTEM, &user_prompt)
        .await
        .map_err(from_ai)?;
    let parsed: Vec<WbsDraftItem> = serde_json::from_value(value).map_err(|e| CommandError {
        message: format!("AI 输出无法解析为 WBS 结构: {e}"),
    })?;
    let items = sanitize_wbs(parsed);
    if items.is_empty() {
        return Err(CommandError {
            message: "AI 返回了空分解，请补充需求描述后重试".into(),
        });
    }

    let output = serde_json::to_string(&items)?;
    let artifact_id = {
        let db = state.db.lock().map_err(|_| lock_error())?;
        ai_artifacts::create(
            &db.conn,
            AiArtifactKind::WbsDraft,
            Some(&project_id),
            "openai-compatible",
            &cfg.model,
            &requirements,
            &output,
        )?
        .id
    };
    Ok(WbsDraftOut { artifact_id, items })
}

// ---------- 溯源记录 ----------

#[tauri::command]
pub fn list_ai_artifacts(
    state: State<AppState>,
    project_id: Option<String>,
) -> CmdResult<Vec<helm_core::AiArtifact>> {
    let db = state.db.lock().map_err(|_| lock_error())?;
    ai_artifacts::list(&db.conn, project_id.as_deref()).map_err(Into::into)
}

#[tauri::command]
pub fn set_artifact_status(
    state: State<AppState>,
    id: String,
    status: AiArtifactStatus,
) -> CmdResult<helm_core::AiArtifact> {
    let db = state.db.lock().map_err(|_| lock_error())?;
    ai_artifacts::set_status(&db.conn, &id, status).map_err(Into::into)
}
