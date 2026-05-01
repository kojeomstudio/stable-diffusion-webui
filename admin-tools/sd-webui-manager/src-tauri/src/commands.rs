use crate::config::AppConfig;
use crate::server::{self, ServerStatus};
use crate::AppState;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn start_server(state: State<'_, Arc<AppState>>) -> Result<String, String> {
    let sd_dir = {
        let dir = state.sd_webui_dir.read().unwrap();
        dir.clone()
    };

    let sd_dir = sd_dir.ok_or("SD WebUI directory not configured. Please set it in Settings.")?;

    let script_name = if cfg!(target_os = "windows") {
        "webui-user.bat"
    } else {
        "webui-user.sh"
    };

    let script_path = sd_dir.join(script_name);
    if !script_path.exists() {
        return Err(format!("Launch script not found: {}", script_path.display()));
    }

    let extra_args = {
        let config = state.config.read().unwrap();
        config.commandline_args.clone()
    };

    let launch = server::build_launch_command(&sd_dir, extra_args.as_deref());

    state.set_status(ServerStatus::Starting);

    match server::spawn_server(&launch).await {
        Ok(mut child) => {
            let pid = child.id().unwrap_or(0);
            server::read_stdout(&mut child, state.app_handle.clone());

            {
                let mut guard = state.child.lock().await;
                *guard = Some(child);
            }

            state.push_log(format!("[Manager] Server started (PID: {})", pid));
            state.set_status(ServerStatus::Running);

            Ok(format!("Server started (PID: {})", pid))
        }
        Err(e) => {
            state.set_status(ServerStatus::Error(e.to_string()));
            Err(format!("Failed to start server: {}", e))
        }
    }
}

#[tauri::command]
pub async fn stop_server(state: State<'_, Arc<AppState>>) -> Result<String, String> {
    {
        let status = state.status.read().unwrap();
        if *status == ServerStatus::Stopped {
            return Ok("Server is already stopped".to_string());
        }
    }

    state.set_status(ServerStatus::Stopping);

    let mut guard = state.child.lock().await;
    if let Some(child) = guard.as_mut() {
        server::kill_process_tree(child).await.map_err(|e| e.to_string())?;
        *guard = None;
    }

    state.push_log("[Manager] Server stopped".to_string());
    state.set_status(ServerStatus::Stopped);

    Ok("Server stopped".to_string())
}

#[tauri::command]
pub async fn restart_server(state: State<'_, Arc<AppState>>) -> Result<String, String> {
    stop_server(state.clone()).await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    start_server(state).await
}

#[tauri::command]
pub fn get_status(state: State<'_, Arc<AppState>>) -> String {
    let status = state.status.read().unwrap();
    match &*status {
        ServerStatus::Stopped => "stopped".to_string(),
        ServerStatus::Starting => "starting".to_string(),
        ServerStatus::Running => "running".to_string(),
        ServerStatus::Stopping => "stopping".to_string(),
        ServerStatus::Error(e) => format!("error:{}", e),
    }
}

#[tauri::command]
pub fn get_logs(state: State<'_, Arc<AppState>>, last_n: Option<usize>) -> Vec<String> {
    state.get_logs(last_n.unwrap_or(200))
}

#[tauri::command]
pub fn get_config(state: State<'_, Arc<AppState>>) -> AppConfig {
    state.config.read().unwrap().clone()
}

#[tauri::command]
pub fn save_config(
    state: State<'_, Arc<AppState>>,
    config: AppConfig,
) -> Result<String, String> {
    config.save().map_err(|e| e.to_string())?;

    {
        let mut current = state.config.write().unwrap();
        *current = config.clone();
    }
    {
        let mut dir = state.sd_webui_dir.write().unwrap();
        *dir = config.sd_webui_path.as_ref().map(PathBuf::from);
    }

    Ok("Configuration saved".to_string())
}

#[tauri::command]
pub fn detect_sd_webui_path() -> Option<String> {
    server::detect_sd_webui_dir().map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn check_sd_server_health(state: State<'_, Arc<AppState>>) -> Result<String, String> {
    let port = {
        let config = state.config.read().unwrap();
        config.sd_port
    };

    let url = format!("http://127.0.0.1:{}/sdapi/v1/sd-models", port);

    match reqwest::Client::new()
        .get(&url)
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => Ok("healthy".to_string()),
        Ok(resp) => Ok(format!("unhealthy: HTTP {}", resp.status())),
        Err(e) => Ok(format!("unreachable: {}", e)),
    }
}
