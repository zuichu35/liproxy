#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use chrono::{Datelike, Local, NaiveDate};
use regex::Regex;
use rusqlite::{params, Connection};
use serde::Serialize;
use std::{collections::HashMap, sync::Mutex};

struct AppState {
    conn: Mutex<Connection>,
}

#[derive(Serialize)]
struct Task {
    id: i64,
    title: String,
    task_type: String,
    status: String,
    priority: i64,
    due_at: Option<String>,
    est_minutes: i64,
    spent_minutes: i64,
    notes: String,
    raw_input: String,
}

#[derive(Serialize)]
struct Course {
    id: i64,
    name: String,
    weekday: i64,
    start_time: String,
    end_time: String,
    weeks: String,
    location: String,
    teacher: String,
}

#[derive(Serialize)]
struct Habit {
    id: i64,
    name: String,
    weekly_target: i64,
}

#[tauri::command]
fn init_db(state: tauri::State<AppState>) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS tasks (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          title TEXT NOT NULL,
          task_type TEXT NOT NULL DEFAULT '杂项',
          status TEXT NOT NULL DEFAULT 'Inbox',
          priority INTEGER NOT NULL DEFAULT 3,
          due_at TEXT,
          est_minutes INTEGER NOT NULL DEFAULT 60,
          spent_minutes INTEGER NOT NULL DEFAULT 0,
          notes TEXT NOT NULL DEFAULT '',
          raw_input TEXT NOT NULL DEFAULT '',
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS task_blocks (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          task_id INTEGER NOT NULL,
          title TEXT NOT NULL,
          minutes INTEGER NOT NULL,
          start_at TEXT,
          end_at TEXT,
          status TEXT NOT NULL DEFAULT 'Planned',
          created_at TEXT NOT NULL,
          FOREIGN KEY(task_id) REFERENCES tasks(id)
        );
        CREATE TABLE IF NOT EXISTS courses (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          name TEXT NOT NULL,
          weekday INTEGER NOT NULL,
          start_time TEXT NOT NULL,
          end_time TEXT NOT NULL,
          weeks TEXT NOT NULL,
          location TEXT NOT NULL,
          teacher TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS focus_sessions (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          task_id INTEGER NOT NULL,
          minutes INTEGER NOT NULL,
          interrupted INTEGER NOT NULL,
          started_at TEXT NOT NULL,
          ended_at TEXT NOT NULL,
          FOREIGN KEY(task_id) REFERENCES tasks(id)
        );
        CREATE TABLE IF NOT EXISTS habits (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          name TEXT NOT NULL,
          weekly_target INTEGER NOT NULL DEFAULT 3
        );
        CREATE TABLE IF NOT EXISTS habit_logs (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          habit_id INTEGER NOT NULL,
          logged_at TEXT NOT NULL,
          FOREIGN KEY(habit_id) REFERENCES habits(id)
        );
        CREATE TABLE IF NOT EXISTS settings (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          key TEXT NOT NULL UNIQUE,
          value TEXT NOT NULL
        );
    "#,
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn now_str() -> String {
    Local::now().naive_local().to_string()
}

#[derive(Default)]
struct ParsedInput {
    title: String,
    due_at: Option<String>,
    est_minutes: i64,
    task_type: String,
}

fn parse_input(input: &str) -> ParsedInput {
    let mut parsed = ParsedInput {
        title: input.to_string(),
        due_at: None,
        est_minutes: 60,
        task_type: "杂项".to_string(),
    };

    if input.contains("作业") {
        parsed.task_type = "作业".to_string();
    } else if input.contains("英语") {
        parsed.task_type = "英语".to_string();
    } else if input.contains("论文") {
        parsed.task_type = "论文".to_string();
    } else if input.contains("游泳") || input.contains("健身") || input.contains("攀岩") {
        parsed.task_type = "训练".to_string();
    }

    let re_hour = Regex::new(r"(\d+)\s*小时").expect("regex");
    if let Some(c) = re_hour.captures(input) {
        parsed.est_minutes = c[1].parse::<i64>().unwrap_or(1) * 60;
    }
    let re_min = Regex::new(r"(\d+)\s*分钟").expect("regex");
    if let Some(c) = re_min.captures(input) {
        parsed.est_minutes = c[1].parse::<i64>().unwrap_or(60);
    }

    let re_date = Regex::new(r"(\d{1,2})月(\d{1,2})日").expect("regex");
    if let Some(c) = re_date.captures(input) {
        let year = Local::now().year();
        let m = c[1].parse::<u32>().unwrap_or(1);
        let d = c[2].parse::<u32>().unwrap_or(1);
        if let Some(date) = NaiveDate::from_ymd_opt(year, m, d) {
            parsed.due_at = Some(format!("{} 23:59:00", date));
        }
    }

    if input.contains("明天") {
        let tomorrow = Local::now().date_naive().succ_opt().unwrap_or(Local::now().date_naive());
        parsed.due_at = Some(format!("{} 12:00:00", tomorrow));
    }

    parsed
}

#[tauri::command]
fn create_task_from_input(state: tauri::State<AppState>, input: String) -> Result<i64, String> {
    let parsed = parse_input(&input);
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO tasks (title, task_type, status, priority, due_at, est_minutes, spent_minutes, notes, raw_input, created_at, updated_at)
         VALUES (?1, ?2, 'Inbox', 3, ?3, ?4, 0, '', ?5, ?6, ?6)",
        params![parsed.title, parsed.task_type, parsed.due_at, parsed.est_minutes, input, now_str()],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
fn list_tasks(state: tauri::State<AppState>) -> Result<Vec<Task>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id,title,task_type,status,priority,due_at,est_minutes,spent_minutes,notes,raw_input FROM tasks ORDER BY updated_at DESC")
        .map_err(|e| e.to_string())?;
    let iter = stmt
        .query_map([], |r| {
            Ok(Task {
                id: r.get(0)?,
                title: r.get(1)?,
                task_type: r.get(2)?,
                status: r.get(3)?,
                priority: r.get(4)?,
                due_at: r.get(5)?,
                est_minutes: r.get(6)?,
                spent_minutes: r.get(7)?,
                notes: r.get(8)?,
                raw_input: r.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?;
    Ok(iter.filter_map(Result::ok).collect())
}

#[tauri::command]
fn update_task_status(state: tauri::State<AppState>, id: i64, status: String) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE tasks SET status = ?1, updated_at = ?2 WHERE id = ?3",
        params![status, now_str(), id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn split_task_blocks(state: tauri::State<AppState>, task_id: i64) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let est: i64 = conn
        .query_row("SELECT est_minutes FROM tasks WHERE id=?1", params![task_id], |r| r.get(0))
        .map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM task_blocks WHERE task_id=?1", params![task_id])
        .map_err(|e| e.to_string())?;

    let blocks: Vec<i64> = if est <= 60 {
        vec![est]
    } else if est <= 180 {
        vec![est / 2, est - est / 2]
    } else {
        let mut left = est;
        let mut v = vec![];
        while left > 0 {
            let chunk = if left >= 120 { 120 } else if left >= 90 { 90 } else { left };
            v.push(chunk);
            left -= chunk;
        }
        v
    };

    for (idx, m) in blocks.iter().enumerate() {
        conn.execute(
            "INSERT INTO task_blocks (task_id,title,minutes,start_at,end_at,status,created_at) VALUES (?1,?2,?3,NULL,NULL,'Planned',?4)",
            params![task_id, format!("Block {}", idx + 1), m, now_str()],
        )
        .map_err(|e| e.to_string())?;
    }

    conn.execute(
        "UPDATE tasks SET status='Planned', updated_at=?1 WHERE id=?2",
        params![now_str(), task_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn create_course(
    state: tauri::State<AppState>,
    name: String,
    weekday: i64,
    start_time: String,
    end_time: String,
    weeks: String,
    location: String,
    teacher: String,
) -> Result<i64, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO courses (name,weekday,start_time,end_time,weeks,location,teacher) VALUES (?1,?2,?3,?4,?5,?6,?7)",
        params![name, weekday, start_time, end_time, weeks, location, teacher],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
fn list_courses(state: tauri::State<AppState>) -> Result<Vec<Course>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id,name,weekday,start_time,end_time,weeks,location,teacher FROM courses ORDER BY weekday,start_time")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok(Course {
                id: r.get(0)?,
                name: r.get(1)?,
                weekday: r.get(2)?,
                start_time: r.get(3)?,
                end_time: r.get(4)?,
                weeks: r.get(5)?,
                location: r.get(6)?,
                teacher: r.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?;
    Ok(rows.filter_map(Result::ok).collect())
}

#[tauri::command]
fn create_habit(state: tauri::State<AppState>, name: String, weekly_target: i64) -> Result<i64, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO habits (name,weekly_target) VALUES (?1,?2)",
        params![name, weekly_target],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
fn list_habits(state: tauri::State<AppState>) -> Result<Vec<Habit>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id,name,weekly_target FROM habits ORDER BY id DESC").map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok(Habit {
                id: r.get(0)?,
                name: r.get(1)?,
                weekly_target: r.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?;
    Ok(rows.filter_map(Result::ok).collect())
}

#[tauri::command]
fn log_habit(state: tauri::State<AppState>, habit_id: i64) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO habit_logs (habit_id, logged_at) VALUES (?1, ?2)",
        params![habit_id, now_str()],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn record_focus_session(
    state: tauri::State<AppState>,
    task_id: i64,
    minutes: i64,
    interrupted: bool,
) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let start = now_str();
    let end = now_str();
    conn.execute(
        "INSERT INTO focus_sessions (task_id,minutes,interrupted,started_at,ended_at) VALUES (?1,?2,?3,?4,?5)",
        params![task_id, minutes, if interrupted { 1 } else { 0 }, start, end],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE tasks SET spent_minutes = spent_minutes + ?1, status='Doing', updated_at=?2 WHERE id=?3",
        params![minutes, now_str(), task_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn weekly_review(state: tauri::State<AppState>) -> Result<HashMap<String, serde_json::Value>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let today = Local::now().date_naive();
    let week_start = today
        .checked_sub_days(chrono::Days::new(today.weekday().num_days_from_monday() as u64))
        .unwrap_or(today);
    let week_start_dt = format!("{} 00:00:00", week_start);

    let done_tasks: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM tasks WHERE status='Done' AND updated_at >= ?1",
            params![week_start_dt],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let week_focus_minutes: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(minutes),0) FROM focus_sessions WHERE started_at >= ?1",
            params![week_start_dt],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let today_start = format!("{} 00:00:00", today);
    let today_focus_minutes: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(minutes),0) FROM focus_sessions WHERE started_at >= ?1",
            params![today_start],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let english_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM habit_logs hl JOIN habits h ON hl.habit_id=h.id WHERE hl.logged_at >= ?1 AND h.name LIKE '%英语%'",
            params![week_start_dt],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let training_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM habit_logs hl JOIN habits h ON hl.habit_id=h.id WHERE hl.logged_at >= ?1 AND (h.name LIKE '%训练%' OR h.name LIKE '%健身%' OR h.name LIKE '%游泳%' OR h.name LIKE '%攀岩%')",
            params![week_start_dt],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let paper_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM habit_logs hl JOIN habits h ON hl.habit_id=h.id WHERE hl.logged_at >= ?1 AND h.name LIKE '%论文%'",
            params![week_start_dt],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let summary = if training_count < 2 {
        "本周训练不足".to_string()
    } else if english_count == 0 {
        "英语学习断档".to_string()
    } else if paper_count == 0 {
        "论文阅读未达标".to_string()
    } else {
        "执行良好，下周保持节奏".to_string()
    };

    let mut m = HashMap::new();
    m.insert("done_tasks".into(), done_tasks.into());
    m.insert("week_focus_minutes".into(), week_focus_minutes.into());
    m.insert("today_focus_minutes".into(), today_focus_minutes.into());
    m.insert("english_count".into(), english_count.into());
    m.insert("training_count".into(), training_count.into());
    m.insert("paper_count".into(), paper_count.into());
    m.insert("summary".into(), summary.into());
    Ok(m)
}

fn main() {
    let db = Connection::open("todo_timeblocks.db").expect("failed to open sqlite");

    tauri::Builder::default()
        .manage(AppState { conn: Mutex::new(db) })
        .invoke_handler(tauri::generate_handler![
            init_db,
            create_task_from_input,
            list_tasks,
            update_task_status,
            split_task_blocks,
            create_course,
            list_courses,
            create_habit,
            list_habits,
            log_habit,
            record_focus_session,
            weekly_review,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
