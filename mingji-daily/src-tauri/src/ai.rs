// AI 能力：DeepSeek 调用层（Key 仅存本机 config.json，前端不可见）
// 聚合数据只传统计口径，不传账单备注原文（隐私最小化）

use crate::models::{AiConfigInput, AiConfigPublic, AiConfigStore, AiParsedLine, CycleRange};
use rusqlite::{Connection, OptionalExtension};
use serde_json::{json, Value};
use std::path::PathBuf;

const API_URL: &str = "https://api.deepseek.com/chat/completions";

// ---------- 配置存储 ----------

fn config_path() -> PathBuf {
    crate::db::resolve_data_dir().join("config.json")
}

pub fn load_config() -> AiConfigStore {
    std::fs::read_to_string(config_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_config(cfg: &AiConfigStore) -> Result<(), String> {
    let dir = crate::db::resolve_data_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let text = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    std::fs::write(config_path(), text).map_err(|e| e.to_string())
}

// ---------- DeepSeek 调用 ----------

pub async fn call_deepseek(system: &str, user: &str, max_tokens: u32) -> Result<String, String> {
    let cfg = load_config();
    if cfg.api_key.trim().is_empty() {
        return Err("尚未配置 DeepSeek API Key".into());
    }
    let client = reqwest::Client::new();
    let body = json!({
        "model": cfg.model,
        "messages": [
            { "role": "system", "content": system },
            { "role": "user", "content": user }
        ],
        "temperature": 0.3,
        "max_tokens": max_tokens,
        "response_format": { "type": "json_object" },
        "stream": false
    });
    let resp = client
        .post(API_URL)
        .bearer_auth(&cfg.api_key)
        .json(&body)
        .timeout(std::time::Duration::from_secs(90))
        .send()
        .await
        .map_err(|e| format!("请求失败：{}", e))?;
    let status = resp.status();
    let text = resp
        .text()
        .await
        .map_err(|e| format!("读取响应失败：{}", e))?;
    let v: Value = serde_json::from_str(&text)
        .map_err(|_| format!("AI 返回异常（HTTP {}）：{}", status.as_u16(), truncate(&text, 200)))?;
    if !status.is_success() {
        let msg = v["error"]["message"].as_str().unwrap_or("未知错误");
        return Err(format!("AI 接口错误（HTTP {}）：{}", status.as_u16(), msg));
    }
    v["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "AI 返回内容格式异常".to_string())
}

fn truncate(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

fn today_str() -> String {
    // 与前端一致的 yyyy-mm-dd（本地时区）
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // 粗略本地日期（UTC+8 固定，国内场景足够）
    let days = (secs + 8 * 3600) / 86400;
    let (y, m, d) = civil_from_days(days as i64);
    format!("{:04}-{:02}-{:02}", y, m, d)
}

/// days since epoch → (y, m, d)（Howard Hinnant 算法）
fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn date_diff_days(start: &str, end: &str) -> i64 {
    let p1: Vec<i64> = start.split('-').filter_map(|x| x.parse().ok()).collect();
    let p2: Vec<i64> = end.split('-').filter_map(|x| x.parse().ok()).collect();
    if p1.len() != 3 || p2.len() != 3 {
        return 1;
    }
    let f = |y: i64, m: i64, d: i64| {
        let y = if m <= 2 { y - 1 } else { y };
        let era = if y >= 0 { y } else { y - 399 } / 400;
        let yoe = y - era * 400;
        let mp = (m + 9) % 12;
        let doy = (153 * mp + 2) / 5 + d - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146097 + doe - 719468
    };
    f(p2[0], p2[1], p2[2]) - f(p1[0], p1[1], p1[2]) + 1
}

// ---------- 周期聚合（只传统计口径，不传备注原文） ----------

pub fn aggregate_cycles(conn: &Connection, cycles: &[CycleRange]) -> Result<Vec<Value>, String> {
    let mut out = Vec::new();
    for c in cycles {
        let income: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(amount),0) FROM bill WHERE status=1 AND type=2 AND bill_date>=?1 AND bill_date<=?2",
                rusqlite::params![c.start, c.end],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        let expense: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(amount),0) FROM bill WHERE status=1 AND type=1 AND bill_date>=?1 AND bill_date<=?2",
                rusqlite::params![c.start, c.end],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        let recorded_days: i64 = conn
            .query_row(
                "SELECT COUNT(DISTINCT bill_date) FROM bill WHERE status=1 AND bill_date>=?1 AND bill_date<=?2",
                rusqlite::params![c.start, c.end],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;

        let mut cats: Vec<(String, i64)> = Vec::new();
        {
            let mut stmt = conn
                .prepare(
                    "SELECT COALESCE(p.name, c.name), SUM(b.amount)
                     FROM bill b JOIN category c ON b.category_id = c.id
                     LEFT JOIN category p ON c.parent_id = p.id
                     WHERE b.status=1 AND b.type=1 AND b.bill_date>=?1 AND b.bill_date<=?2
                     GROUP BY b.category_id ORDER BY SUM(b.amount) DESC LIMIT 8",
                )
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map(rusqlite::params![c.start, c.end], |r| {
                    Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
                })
                .map_err(|e| e.to_string())?;
            for row in rows {
                cats.push(row.map_err(|e| e.to_string())?);
            }
        }

        let food_id: Option<i64> = conn
            .query_row(
                "SELECT id FROM category WHERE name='餐饮' AND parent_id IS NULL LIMIT 1",
                [],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        let mut food: Vec<(String, i64)> = Vec::new();
        if let Some(fid) = food_id {
            let mut stmt = conn
                .prepare(
                    "SELECT c.name, SUM(b.amount) FROM bill b JOIN category c ON b.category_id = c.id
                     WHERE b.status=1 AND b.type=1 AND (c.parent_id=?1 OR c.id=?1)
                       AND b.bill_date>=?2 AND b.bill_date<=?3
                     GROUP BY c.id ORDER BY SUM(b.amount) DESC",
                )
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map(rusqlite::params![fid, c.start, c.end], |r| {
                    Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
                })
                .map_err(|e| e.to_string())?;
            for row in rows {
                food.push(row.map_err(|e| e.to_string())?);
            }
        }

        let total_days = date_diff_days(&c.start, &c.end).max(1);
        let avg_daily = (expense as f64 / total_days as f64 / 100.0 * 100.0).round() / 100.0;
        out.push(json!({
            "label": format!("{} ~ {}", c.start, c.end),
            "days": total_days,
            "recorded_days": recorded_days,
            "income": income as f64 / 100.0,
            "expense": expense as f64 / 100.0,
            "balance": (income - expense) as f64 / 100.0,
            "avg_daily": avg_daily,
            "categories": cats.iter().map(|(n, a)| json!({
                "name": n,
                "amount": *a as f64 / 100.0,
                "pct": if expense > 0 { ((*a as f64 / expense as f64) * 1000.0).round() / 10.0 } else { 0.0 }
            })).collect::<Vec<_>>(),
            "food": food.iter().map(|(n, a)| json!({"name": n, "amount": *a as f64 / 100.0})).collect::<Vec<_>>(),
        }));
    }
    Ok(out)
}

// ---------- 命令 ----------

#[tauri::command]
pub fn get_ai_config() -> AiConfigPublic {
    let cfg = load_config();
    AiConfigPublic {
        enabled: cfg.enabled,
        has_key: !cfg.api_key.trim().is_empty(),
        model: cfg.model,
    }
}

#[tauri::command]
pub async fn set_ai_config(input: AiConfigInput) -> Result<AiConfigPublic, String> {
    let mut cfg = load_config();
    if let Some(e) = input.enabled {
        cfg.enabled = e;
    }
    if let Some(k) = input.api_key {
        cfg.api_key = k.trim().to_string();
    }
    if let Some(m) = input.model {
        if !m.trim().is_empty() {
            cfg.model = m.trim().to_string();
        }
    }
    save_config(&cfg)?;
    Ok(AiConfigPublic {
        enabled: cfg.enabled,
        has_key: !cfg.api_key.trim().is_empty(),
        model: cfg.model,
    })
}

#[tauri::command]
pub async fn test_ai_connection() -> Result<String, String> {
    // 注意：response_format=json_object 要求提示词包含 "json" 字样，且输出截断会返回 400
    let _ = call_deepseek(
        "你是连接测试助手。收到消息后只输出以下 JSON，不要输出其他内容：{\"ok\": true}",
        "ping",
        256,
    )
    .await?;
    let cfg = load_config();
    Ok(format!("连接成功（模型：{}）", cfg.model))
}

const PARSE_SYSTEM: &str = "你是账单解析助手。把每行原始账单文本解析为结构化字段。只输出合法 JSON，格式：{\"items\":[{\"line\":整数, 原始行序号从0开始, \"date\":\"YYYY-MM-DD\", \"amount\":数字(元), \"type\":1或2(1支出2收入), \"category\":\"分类名\", \"remark\":\"备注\"}]}。date 无法判断时使用提供的今天日期；type 出现收入/工资/退款/报销等词为2，否则1；category 从：餐饮/交通/购物/居住/娱乐/医疗/教育/人情/通讯/其他 中选择最贴切的。禁止输出 JSON 以外的内容。";

const REPORT_SYSTEM: &str = "你是个人记账应用\"铭记日常\"的财务分析助手。基于提供的周期账单聚合数据生成周期报告。只输出合法 JSON：{\"title\":\"报告标题\",\"points\":[\"要点1\",\"要点2\",\"要点3\"(3~5条)],\"conclusion\":\"一句话结论\"}。每条要点必须引用具体数据（分类、金额、占比）。语气友好、有洞察。金额单位人民币元。禁止投资/医疗/借贷建议。";

const SAVINGS_SYSTEM: &str = "你是\"铭记日常\"的省钱顾问。基于用户最近周期的账单聚合数据，给出3~5条可执行的省钱建议。只输出合法 JSON：{\"suggestions\":[{\"title\":\"建议标题\",\"basis\":\"数据依据（必须引用具体分类/金额/占比）\",\"action\":\"具体怎么做\",\"expectation\":\"量化预期，如：预计每月节省约150元\"}]}。每条建议必须有数据依据与量化预期，无依据不得输出。禁止投资/医疗/借贷建议。";

const FOOD_SYSTEM: &str = "你是\"铭记日常\"的饮食建议助手。基于用户餐饮消费画像给出3条饮食建议（自制平替为主）。只输出合法 JSON：{\"suggestions\":[{\"item\":\"高频消费项，如：奶茶\",\"basis\":\"数据依据\",\"dish\":\"推荐的自制菜/饮品名\",\"recipe\":\"做法简述（3~5步）\",\"expectation\":\"量化预期\"}]}。禁止医疗/减肥疗效宣称。";

#[tauri::command]
pub async fn ai_parse_lines(lines: Vec<String>) -> Result<Vec<AiParsedLine>, String> {
    let cfg = load_config();
    if !cfg.enabled {
        return Err("AI 功能未开启（设置 → AI 智能服务）".into());
    }
    if lines.is_empty() {
        return Err("没有需要识别的行".into());
    }
    let numbered = lines
        .iter()
        .enumerate()
        .map(|(i, l)| format!("{}. {}", i, l))
        .collect::<Vec<_>>()
        .join("\n");
    let user = format!("今天日期：{}\n待解析行（每行格式：序号. 原文）：\n{}", today_str(), numbered);
    let content = call_deepseek(PARSE_SYSTEM, &user, 2048).await?;
    let v: Value = serde_json::from_str(&content).map_err(|e| format!("AI 返回解析失败：{}", e))?;
    let arr = v["items"].as_array().ok_or("AI 返回缺少 items 数组")?;
    let mut out = Vec::new();
    for it in arr {
        out.push(AiParsedLine {
            line: it["line"].as_i64().unwrap_or(out.len() as i64),
            date: it["date"].as_str().unwrap_or("").to_string(),
            amount: it["amount"].as_f64().unwrap_or(0.0),
            r#type: if it["type"].as_i64().unwrap_or(1) == 2 { 2 } else { 1 },
            category: it["category"].as_str().unwrap_or("其他").to_string(),
            remark: it["remark"].as_str().unwrap_or("").to_string(),
        });
    }
    Ok(out)
}

#[tauri::command]
pub async fn ai_generate(
    state: tauri::State<'_, crate::db::Db>,
    feature: String,
    cycles: Vec<CycleRange>,
) -> Result<String, String> {
    let cfg = load_config();
    if !cfg.enabled {
        return Err("AI 功能未开启（设置 → AI 智能服务）".into());
    }
    // 锁必须放在独立作用域内，块结束时立即释放，不能跨过下面的 .await（Send 约束）
    let aggregated = {
        let db: &crate::db::Db = &state;
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        aggregate_cycles(&conn, &cycles)?
    };
    let (system, max_tokens) = match feature.as_str() {
        "report" => (REPORT_SYSTEM, 2048u32),
        "savings" => (SAVINGS_SYSTEM, 2048),
        "food" => (FOOD_SYSTEM, 2048),
        _ => return Err("未知功能类型".into()),
    };
    let user = serde_json::to_string(&aggregated).map_err(|e| e.to_string())?;
    call_deepseek(system, &user, max_tokens).await
}
