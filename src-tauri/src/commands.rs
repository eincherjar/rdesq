use std::process::Command as ProcCommand;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use tauri::State;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    // Manager,
};
use tauri_plugin_autostart::ManagerExt;

use crate::db::Database;
use crate::models::*;
use crate::ping;

fn command_exists(name: &str) -> bool {
    let cmd = {
        #[cfg(target_os = "windows")]
        { "where" }
        #[cfg(not(target_os = "windows"))]
        { "which" }
    };
    ProcCommand::new(cmd).arg(name).output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// ─── CONNECTIONS ─────────────────────────────

#[tauri::command]
pub fn list_connections(db: State<Database>) -> Result<Vec<ConnEntry>, String> {
    db.list_connections().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_connection(db: State<Database>, id: String) -> Result<ConnEntry, String> {
    db.get_connection(&id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "not found".into())
}

#[tauri::command]
pub fn create_connection(db: State<Database>, data: ConnEntryInput) -> Result<ConnEntry, String> {
    db.create_connection(data).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_connection(db: State<Database>, id: String, data: ConnEntryInput) -> Result<ConnEntry, String> {
    db.update_connection(&id, data)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "not found".into())
}

#[tauri::command]
pub fn delete_connection(db: State<Database>, id: String) -> Result<(), String> {
    db.delete_connection(&id).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn duplicate_connection(db: State<Database>, id: String) -> Result<ConnEntry, String> {
    db.duplicate_connection(&id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "not found".into())
}

#[tauri::command]
pub fn set_favorite(db: State<Database>, id: String, favorite: bool) -> Result<ConnEntry, String> {
    db.set_favorite(&id, favorite)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "not found".into())
}

#[tauri::command]
pub async fn launch_connection(db: State<'_, Database>, id: String) -> Result<LaunchResult, String> {
    let conn = db.get_connection(&id)
        .map_err(|e| e.to_string())?
        .ok_or_else::<String, _>(|| "not found".into())?;

    match conn.protocol.as_str() {
        "ssh" => launch_ssh(&conn),
        "rdp" => launch_rdp(&conn),
        _ => Ok(LaunchResult { success: false, message: format!("unknown protocol: {}", conn.protocol) }),
    }
}

fn launch_ssh(conn: &ConnEntry) -> Result<LaunchResult, String> {
    let target = if conn.username.is_empty() {
        conn.host.clone()
    } else {
        format!("{}@{}", conn.username, conn.host)
    };

    let mut args: Vec<String> = vec![
        "-o".into(), "StrictHostKeyChecking=no".into(),
        "-o".into(), "ConnectTimeout=5".into(),
        target,
    ];

    if conn.port != 22 {
        args.push("-p".into());
        args.push(conn.port.to_string());
    }

    let pass = conn.password.as_deref().unwrap_or("").to_string();
    let key = conn.private_key_path.as_deref().unwrap_or("").to_string();

    match conn.auth_type.as_str() {
        "key" if !key.is_empty() => {
            args.insert(0, "-i".into());
            args.insert(1, key);
        }
        "password" if !pass.is_empty() => {
            #[cfg(target_os = "windows")]
            {
                // Windows ssh.exe handles password interactively in the terminal
            }
            #[cfg(not(target_os = "windows"))]
            {
                if command_exists("sshpass") {
                    let mut full = vec!["-p".to_string(), pass, "ssh".to_string()];
                    full.extend(args.clone());
                    return launch_in_system_terminal("sshpass", &full);
                }
                return Ok(LaunchResult {
                    success: false,
                    message: "sshpass not installed. Install it or use key-based auth.".into(),
                });
            }
        }
        _ => {}
    }

    launch_in_system_terminal("ssh", &args)
}

fn launch_in_system_terminal(program: &str, args: &[String]) -> Result<LaunchResult, String> {
    #[cfg(target_os = "linux")]
    {
        fn try_term(term: &str, program: &str, args: &[String]) -> Result<Option<LaunchResult>, String> {
            if !command_exists(term) { return Ok(None); }
            let mut cmd = ProcCommand::new(term);
            match term {
                "konsole" => { cmd.arg("--hold").arg("-e"); }
                t if t.contains("gnome-terminal") => { cmd.arg("--"); }
                _ => { cmd.arg("-e"); }
            }
            cmd.arg(program);
            for a in args { cmd.arg(a); }
            match cmd.spawn() {
                Ok(_) => Ok(Some(LaunchResult { success: true, message: "SSH launched".into() })),
                Err(e) => Ok(Some(LaunchResult { success: false, message: format!("Failed to launch terminal: {}", e) })),
            }
        }

        let cmd_out = |cmd: &str, args: &[&str]| -> Option<String> {
            ProcCommand::new(cmd).args(args).output().ok()
                .filter(|o| o.status.success())
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_string())
        };

        // 1. $TERMINAL env var
        if let Ok(term) = std::env::var("TERMINAL") {
            if !term.is_empty() {
                if let Some(r) = try_term(&term, program, args)? { return Ok(r); }
            }
        }

        // 2. KDE default terminal
        if let Some(term) = cmd_out("kreadconfig5", &["--file", "kdeglobals", "--group", "General", "--key", "TerminalApplication"]) {
            if let Some(r) = try_term(&term, program, args)? { return Ok(r); }
        }

        // 3. GNOME default terminal
        if let Some(term) = cmd_out("gsettings", &["get", "org.gnome.desktop.default-applications.terminal", "exec"]) {
            let term = term.trim_matches('\'');
            if !term.is_empty() {
                if let Some(r) = try_term(term, program, args)? { return Ok(r); }
            }
        }

        // 4. x-terminal-emulator (Debian alternatives)
        if command_exists("x-terminal-emulator") {
            let mut cmd = ProcCommand::new("x-terminal-emulator");
            cmd.arg("-e").arg(program);
            for a in args { cmd.arg(a); }
            if cmd.spawn().is_ok() {
                return Ok(LaunchResult { success: true, message: "SSH launched".into() });
            }
        }

        // 5. Fallback: known terminals
        for &term in &["konsole", "gnome-terminal", "xfce4-terminal", "xterm"] {
            if let Some(r) = try_term(term, program, args)? { return Ok(r); }
        }

        match ProcCommand::new(program).args(args).spawn() {
            Ok(_) => Ok(LaunchResult { success: true, message: "SSH launched (no terminal)".into() }),
            Err(e) => Ok(LaunchResult { success: false, message: format!("Failed to launch SSH: {}", e) }),
        }
    }

    #[cfg(target_os = "macos")]
    {
        let mut script = String::from("tell application \"Terminal\" to do script \"");
        script.push_str(program);
        for a in args {
            script.push(' ');
            script.push_str(a);
        }
        script.push('"');
        match ProcCommand::new("osascript").args(["-e", &script]).spawn() {
            Ok(_) => Ok(LaunchResult { success: true, message: "SSH launched".into() }),
            Err(e) => Ok(LaunchResult { success: false, message: format!("Failed to launch Terminal: {}", e) }),
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Build quoted cmdline for shells (preserves spaces in paths)
        let mut cmdline = String::new();
        cmdline.push_str(program);
        for a in args {
            cmdline.push(' ');
            if a.contains(' ') {
                cmdline.push('"');
                cmdline.push_str(a);
                cmdline.push('"');
            } else {
                cmdline.push_str(a);
            }
        }

        if command_exists("wt") {
            // Windows Terminal: wrap in cmd /k so the window stays open
            let mut cmd = ProcCommand::new("wt.exe");
            cmd.arg("cmd").arg("/k").arg(&cmdline);
            match cmd.spawn() {
                Ok(_) => return Ok(LaunchResult { success: true, message: "SSH launched".into() }),
                Err(_) => {}
            }
        }

        if command_exists("pwsh") {
            match ProcCommand::new("pwsh.exe").args(["-NoExit", "-Command", &cmdline]).spawn() {
                Ok(_) => return Ok(LaunchResult { success: true, message: "SSH launched".into() }),
                Err(_) => {}
            }
        }

        if command_exists("powershell") {
            match ProcCommand::new("powershell.exe").args(["-NoExit", "-Command", &cmdline]).spawn() {
                Ok(_) => return Ok(LaunchResult { success: true, message: "SSH launched".into() }),
                Err(_) => {}
            }
        }

        // Use CREATE_NO_WINDOW to suppress any console flash
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        match ProcCommand::new("cmd.exe")
            .creation_flags(CREATE_NO_WINDOW)
            .args(["/c", &format!("start \"SSH\" cmd /k \"{}\"", cmdline)])
            .spawn()
        {
            Ok(_) => Ok(LaunchResult { success: true, message: "SSH launched".into() }),
            Err(e) => Ok(LaunchResult { success: false, message: format!("Failed to launch SSH: {}", e) }),
        }
    }
}

#[cfg(target_os = "windows")]
fn launch_rdp(conn: &ConnEntry) -> Result<LaunchResult, String> {
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    if let Some(pass) = &conn.password {
        if !pass.is_empty() {
            let ep = format!("/generic:TERMSRV/{}", conn.host);
            let user = format!("/user:{}", conn.username);
            let pw = format!("/pass:{}", pass);
            let _ = ProcCommand::new("cmdkey")
                .creation_flags(CREATE_NO_WINDOW)
                .args([ep, user, pw])
                .output();
        }
    }

    let host_arg = format!("/v:{}", conn.host);
    match ProcCommand::new("mstsc").args([host_arg]).spawn() {
        Ok(_) => Ok(LaunchResult { success: true, message: "RDP launched".into() }),
        Err(e) => Ok(LaunchResult { success: false, message: format!("Failed to launch RDP: {}", e) }),
    }
}

#[cfg(not(target_os = "windows"))]
fn launch_rdp(conn: &ConnEntry) -> Result<LaunchResult, String> {
    let client = if command_exists("xfreerdp3") { "xfreerdp3" } else { "xfreerdp" };

    let mut args = vec![
        "/v", &conn.host,
        "/u", &conn.username,
        "/cert-ignore",
        "/dynamic-resolution",
        "+clipboard",
    ];

    let pass = conn.password.as_deref().unwrap_or("");
    if !pass.is_empty() {
        args.push("/p");
        args.push(pass);
    }

    match ProcCommand::new(client).args(&args).spawn() {
        Ok(_) => Ok(LaunchResult { success: true, message: "RDP launched".into() }),
        Err(e) => Ok(LaunchResult { success: false, message: format!("Failed to launch RDP client: {}. Install xfreerdp.", e) }),
    }
}

// ─── GROUPS ───────────────────────────────────

#[tauri::command]
pub fn list_groups(db: State<Database>) -> Result<Vec<Group>, String> {
    db.list_groups().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_group(db: State<Database>, data: GroupInput) -> Result<Group, String> {
    db.create_group(data).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_group(db: State<Database>, id: String, data: GroupInput) -> Result<(), String> {
    db.update_group(&id, data).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_group(db: State<Database>, id: String) -> Result<(), String> {
    db.delete_group(&id).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn update_group_order(db: State<Database>, orders: Vec<GroupOrder>) -> Result<(), String> {
    db.update_group_order(orders).map_err(|e| e.to_string())
}

// ─── SETTINGS ─────────────────────────────────

#[tauri::command]
pub fn get_settings(db: State<Database>) -> Settings {
    db.get_settings()
}

#[tauri::command]
pub fn save_settings(app: tauri::AppHandle, db: State<Database>, data: SettingsInput) -> Result<(), String> {
    if let Some(new_val) = data.start_with_system {
        let current = db.get_settings();
        if current.start_with_system != new_val {
            if new_val {
                app.autolaunch().enable().map_err(|e| e.to_string())?;
            } else {
                app.autolaunch().disable().map_err(|e| e.to_string())?;
            }
        }
    }
    if let Some(new_lang) = &data.lang {
        let current = db.get_settings();
        if &current.lang != new_lang {
            rebuild_tray_menu(&app, new_lang);
        }
    }
    db.save_settings(data);
    Ok(())
}

fn rebuild_tray_menu(app: &tauri::AppHandle, lang: &str) {
    let (show_text, quit_text) = if lang == "pl" { ("Pokaż", "Zakończ") } else { ("Show", "Quit") };
    let show = MenuItemBuilder::with_id("show", show_text).build(app);
    let quit = MenuItemBuilder::with_id("quit", quit_text).build(app);
    if let (Ok(show), Ok(quit)) = (show, quit) {
        if let Ok(menu) = MenuBuilder::new(app).item(&show).item(&quit).build() {
            if let Some(tray) = app.tray_by_id("main") {
                let _ = tray.set_menu(Some(menu));
            }
        }
    }
}

// ─── TAGS ─────────────────────────────────────

#[tauri::command]
pub fn list_tags(db: State<Database>) -> Result<Vec<TagInfo>, String> {
    db.list_tags().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn rename_tag(db: State<Database>, old_name: String, new_name: String) -> Result<(), String> {
    db.rename_tag(&old_name, &new_name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_tag(db: State<Database>, name: String) -> Result<(), String> {
    db.delete_tag(&name).map_err(|e| e.to_string())
}

// ─── PING ─────────────────────────────────────

#[tauri::command]
pub async fn ping_hosts(targets: Vec<PingTarget>) -> Vec<PingResult> {
    ping::ping_hosts(targets).await
}

// ─── EXPORT / IMPORT ──────────────────────────

#[tauri::command]
pub fn export_data(db: State<Database>) -> Result<ExportData, String> {
    db.export_data().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn import_data(db: State<Database>, data: ImportData) -> Result<ImportResult, String> {
    db.import_data(data).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_export_file(app: tauri::AppHandle, data: String) -> Result<String, String> {
    use tauri_plugin_dialog::DialogExt;

    let path = app.dialog()
        .file()
        .add_filter("JSON", &["json"])
        .set_file_name("rdesq-export.json")
        .blocking_save_file();

    match path {
        Some(p) => {
            let path_str = p.to_string();
            std::fs::write(&path_str, &data).map_err(|e| e.to_string())?;
            Ok("saved".to_string())
        }
        None => Err("cancelled".to_string()),
    }
}
