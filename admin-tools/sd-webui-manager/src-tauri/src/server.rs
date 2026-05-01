use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tauri::Emitter;
use tokio::process::{Child, Command};

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub enum ServerStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error(String),
}

pub struct LaunchInfo {
    pub program: String,
    pub args: Vec<String>,
    pub working_dir: PathBuf,
    pub env_vars: Vec<(String, String)>,
}

pub fn build_launch_command(sd_webui_dir: &Path, extra_args: Option<&str>) -> LaunchInfo {
    #[cfg(target_os = "windows")]
    {
        let script = sd_webui_dir.join("webui-user.bat");
        let mut args = vec!["/C".to_string()];
        if let Some(a) = extra_args {
            let mut bat_args = a.split_whitespace().map(String::from).collect();
            args.append(&mut bat_args);
        }
        args.push(script.to_string_lossy().to_string());
        LaunchInfo {
            program: "cmd".to_string(),
            args,
            working_dir: sd_webui_dir.to_path_buf(),
            env_vars: vec![],
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let script = sd_webui_dir.join("webui-user.sh");
        let mut args = vec![script.to_string_lossy().to_string()];
        if let Some(a) = extra_args {
            let mut extra = a.split_whitespace().map(String::from).collect();
            args.append(&mut extra);
        }
        let mut env_vars = vec![];
        if std::env::var("PYTORCH_ENABLE_MPS_FALLBACK").is_err() {
            env_vars.push(("PYTORCH_ENABLE_MPS_FALLBACK".to_string(), "1".to_string()));
        }
        LaunchInfo {
            program: "bash".to_string(),
            args,
            working_dir: sd_webui_dir.to_path_buf(),
            env_vars,
        }
    }
}

pub async fn spawn_server(launch: &LaunchInfo) -> std::io::Result<Child> {
    let mut cmd = Command::new(&launch.program);
    cmd.args(&launch.args)
        .current_dir(&launch.working_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    for (k, v) in &launch.env_vars {
        cmd.env(k, v);
    }

    #[cfg(unix)]
    #[allow(unused_imports)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }

    cmd.spawn()
}

pub async fn kill_process_tree(child: &mut Child) -> std::io::Result<()> {
    let pid = match child.id() {
        Some(p) => p,
        None => {
            let _ = child.kill().await;
            return Ok(());
        }
    };

    #[cfg(unix)]
    {
        unsafe {
            libc::kill(-(pid as i32), libc::SIGTERM);
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        let _ = child.try_wait();
        match child.try_wait() {
            Ok(Some(_)) => return Ok(()),
            Ok(None) => {
                unsafe {
                    libc::kill(-(pid as i32), libc::SIGKILL);
                }
            }
            Err(_) => return Ok(()),
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        let _ = child.kill().await;
    }

    #[cfg(not(unix))]
    {
        let _ = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .output()
            .await;
        let _ = child.kill().await;
    }

    Ok(())
}

pub fn read_stdout(child: &mut Child, app_handle: tauri::AppHandle) {
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    if let Some(stdout) = stdout {
        let h = app_handle.clone();
        tauri::async_runtime::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let _ = h.emit("server-log", &line);
            }
        });
    }

    if let Some(stderr) = stderr {
        let h = app_handle.clone();
        tauri::async_runtime::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let _ = h.emit("server-log", &line);
            }
        });
    }
}

fn is_sd_webui_dir(dir: &Path) -> bool {
    dir.join("launch.py").exists() && dir.join("webui.py").exists()
}

fn search_upward(start: &Path, max_depth: usize) -> Option<PathBuf> {
    let mut dir = start;
    for _ in 0..max_depth {
        if is_sd_webui_dir(dir) {
            return Some(dir.to_path_buf());
        }
        let sd_child = dir.join("stable-diffusion-webui");
        if sd_child.join("launch.py").exists() {
            return Some(sd_child);
        }
        dir = dir.parent()?;
    }
    None
}

fn search_children_for_sd_webui(root: &Path, max_depth: usize) -> Option<PathBuf> {
    if max_depth == 0 {
        return None;
    }
    if is_sd_webui_dir(root) {
        return Some(root.to_path_buf());
    }
    let sd_child = root.join("stable-diffusion-webui");
    if is_sd_webui_dir(&sd_child) {
        return Some(sd_child);
    }
    let Ok(entries) = std::fs::read_dir(root) else {
        return None;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        if file_name.starts_with('.') {
            continue;
        }
        if let Some(found) = search_children_for_sd_webui(&path, max_depth - 1) {
            return Some(found);
        }
    }
    None
}

pub fn detect_sd_webui_dir() -> Option<PathBuf> {
    if let Ok(cwd) = std::env::current_dir() {
        if let Some(p) = search_upward(&cwd, 8) {
            return Some(p);
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            if let Some(p) = search_upward(exe_dir, 8) {
                return Some(p);
            }
        }
    }

    let home = dirs::home_dir()?;
    let candidates = vec![
        home.join("AI").join("image-generative").join("stable-diffusion-webui"),
        home.join("stable-diffusion-webui"),
        PathBuf::from("/opt/stable-diffusion-webui"),
    ];
    for c in candidates {
        if is_sd_webui_dir(&c) {
            return Some(c);
        }
    }

    if let Some(p) = search_children_for_sd_webui(&home, 3) {
        return Some(p);
    }

    None
}
