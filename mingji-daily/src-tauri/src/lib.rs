mod db;
mod models;

use db::Db;
use models::*;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use base64::{engine::general_purpose::STANDARD as B64, Engine};
use rusqlite::backup::Backup;
use rusqlite::{params, params_from_iter, Connection, OptionalExtension, ToSql};
use tauri::{Manager, State};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let db = db::init()?;
            // 允许前端通过 asset 协议访问数据目录中的媒体文件（照片/视频）
            let data_dir = db::resolve_data_dir();
            app.asset_protocol_scope()
                .allow_directory(&data_dir, true)
                .map_err(|e| e.to_string())?;
            app.manage(db);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_bootstrap,
            get_data_dir,
            list_bills,
            create_bill,
            update_bill,
            delete_bill,
            create_category,
            update_category,
            delete_category,
            create_account,
            update_account,
            delete_account,
            create_cycle_rule,
            update_cycle_rule,
            delete_cycle_rule,
            attach_media,
            list_media,
            delete_media,
            get_media_usage,
            read_import_file,
            check_import_duplicate,
            confirm_import,
            list_import_tasks,
            revoke_import,
            create_backup,
            list_backups,
            restore_backup,
            delete_backup,
            export_bills_csv,
            export_all_json,
            get_orphan_media,
            clean_orphan_media,
            check_db_integrity,
            migrate_data_dir
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            // 退出前自动备份（保留最近 7 份）
            if let tauri::RunEvent::Exit = event {
                let _ = auto_backup(app);
            }
        });
}

// ---------- 基础数据 ----------

#[tauri::command]
fn get_bootstrap(state: State<Db>) -> Result<Bootstrap, String> {
    let conn = lock(&state)?;
    Ok(Bootstrap {
        categories: query_categories(&conn)?,
        accounts: query_accounts(&conn)?,
        cycle_rules: query_cycle_rules(&conn)?,
    })
}

#[tauri::command]
fn get_data_dir() -> String {
    db::resolve_data_dir().to_string_lossy().to_string()
}

fn query_categories(conn: &Connection) -> Result<Vec<Category>, String> {
    let mut stmt = conn
        .prepare("SELECT id, parent_id, name, sort, status, is_builtin FROM category ORDER BY sort, id")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok(Category {
                id: r.get(0)?,
                parent_id: r.get(1)?,
                name: r.get(2)?,
                sort: r.get(3)?,
                status: r.get(4)?,
                is_builtin: r.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

fn query_accounts(conn: &Connection) -> Result<Vec<Account>, String> {
    let mut stmt = conn
        .prepare("SELECT id, name, sort, status FROM account ORDER BY sort, id")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok(Account {
                id: r.get(0)?,
                name: r.get(1)?,
                sort: r.get(2)?,
                status: r.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

// ---------- 账单 ----------

const BILL_COLS: &str = "id, type, amount, category_id, account_id, bill_date, remark, source, status, created_at, updated_at";

fn row_to_bill(r: &rusqlite::Row) -> rusqlite::Result<Bill> {
    Ok(Bill {
        id: r.get(0)?,
        r#type: r.get(1)?,
        amount: r.get(2)?,
        category_id: r.get(3)?,
        account_id: r.get(4)?,
        bill_date: r.get(5)?,
        remark: r.get(6)?,
        source: r.get(7)?,
        status: r.get(8)?,
        created_at: r.get(9)?,
        updated_at: r.get(10)?,
    })
}

#[tauri::command]
fn list_bills(state: State<Db>, filter: Option<BillFilter>) -> Result<Vec<Bill>, String> {
    let conn = lock(&state)?;
    let mut conds: Vec<String> = vec!["status = 1".into()];
    let mut params: Vec<Box<dyn ToSql>> = vec![];

    if let Some(f) = &filter {
        if let Some(t) = f.r#type {
            conds.push("type = ?".into());
            params.push(Box::new(t));
        }
        if let Some(c) = f.category_id {
            // 选择一级分类时，同时匹配其全部子分类（统计穿透与筛选共用）
            conds.push(
                "category_id IN (SELECT id FROM category WHERE id = ? OR parent_id = ?)".into(),
            );
            params.push(Box::new(c));
            params.push(Box::new(c));
        }
        if let Some(a) = f.account_id {
            conds.push("account_id = ?".into());
            params.push(Box::new(a));
        }
        if let Some(s) = &f.start_date {
            conds.push("bill_date >= ?".into());
            params.push(Box::new(s.clone()));
        }
        if let Some(e) = &f.end_date {
            conds.push("bill_date <= ?".into());
            params.push(Box::new(e.clone()));
        }
        if let Some(k) = &f.keyword {
            conds.push("remark LIKE '%' || ? || '%'".into());
            params.push(Box::new(k.clone()));
        }
    }

    let sql = format!(
        "SELECT {} FROM bill WHERE {} ORDER BY bill_date DESC, id DESC",
        BILL_COLS,
        conds.join(" AND ")
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params_from_iter(params.iter().map(|p| p.as_ref())), row_to_bill)
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
fn create_bill(state: State<Db>, input: BillInput) -> Result<Bill, String> {
    validate_bill_input(&input)?;
    let conn = lock(&state)?;
    conn.execute(
        "INSERT INTO bill (type, amount, category_id, account_id, bill_date, remark, source)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1)",
        params![
            input.r#type,
            input.amount,
            input.category_id,
            input.account_id,
            input.bill_date,
            input.remark.unwrap_or_default()
        ],
    )
    .map_err(|e| e.to_string())?;
    query_bill(&conn, conn.last_insert_rowid())
}

#[tauri::command]
fn update_bill(state: State<Db>, id: i64, input: BillInput) -> Result<Bill, String> {
    validate_bill_input(&input)?;
    let conn = lock(&state)?;
    let affected = conn
        .execute(
            "UPDATE bill SET type = ?1, amount = ?2, category_id = ?3, account_id = ?4,
             bill_date = ?5, remark = ?6, updated_at = datetime('now','localtime')
             WHERE id = ?7 AND status = 1",
            params![
                input.r#type,
                input.amount,
                input.category_id,
                input.account_id,
                input.bill_date,
                input.remark.unwrap_or_default(),
                id
            ],
        )
        .map_err(|e| e.to_string())?;
    if affected == 0 {
        return Err("账单不存在或已删除".into());
    }
    query_bill(&conn, id)
}

#[tauri::command]
fn delete_bill(state: State<Db>, id: i64) -> Result<(), String> {
    let conn = lock(&state)?;
    let affected = conn
        .execute("UPDATE bill SET status = 0 WHERE id = ?1 AND status = 1", params![id])
        .map_err(|e| e.to_string())?;
    if affected == 0 {
        return Err("账单不存在或已删除".into());
    }
    // 联动删除媒体文件与记录
    remove_media_files(&conn, id)?;
    Ok(())
}

fn query_bill(conn: &Connection, id: i64) -> Result<Bill, String> {
    conn.query_row(
        &format!("SELECT {} FROM bill WHERE id = ?1", BILL_COLS),
        params![id],
        row_to_bill,
    )
    .map_err(|e| e.to_string())
}

fn validate_bill_input(input: &BillInput) -> Result<(), String> {
    if input.r#type != 1 && input.r#type != 2 {
        return Err("收支类型无效".into());
    }
    if input.amount <= 0 {
        return Err("金额必须大于 0".into());
    }
    if input.bill_date.is_empty() {
        return Err("日期不能为空".into());
    }
    if input.category_id <= 0 || input.account_id <= 0 {
        return Err("分类或账户无效".into());
    }
    Ok(())
}

// ---------- 分类 ----------

fn query_category(conn: &Connection, id: i64) -> Result<Category, String> {
    conn.query_row(
        "SELECT id, parent_id, name, sort, status, is_builtin FROM category WHERE id = ?1",
        params![id],
        |r| {
            Ok(Category {
                id: r.get(0)?,
                parent_id: r.get(1)?,
                name: r.get(2)?,
                sort: r.get(3)?,
                status: r.get(4)?,
                is_builtin: r.get(5)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn create_category(state: State<Db>, name: String, parent_id: Option<i64>) -> Result<Category, String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("分类名不能为空".into());
    }
    let conn = lock(&state)?;
    conn.execute(
        "INSERT INTO category (name, parent_id) VALUES (?1, ?2)",
        params![name, parent_id],
    )
    .map_err(|e| e.to_string())?;
    query_category(&conn, conn.last_insert_rowid())
}

#[tauri::command]
fn update_category(state: State<Db>, id: i64, name: String, status: i64) -> Result<Category, String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("分类名不能为空".into());
    }
    let conn = lock(&state)?;
    let affected = conn
        .execute(
            "UPDATE category SET name = ?1, status = ?2 WHERE id = ?3",
            params![name, status, id],
        )
        .map_err(|e| e.to_string())?;
    if affected == 0 {
        return Err("分类不存在".into());
    }
    query_category(&conn, id)
}

#[tauri::command]
fn delete_category(state: State<Db>, id: i64) -> Result<(), String> {
    let conn = lock(&state)?;
    let child_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM category WHERE parent_id = ?1", params![id], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if child_count > 0 {
        return Err("请先删除该分类下的子分类".into());
    }
    let used: i64 = conn
        .query_row("SELECT COUNT(*) FROM bill WHERE category_id = ?1 AND status = 1", params![id], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if used > 0 {
        return Err("该分类已被账单使用，只能停用".into());
    }
    conn.execute("DELETE FROM category WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ---------- 账户 ----------

fn query_account(conn: &Connection, id: i64) -> Result<Account, String> {
    conn.query_row(
        "SELECT id, name, sort, status FROM account WHERE id = ?1",
        params![id],
        |r| {
            Ok(Account {
                id: r.get(0)?,
                name: r.get(1)?,
                sort: r.get(2)?,
                status: r.get(3)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn create_account(state: State<Db>, name: String) -> Result<Account, String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("账户名不能为空".into());
    }
    let conn = lock(&state)?;
    conn.execute("INSERT INTO account (name) VALUES (?1)", params![name])
        .map_err(|e| e.to_string())?;
    query_account(&conn, conn.last_insert_rowid())
}

#[tauri::command]
fn update_account(state: State<Db>, id: i64, name: String, status: i64) -> Result<Account, String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("账户名不能为空".into());
    }
    let conn = lock(&state)?;
    let affected = conn
        .execute("UPDATE account SET name = ?1, status = ?2 WHERE id = ?3", params![name, status, id])
        .map_err(|e| e.to_string())?;
    if affected == 0 {
        return Err("账户不存在".into());
    }
    query_account(&conn, id)
}

#[tauri::command]
fn delete_account(state: State<Db>, id: i64) -> Result<(), String> {
    let conn = lock(&state)?;
    let used: i64 = conn
        .query_row("SELECT COUNT(*) FROM bill WHERE account_id = ?1 AND status = 1", params![id], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if used > 0 {
        return Err("该账户已被账单使用，只能停用".into());
    }
    conn.execute("DELETE FROM account WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ---------- 周期规则 ----------

fn query_cycle_rules(conn: &Connection) -> Result<Vec<CycleRule>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, type, start_date, length, length_unit, week_start, month_start_day, end_date, status
             FROM cycle_rule ORDER BY id",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok(CycleRule {
                id: r.get(0)?,
                name: r.get(1)?,
                r#type: r.get(2)?,
                start_date: r.get(3)?,
                length: r.get(4)?,
                length_unit: r.get(5)?,
                week_start: r.get(6)?,
                month_start_day: r.get(7)?,
                end_date: r.get(8)?,
                status: r.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

fn query_cycle_rule(conn: &Connection, id: i64) -> Result<CycleRule, String> {
    conn.query_row(
        "SELECT id, name, type, start_date, length, length_unit, week_start, month_start_day, end_date, status
         FROM cycle_rule WHERE id = ?1",
        params![id],
        |r| {
            Ok(CycleRule {
                id: r.get(0)?,
                name: r.get(1)?,
                r#type: r.get(2)?,
                start_date: r.get(3)?,
                length: r.get(4)?,
                length_unit: r.get(5)?,
                week_start: r.get(6)?,
                month_start_day: r.get(7)?,
                end_date: r.get(8)?,
                status: r.get(9)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

fn validate_cycle_input(input: &CycleRuleInput) -> Result<(), String> {
    if input.name.trim().is_empty() {
        return Err("规则名称不能为空".into());
    }
    if input.start_date.len() != 10 {
        return Err("起始日期无效".into());
    }
    match input.r#type.as_str() {
        "week" => {
            if !(1..=7).contains(&input.week_start) {
                return Err("周起始日无效".into());
            }
        }
        "month" => {
            if !(1..=28).contains(&input.month_start_day) {
                return Err("月起始日仅支持 1~28 日".into());
            }
        }
        "custom" => {
            if input.length < 1 {
                return Err("周期长度必须 ≥ 1".into());
            }
            if !["day", "week", "month"].contains(&input.length_unit.as_str()) {
                return Err("周期单位无效".into());
            }
        }
        _ => return Err("周期类型无效".into()),
    }
    if let Some(end) = &input.end_date {
        if !end.is_empty() && end < &input.start_date {
            return Err("结束日期不能早于起始日期".into());
        }
    }
    if input.status != 0 && input.status != 1 {
        return Err("状态无效".into());
    }
    Ok(())
}

fn check_enabled_limit(conn: &Connection, exclude_id: i64) -> Result<(), String> {
    let enabled: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM cycle_rule WHERE status = 1 AND id != ?1",
            params![exclude_id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if enabled >= 5 {
        return Err("最多同时启用 5 条周期规则".into());
    }
    Ok(())
}

#[tauri::command]
fn create_cycle_rule(state: State<Db>, input: CycleRuleInput) -> Result<CycleRule, String> {
    validate_cycle_input(&input)?;
    let conn = lock(&state)?;
    if input.status == 1 {
        check_enabled_limit(&conn, 0)?;
    }
    conn.execute(
        "INSERT INTO cycle_rule (name, type, start_date, length, length_unit, week_start, month_start_day, end_date, status)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            input.name.trim(),
            input.r#type,
            input.start_date,
            input.length,
            input.length_unit,
            input.week_start,
            input.month_start_day,
            input.end_date,
            input.status
        ],
    )
    .map_err(|e| e.to_string())?;
    query_cycle_rule(&conn, conn.last_insert_rowid())
}

#[tauri::command]
fn update_cycle_rule(state: State<Db>, id: i64, input: CycleRuleInput) -> Result<CycleRule, String> {
    validate_cycle_input(&input)?;
    let conn = lock(&state)?;
    if input.status == 1 {
        check_enabled_limit(&conn, id)?;
    }
    let affected = conn
        .execute(
            "UPDATE cycle_rule SET name = ?1, type = ?2, start_date = ?3, length = ?4, length_unit = ?5,
             week_start = ?6, month_start_day = ?7, end_date = ?8, status = ?9 WHERE id = ?10",
            params![
                input.name.trim(),
                input.r#type,
                input.start_date,
                input.length,
                input.length_unit,
                input.week_start,
                input.month_start_day,
                input.end_date,
                input.status,
                id
            ],
        )
        .map_err(|e| e.to_string())?;
    if affected == 0 {
        return Err("周期规则不存在".into());
    }
    query_cycle_rule(&conn, id)
}

#[tauri::command]
fn delete_cycle_rule(state: State<Db>, id: i64) -> Result<(), String> {
    let conn = lock(&state)?;
    conn.execute("DELETE FROM cycle_rule WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ---------- 媒体附件 ----------

fn media_dir_for(bill_id: i64) -> PathBuf {
    db::resolve_data_dir().join("media").join(bill_id.to_string())
}

/// 删除某账单的全部媒体：先删磁盘文件，再删数据库记录
fn remove_media_files(conn: &Connection, bill_id: i64) -> Result<(), String> {
    let mut stmt = conn
        .prepare("SELECT id, ext, type FROM media WHERE bill_id = ?1")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![bill_id], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, i64>(2)?))
        })
        .map_err(|e| e.to_string())?;
    let dir = media_dir_for(bill_id);
    for row in rows {
        let (id, ext, mtype) = row.map_err(|e| e.to_string())?;
        let _ = std::fs::remove_file(dir.join(format!("{}.{}", id, ext)));
        if mtype == 1 {
            let _ = std::fs::remove_file(dir.join(format!("{}_thumb.jpg", id)));
        }
    }
    conn.execute("DELETE FROM media WHERE bill_id = ?1", params![bill_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn make_thumbnail(src: &Path, dest: &Path) -> Result<PathBuf, String> {
    let img = image::open(src).map_err(|e| e.to_string())?;
    let thumb = img.thumbnail(640, 640);
    thumb
        .save_with_format(dest, image::ImageFormat::Jpeg)
        .map_err(|e| e.to_string())?;
    Ok(dest.to_path_buf())
}

fn build_media(bill_id: i64, id: i64, mtype: i64, file_name: String, ext: String, size: i64) -> Media {
    let dir = media_dir_for(bill_id);
    let path = dir.join(format!("{}.{}", id, ext));
    let thumb_path = if mtype == 1 {
        let t = dir.join(format!("{}_thumb.jpg", id));
        t.is_file().then(|| t.to_string_lossy().to_string())
    } else {
        None
    };
    Media {
        id,
        bill_id,
        r#type: mtype,
        file_name,
        ext,
        size,
        path: path.to_string_lossy().to_string(),
        thumb_path,
    }
}

#[tauri::command]
fn attach_media(state: State<Db>, bill_id: i64, path: String) -> Result<Media, String> {
    let src = PathBuf::from(&path);
    if !src.is_file() {
        return Err("所选文件不存在".into());
    }
    let ext = src
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let is_photo = matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp");
    let is_video = matches!(ext.as_str(), "mp4" | "webm" | "mov" | "m4v");
    if !is_photo && !is_video {
        return Err("仅支持图片（jpg/png/gif/webp/bmp）或视频（mp4/webm/mov/m4v）".into());
    }
    let mtype = if is_photo { 1 } else { 2 };

    let conn = lock(&state)?;
    let exists: i64 = conn
        .query_row("SELECT COUNT(*) FROM bill WHERE id = ?1 AND status = 1", params![bill_id], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if exists == 0 {
        return Err("账单不存在".into());
    }
    let photo_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM media WHERE bill_id = ?1 AND type = 1", params![bill_id], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    let video_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM media WHERE bill_id = ?1 AND type = 2", params![bill_id], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if is_photo && photo_count >= 9 {
        return Err("单笔账单最多 9 张照片".into());
    }
    if is_video && video_count >= 1 {
        return Err("单笔账单最多 1 个视频".into());
    }

    let file_name = src
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    let size = src.metadata().map(|m| m.len()).unwrap_or(0) as i64;

    conn.execute(
        "INSERT INTO media (bill_id, type, file_name, ext, size) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![bill_id, mtype, file_name, ext, size],
    )
    .map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();

    let dir = media_dir_for(bill_id);
    std::fs::create_dir_all(&dir).map_err(|e| format!("无法创建媒体目录：{}", e))?;
    let dest = dir.join(format!("{}.{}", id, ext));
    if let Err(e) = std::fs::copy(&src, &dest) {
        let _ = conn.execute("DELETE FROM media WHERE id = ?1", params![id]);
        return Err(format!("文件复制失败：{}", e));
    }

    if is_photo {
        // 缩略图失败不影响主流程（前端回退显示原图）
        let _ = make_thumbnail(&dest, &dir.join(format!("{}_thumb.jpg", id)));
    }

    Ok(build_media(bill_id, id, mtype, file_name, ext, size))
}

#[tauri::command]
fn list_media(state: State<Db>, bill_id: i64) -> Result<Vec<Media>, String> {
    let conn = lock(&state)?;
    let mut stmt = conn
        .prepare("SELECT id, type, file_name, ext, size FROM media WHERE bill_id = ?1 ORDER BY id")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![bill_id], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, i64>(4)?,
            ))
        })
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for row in rows {
        let (id, mtype, file_name, ext, size) = row.map_err(|e| e.to_string())?;
        out.push(build_media(bill_id, id, mtype, file_name, ext, size));
    }
    Ok(out)
}

#[tauri::command]
fn delete_media(state: State<Db>, id: i64) -> Result<(), String> {
    let conn = lock(&state)?;
    let row = conn
        .query_row(
            "SELECT bill_id, type, ext FROM media WHERE id = ?1",
            params![id],
            |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?, r.get::<_, String>(2)?)),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM media WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    if let Some((bill_id, mtype, ext)) = row {
        let dir = media_dir_for(bill_id);
        let _ = std::fs::remove_file(dir.join(format!("{}.{}", id, ext)));
        if mtype == 1 {
            let _ = std::fs::remove_file(dir.join(format!("{}_thumb.jpg", id)));
        }
    }
    Ok(())
}

#[tauri::command]
fn get_media_usage(state: State<Db>) -> Result<MediaUsage, String> {
    let conn = lock(&state)?;
    let (count, bytes): (i64, i64) = conn
        .query_row(
            "SELECT COUNT(*), COALESCE(SUM(size), 0) FROM media",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|e| e.to_string())?;
    Ok(MediaUsage { count, bytes })
}

// ---------- 账单导入 ----------

#[tauri::command]
fn read_import_file(path: String) -> Result<ImportFile, String> {
    let src = PathBuf::from(&path);
    if !src.is_file() {
        return Err("文件不存在".into());
    }
    let ext = src
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let name = src
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    let size = src.metadata().map(|m| m.len()).unwrap_or(0) as i64;
    let bytes = std::fs::read(&src).map_err(|e| format!("读取文件失败：{}", e))?;
    let md5_str = format!("{:x}", md5::compute(&bytes));

    match ext.as_str() {
        "txt" | "md" | "markdown" => {
            if size > 1_000_000 {
                return Err("txt/md 文件需 ≤ 1MB".into());
            }
            let data = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(&bytes);
            let text = match String::from_utf8(data.to_vec()) {
                Ok(t) => t,
                Err(_) => encoding_rs::GBK.decode(data).0.into_owned(),
            };
            Ok(ImportFile {
                name,
                ext,
                size,
                text: Some(text),
                base64: None,
                md5: md5_str,
            })
        }
        "docx" => {
            if size > 5_000_000 {
                return Err("docx 文件需 ≤ 5MB".into());
            }
            Ok(ImportFile {
                name,
                ext,
                size,
                text: None,
                base64: Some(B64.encode(&bytes)),
                md5: md5_str,
            })
        }
        "doc" => Err("旧版 .doc 暂不支持，请用 Word 另存为 .docx 后导入".into()),
        _ => Err("仅支持 txt / md / docx 文件".into()),
    }
}

#[tauri::command]
fn check_import_duplicate(state: State<Db>, file_md5: String) -> Result<Option<ImportTask>, String> {
    let conn = lock(&state)?;
    let row = conn
        .query_row(
            "SELECT id, file_name, file_md5, total, created_at FROM import_task
             WHERE file_md5 = ?1 ORDER BY id DESC LIMIT 1",
            params![file_md5],
            |r| {
                Ok(ImportTask {
                    id: r.get(0)?,
                    file_name: r.get(1)?,
                    file_md5: r.get(2)?,
                    total: r.get(3)?,
                    created_at: r.get(4)?,
                })
            },
        )
        .optional()
        .map_err(|e| e.to_string())?;
    Ok(row)
}

#[tauri::command]
fn confirm_import(state: State<Db>, input: ConfirmImportInput) -> Result<ImportTask, String> {
    if input.items.is_empty() {
        return Err("没有可导入的账单".into());
    }
    let mut conn = lock(&state)?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // 导入账单默认计入第一个可用账户（可在明细中修改）
    let account_id: i64 = tx
        .query_row("SELECT COALESCE(MIN(id), 1) FROM account WHERE status = 1", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;

    for it in &input.items {
        if it.r#type != 1 && it.r#type != 2 {
            return Err("存在无效的收支类型".into());
        }
        if it.amount <= 0 {
            return Err("存在金额 ≤ 0 的账单，请检查后再导入".into());
        }
        if it.bill_date.len() != 10 {
            return Err(format!("日期无效：{}", it.bill_date));
        }
        let cat_ok: i64 = tx
            .query_row(
                "SELECT COUNT(*) FROM category WHERE id = ?1 AND status = 1",
                params![it.category_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if cat_ok == 0 {
            return Err("存在无效分类，请检查后再导入".into());
        }
    }

    tx.execute(
        "INSERT INTO import_task (file_name, file_md5, total) VALUES (?1, ?2, ?3)",
        params![input.file_name, input.file_md5, input.items.len() as i64],
    )
    .map_err(|e| e.to_string())?;
    let task_id = tx.last_insert_rowid();

    for it in &input.items {
        tx.execute(
            "INSERT INTO bill (type, amount, category_id, account_id, bill_date, remark, source, import_task_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 2, ?7)",
            params![
                it.r#type,
                it.amount,
                it.category_id,
                account_id,
                it.bill_date,
                it.remark,
                task_id
            ],
        )
        .map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())?;

    let created_at: String = conn
        .query_row(
            "SELECT created_at FROM import_task WHERE id = ?1",
            params![task_id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(ImportTask {
        id: task_id,
        file_name: input.file_name,
        file_md5: input.file_md5,
        total: input.items.len() as i64,
        created_at,
    })
}

#[tauri::command]
fn list_import_tasks(state: State<Db>) -> Result<Vec<ImportTask>, String> {
    let conn = lock(&state)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, file_name, file_md5, total, created_at FROM import_task ORDER BY id DESC LIMIT 20",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok(ImportTask {
                id: r.get(0)?,
                file_name: r.get(1)?,
                file_md5: r.get(2)?,
                total: r.get(3)?,
                created_at: r.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
fn revoke_import(state: State<Db>, task_id: i64) -> Result<(), String> {
    let conn = lock(&state)?;
    let fresh: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM import_task
             WHERE id = ?1 AND created_at >= datetime('now','localtime','-1 day')",
            params![task_id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if fresh == 0 {
        return Err("仅可撤销 24 小时内的导入".into());
    }
    conn.execute(
        "UPDATE bill SET status = 0 WHERE import_task_id = ?1 AND source = 2 AND status = 1",
        params![task_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

// ---------- 数据管理（备份/恢复/导出/清理/迁移） ----------

fn backups_dir() -> PathBuf {
    db::resolve_data_dir().join("backups")
}

fn ts_stamp(conn: &Connection) -> Result<String, String> {
    let ts: String = conn
        .query_row("SELECT datetime('now','localtime')", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    Ok(ts.replace(' ', "_").replace(':', "-"))
}

fn stamp_to_display(stamp: &str) -> String {
    stamp.replace('_', " ")
}

fn walk_files(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for e in std::fs::read_dir(dir)? {
        let p = e?.path();
        if p.is_dir() {
            walk_files(&p, out)?;
        } else {
            out.push(p);
        }
    }
    Ok(())
}

fn dir_size(dir: &Path) -> i64 {
    let mut files = Vec::new();
    if walk_files(dir, &mut files).is_err() {
        return 0;
    }
    files
        .iter()
        .filter_map(|f| std::fs::metadata(f).ok())
        .map(|m| m.len())
        .sum::<u64>() as i64
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dst).map_err(|e| e.to_string())?;
    for e in std::fs::read_dir(src).map_err(|e| e.to_string())? {
        let e = e.map_err(|e| e.to_string())?;
        let from = e.path();
        let to = dst.join(e.file_name());
        if from.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            std::fs::copy(&from, &to).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn validate_backup_name(name: &str) -> Result<(), String> {
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err("备份名无效".into());
    }
    Ok(())
}

fn prune_auto_backups(bdir: &Path) {
    let mut list: Vec<PathBuf> = std::fs::read_dir(bdir)
        .map(|rd| rd.filter_map(|e| e.ok()).map(|e| e.path()).collect())
        .unwrap_or_default();
    list.retain(|p| {
        p.file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with("auto-"))
            .unwrap_or(false)
    });
    list.sort();
    while list.len() > 7 {
        let Some(f) = list.first() else { break };
        let _ = std::fs::remove_file(f);
        list.remove(0);
    }
}

fn auto_backup(app: &tauri::AppHandle) -> Result<(), String> {
    let state = app.state::<Db>();
    let db: &Db = &state;
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
        .map_err(|e| e.to_string())?;
    let stamp = ts_stamp(&conn)?;
    drop(conn);

    let bdir = backups_dir();
    std::fs::create_dir_all(&bdir).map_err(|e| e.to_string())?;
    let src = db::resolve_data_dir().join("app.db");
    if src.is_file() {
        std::fs::copy(&src, bdir.join(format!("auto-{}.db", stamp)))
            .map_err(|e| e.to_string())?;
    }
    prune_auto_backups(&bdir);
    Ok(())
}

#[tauri::command]
fn create_backup(state: State<Db>, with_media: bool) -> Result<BackupInfo, String> {
    let conn = lock(&state)?;
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
        .map_err(|e| e.to_string())?;
    let stamp = ts_stamp(&conn)?;
    drop(conn);

    let bdir = backups_dir();
    std::fs::create_dir_all(&bdir).map_err(|e| e.to_string())?;
    let src_db = db::resolve_data_dir().join("app.db");
    if !src_db.is_file() {
        return Err("数据库文件不存在".into());
    }

    let (name, size) = if with_media {
        let dir = bdir.join(format!("manual-{}", stamp));
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        std::fs::copy(&src_db, dir.join("app.db")).map_err(|e| e.to_string())?;
        let src_media = db::resolve_data_dir().join("media");
        if src_media.exists() {
            copy_dir_recursive(&src_media, &dir.join("media"))?;
        }
        let size = dir_size(&dir);
        (
            dir.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string(),
            size,
        )
    } else {
        let dst = bdir.join(format!("manual-{}.db", stamp));
        std::fs::copy(&src_db, &dst).map_err(|e| e.to_string())?;
        let size = std::fs::metadata(&dst).map(|m| m.len()).unwrap_or(0) as i64;
        (format!("manual-{}.db", stamp), size)
    };

    Ok(BackupInfo {
        name: name.clone(),
        path: bdir.join(&name).to_string_lossy().to_string(),
        size,
        with_media,
        created_at: stamp_to_display(&stamp),
    })
}

#[tauri::command]
fn list_backups() -> Result<Vec<BackupInfo>, String> {
    let bdir = backups_dir();
    std::fs::create_dir_all(&bdir).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for e in std::fs::read_dir(&bdir).map_err(|e| e.to_string())? {
        let e = e.map_err(|e| e.to_string())?;
        let name = e.file_name().to_string_lossy().to_string();
        if !(name.starts_with("auto-") || name.starts_with("manual-")) {
            continue;
        }
        let path = e.path();
        let is_dir = path.is_dir();
        let size = if is_dir {
            dir_size(&path)
        } else {
            std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0) as i64
        };
        let stamp = name
            .trim_start_matches("auto-")
            .trim_start_matches("manual-")
            .trim_end_matches(".db")
            .to_string();
        out.push(BackupInfo {
            name: name.clone(),
            path: path.to_string_lossy().to_string(),
            size,
            with_media: is_dir,
            created_at: stamp_to_display(&stamp),
        });
    }
    out.sort_by(|a, b| b.name.cmp(&a.name));
    Ok(out)
}

#[tauri::command]
fn restore_backup(state: State<Db>, name: String) -> Result<(), String> {
    validate_backup_name(&name)?;
    let src_path = backups_dir().join(&name);
    if !src_path.exists() {
        return Err("备份不存在".into());
    }
    let db_src = if src_path.is_dir() {
        src_path.join("app.db")
    } else {
        src_path.clone()
    };
    if !db_src.is_file() {
        return Err("备份数据不完整".into());
    }

    let mut conn = lock(&state)?;
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
        .map_err(|e| e.to_string())?;
    {
        let src_conn = Connection::open(&db_src).map_err(|e| e.to_string())?;
        let backup = Backup::new(&src_conn, &mut *conn).map_err(|e| e.to_string())?;
        backup
            .run_to_completion(256, std::time::Duration::from_millis(10), None)
            .map_err(|e| e.to_string())?;
    } // backup 在此结束对 conn 的借用
    drop(conn);

    if src_path.is_dir() {
        let src_media = src_path.join("media");
        if src_media.is_dir() {
            let dst_media = db::resolve_data_dir().join("media");
            if dst_media.exists() {
                std::fs::remove_dir_all(&dst_media).map_err(|e| e.to_string())?;
            }
            copy_dir_recursive(&src_media, &dst_media)?;
        }
    }
    Ok(())
}

#[tauri::command]
fn delete_backup(name: String) -> Result<(), String> {
    validate_backup_name(&name)?;
    let p = backups_dir().join(&name);
    if !p.exists() {
        return Err("备份不存在".into());
    }
    if p.is_dir() {
        std::fs::remove_dir_all(&p).map_err(|e| e.to_string())?;
    } else {
        std::fs::remove_file(&p).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[tauri::command]
fn export_bills_csv(state: State<Db>, path: String) -> Result<ExportResult, String> {
    let conn = lock(&state)?;
    let mut stmt = conn
        .prepare(
            "SELECT b.bill_date, b.type, b.amount, c.name, p.name, a.name, b.remark, b.source
             FROM bill b
             LEFT JOIN category c ON c.id = b.category_id
             LEFT JOIN category p ON p.id = c.parent_id
             LEFT JOIN account a ON a.id = b.account_id
             WHERE b.status = 1
             ORDER BY b.bill_date, b.id",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, Option<String>>(4)?,
                r.get::<_, Option<String>>(5)?,
                r.get::<_, String>(6)?,
                r.get::<_, i64>(7)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut csv = String::from("\u{FEFF}日期,类型,金额(元),一级分类,二级分类,账户,备注,来源\n");
    let mut count = 0i64;
    for row in rows {
        let (date, t, amount, cat, pcat, account, remark, source) =
            row.map_err(|e| e.to_string())?;
        csv.push_str(&format!(
            "{},{},{:.2},{},{},{},{},{}\n",
            csv_escape(&date),
            if t == 2 { "收入" } else { "支出" },
            amount as f64 / 100.0,
            csv_escape(pcat.as_deref().unwrap_or("")),
            csv_escape(cat.as_deref().unwrap_or("")),
            csv_escape(account.as_deref().unwrap_or("")),
            csv_escape(&remark),
            if source == 2 { "导入" } else { "手工" }
        ));
        count += 1;
    }
    std::fs::write(&path, csv).map_err(|e| e.to_string())?;
    Ok(ExportResult { count, path })
}

#[tauri::command]
fn export_all_json(state: State<Db>, path: String) -> Result<ExportResult, String> {
    let conn = lock(&state)?;
    let categories = query_categories(&conn)?;
    let accounts = query_accounts(&conn)?;
    let cycle_rules = query_cycle_rules(&conn)?;

    let bills = {
        let mut stmt = conn
            .prepare(&format!(
                "SELECT {} FROM bill WHERE status = 1 ORDER BY bill_date, id",
                BILL_COLS
            ))
            .map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], row_to_bill).map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?
    };

    let media: Vec<serde_json::Value> = {
        let mut stmt = conn
            .prepare("SELECT id, bill_id, type, file_name, ext, size FROM media ORDER BY id")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, i64>(5)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        let mut v = Vec::new();
        for row in rows {
            let (id, bill_id, mtype, file_name, ext, size) = row.map_err(|e| e.to_string())?;
            v.push(serde_json::json!({
                "id": id,
                "bill_id": bill_id,
                "type": mtype,
                "file_name": file_name,
                "ext": ext,
                "size": size,
                "rel_path": format!("media/{}/{}.{}", bill_id, id, ext),
            }));
        }
        v
    };

    let bill_count = bills.len() as i64;
    let json = serde_json::json!({
        "app": "铭记日常",
        "schema_version": 1,
        "exported_at": ts_stamp(&conn)?,
        "categories": categories,
        "accounts": accounts,
        "cycle_rules": cycle_rules,
        "bills": bills,
        "media": media,
    });
    drop(conn);
    std::fs::write(&path, serde_json::to_string_pretty(&json).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    Ok(ExportResult {
        count: bill_count,
        path,
    })
}

fn orphan_scan(delete: bool) -> Result<OrphanInfo, String> {
    // 用独立只读连接收集应存在的文件名集合
    let media_root = db::resolve_data_dir().join("media");
    let mut valid: HashSet<PathBuf> = HashSet::new();
    {
        let db_path = db::resolve_data_dir().join("app.db");
        let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, bill_id, type, ext FROM media")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, String>(3)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            let (id, bill_id, mtype, ext) = row.map_err(|e| e.to_string())?;
            let dir = media_root.join(bill_id.to_string());
            valid.insert(dir.join(format!("{}.{}", id, ext)));
            if mtype == 1 {
                valid.insert(dir.join(format!("{}_thumb.jpg", id)));
            }
        }
    }

    let mut files = Vec::new();
    walk_files(&media_root, &mut files).map_err(|e| e.to_string())?;
    let mut count = 0i64;
    let mut bytes = 0i64;
    for f in files {
        if valid.contains(&f) {
            continue;
        }
        count += 1;
        if let Ok(m) = std::fs::metadata(&f) {
            bytes += m.len() as i64;
        }
        if delete {
            let _ = std::fs::remove_file(&f);
        }
    }
    Ok(OrphanInfo { count, bytes })
}

#[tauri::command]
fn get_orphan_media() -> Result<OrphanInfo, String> {
    orphan_scan(false)
}

#[tauri::command]
fn clean_orphan_media() -> Result<OrphanInfo, String> {
    orphan_scan(true)
}

#[tauri::command]
fn check_db_integrity(state: State<Db>) -> Result<String, String> {
    let conn = lock(&state)?;
    let s: String = conn
        .query_row("PRAGMA quick_check", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    Ok(s)
}

#[tauri::command]
fn migrate_data_dir(state: State<Db>, target: String) -> Result<String, String> {
    let dst = PathBuf::from(&target);
    if !dst.is_dir() {
        return Err("目标目录不存在".into());
    }
    let src = db::resolve_data_dir();
    if dst == src {
        return Err("目标目录与当前数据目录相同".into());
    }
    let conn = lock(&state)?;
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
        .map_err(|e| e.to_string())?;
    drop(conn);

    let src_db = src.join("app.db");
    if src_db.is_file() {
        std::fs::copy(&src_db, dst.join("app.db")).map_err(|e| e.to_string())?;
    }
    let src_media = src.join("media");
    if src_media.exists() {
        copy_dir_recursive(&src_media, &dst.join("media"))?;
    }
    Ok(dst.to_string_lossy().to_string())
}

fn lock<'a>(state: &'a State<'_, Db>) -> Result<std::sync::MutexGuard<'a, Connection>, String> {
    let db: &'a Db = state; // 通过 Deref 拿到 Db 引用，锁的借用生命周期与引用一致
    db.0.lock().map_err(|e| e.to_string())
}
