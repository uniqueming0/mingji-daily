use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::Connection;

pub struct Db(pub Mutex<Connection>);

/// 数据目录规则（对应 PRD FR-DIS-03 / FR-DIS-13 / FR-DIS-05）：
/// - 便携版：程序同目录 `data\`（整个文件夹拷贝即可搬家）
/// - 安装版 / 运行于临时目录（未解压直接双击）：`%APPDATA%\铭记日常\`
pub fn resolve_data_dir() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    let exe_str = exe_dir.to_string_lossy().to_lowercase();
    let temp_str = std::env::temp_dir().to_string_lossy().to_lowercase();

    let in_program_dirs = exe_str.contains("\\program files") || exe_str.contains("\\programs\\");
    let in_temp = exe_str.starts_with(&temp_str);

    if in_program_dirs || in_temp {
        let appdata = std::env::var("APPDATA")
            .unwrap_or_else(|_| exe_dir.to_string_lossy().to_string());
        PathBuf::from(appdata).join("铭记日常")
    } else {
        exe_dir.join("data")
    }
}

pub fn init() -> Result<Db, String> {
    let data_dir = resolve_data_dir();
    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("无法创建数据目录 {}：{}", data_dir.display(), e))?;
    let db_path = data_dir.join("app.db");
    let conn = Connection::open(&db_path)
        .map_err(|e| format!("无法打开数据库 {}：{}", db_path.display(), e))?;
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA foreign_keys=ON;",
    )
    .map_err(|e| e.to_string())?;
    create_schema(&conn)?;
    seed(&conn)?;
    Ok(Db(Mutex::new(conn)))
}

fn create_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS category (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            parent_id INTEGER,
            name TEXT NOT NULL,
            sort INTEGER NOT NULL DEFAULT 0,
            status INTEGER NOT NULL DEFAULT 1,
            is_builtin INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
        );
        CREATE TABLE IF NOT EXISTS account (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            sort INTEGER NOT NULL DEFAULT 0,
            status INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
        );
        CREATE TABLE IF NOT EXISTS cycle_rule (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            type TEXT NOT NULL DEFAULT 'month',
            start_date TEXT NOT NULL,
            length INTEGER NOT NULL DEFAULT 1,
            length_unit TEXT NOT NULL DEFAULT 'month',
            week_start INTEGER NOT NULL DEFAULT 1,
            month_start_day INTEGER NOT NULL DEFAULT 1,
            end_date TEXT,
            status INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
        );
        CREATE TABLE IF NOT EXISTS bill (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            type INTEGER NOT NULL DEFAULT 1,
            amount INTEGER NOT NULL,
            category_id INTEGER NOT NULL,
            account_id INTEGER NOT NULL,
            bill_date TEXT NOT NULL,
            remark TEXT NOT NULL DEFAULT '',
            source INTEGER NOT NULL DEFAULT 1,
            status INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now','localtime')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
        );
        CREATE INDEX IF NOT EXISTS idx_bill_date ON bill(bill_date);
        CREATE INDEX IF NOT EXISTS idx_bill_category ON bill(category_id);
        CREATE INDEX IF NOT EXISTS idx_bill_account ON bill(account_id);
        CREATE TABLE IF NOT EXISTS media (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            bill_id INTEGER NOT NULL,
            type INTEGER NOT NULL DEFAULT 1,
            file_name TEXT NOT NULL DEFAULT '',
            ext TEXT NOT NULL,
            size INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
        );
        CREATE INDEX IF NOT EXISTS idx_media_bill ON media(bill_id);
        CREATE TABLE IF NOT EXISTS import_task (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_name TEXT NOT NULL,
            file_md5 TEXT NOT NULL,
            total INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
        );
        CREATE INDEX IF NOT EXISTS idx_import_md5 ON import_task(file_md5);",
    )
    .map_err(|e| e.to_string())?;

    // 旧库迁移：给 bill 表补充 import_task_id 列（用于导入撤销）
    let has_task_col: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('bill') WHERE name = 'import_task_id'",
            [],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if has_task_col == 0 {
        conn.execute("ALTER TABLE bill ADD COLUMN import_task_id INTEGER", [])
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 内置数据（对应 PRD 附录 B 默认分类表）
fn seed(conn: &Connection) -> Result<(), String> {
    let cat_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM category", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if cat_count == 0 {
        let cats: &[(&str, Option<&str>)] = &[
            ("餐饮", None),
            ("早餐", Some("餐饮")), ("午餐", Some("餐饮")), ("晚餐", Some("餐饮")),
            ("外卖", Some("餐饮")), ("聚餐", Some("餐饮")), ("咖啡奶茶", Some("餐饮")), ("零食", Some("餐饮")),
            ("交通", None),
            ("公交地铁", Some("交通")), ("打车", Some("交通")), ("加油", Some("交通")),
            ("停车", Some("交通")), ("火车机票", Some("交通")),
            ("购物", None),
            ("日用百货", Some("购物")), ("服饰", Some("购物")), ("美妆", Some("购物")),
            ("数码", Some("购物")), ("宠物", Some("购物")),
            ("居住", None),
            ("房租", Some("居住")), ("水电燃气", Some("居住")), ("物业", Some("居住")), ("家居", Some("居住")),
            ("娱乐", None),
            ("电影演出", Some("娱乐")), ("游戏", Some("娱乐")), ("旅行", Some("娱乐")), ("运动健身", Some("娱乐")),
            ("医疗", None),
            ("门诊", Some("医疗")), ("药品", Some("医疗")), ("体检", Some("医疗")),
            ("教育", None),
            ("课程", Some("教育")), ("书籍", Some("教育")), ("考试", Some("教育")),
            ("人情", None),
            ("红包", Some("人情")), ("礼物", Some("人情")), ("请客", Some("人情")),
            ("通讯", None),
            ("话费", Some("通讯")), ("网费", Some("通讯")),
            ("收入类", None),
            ("工资", Some("收入类")), ("奖金", Some("收入类")), ("兼职", Some("收入类")),
            ("理财", Some("收入类")), ("退款", Some("收入类")), ("其他收入", Some("收入类")),
            ("其他", None),
        ];
        let mut ids: HashMap<&str, i64> = HashMap::new();
        for (name, parent) in cats {
            let parent_id = parent.and_then(|p| ids.get(p).copied());
            conn.execute(
                "INSERT INTO category (name, parent_id, is_builtin) VALUES (?1, ?2, 1)",
                rusqlite::params![name, parent_id],
            )
            .map_err(|e| e.to_string())?;
            ids.insert(name, conn.last_insert_rowid());
        }
    }

    let acc_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM account", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if acc_count == 0 {
        for name in ["现金", "微信", "支付宝", "银行卡"] {
            conn.execute(
                "INSERT INTO account (name) VALUES (?1)",
                rusqlite::params![name],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    let rule_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM cycle_rule", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if rule_count == 0 {
        conn.execute(
            "INSERT INTO cycle_rule (name, type, start_date) VALUES ('默认周期', 'month', date('now','localtime'))",
            [],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}
