use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use tauri::Emitter;
use tauri::Manager;
use tokio::process::Child;
use clap::Parser;

mod api_server;
mod cli;
mod commands;
mod config;
mod server;

pub struct AppState {
    pub child: tokio::sync::Mutex<Option<Child>>,
    pub config: RwLock<config::AppConfig>,
    pub logs: Mutex<VecDeque<String>>,
    pub status: RwLock<server::ServerStatus>,
    pub sd_webui_dir: RwLock<Option<PathBuf>>,
    pub app_handle: tauri::AppHandle,
}

impl AppState {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        let config = config::AppConfig::load().unwrap_or_default();
        let sd_webui_dir = config.sd_webui_path.as_ref().map(PathBuf::from);
        Self {
            child: tokio::sync::Mutex::new(None),
            config: RwLock::new(config),
            logs: Mutex::new(VecDeque::with_capacity(10000)),
            status: RwLock::new(server::ServerStatus::Stopped),
            sd_webui_dir: RwLock::new(sd_webui_dir),
            app_handle,
        }
    }

    pub fn push_log(&self, line: String) {
        let mut logs = self.logs.lock().unwrap();
        if logs.len() >= 10000 {
            logs.pop_front();
        }
        logs.push_back(line);
    }

    pub fn get_logs(&self, last_n: usize) -> Vec<String> {
        let logs = self.logs.lock().unwrap();
        let skip = if logs.len() > last_n {
            logs.len() - last_n
        } else {
            0
        };
        logs.iter().skip(skip).cloned().collect()
    }

    pub fn set_status(&self, status: server::ServerStatus) {
        let status_str = match &status {
            server::ServerStatus::Stopped => "stopped",
            server::ServerStatus::Starting => "starting",
            server::ServerStatus::Running => "running",
            server::ServerStatus::Stopping => "stopping",
            server::ServerStatus::Error(_) => "error",
        };
        {
            let mut s = self.status.write().unwrap();
            *s = status;
        }
        let _ = self.app_handle.emit("server-status", status_str);
    }
}

fn main() {
    let cli_args = cli::Cli::try_parse();

    match &cli_args {
        Ok(cli_args) if cli_args.command.is_some() => {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
            let exit_code = rt.block_on(cli::run_cli(cli_args.clone()));
            if exit_code >= 0 {
                std::process::exit(exit_code);
            }
        }
        Err(e) => {
            e.print().expect("Failed to print CLI help");
            std::process::exit(if e.use_stderr() { 1 } else { 0 });
        }
        Ok(_) => {}
    }

    #[cfg(not(debug_assertions))]
    {
        std::env::set_var("TERM", "dumb");
    }

    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            let state = Arc::new(AppState::new(handle));
            app.manage(state.clone());

            let state_for_api = state.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = api_server::start(state_for_api).await {
                    eprintln!("API server error: {}", e);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::start_server,
            commands::stop_server,
            commands::restart_server,
            commands::get_status,
            commands::get_logs,
            commands::get_config,
            commands::save_config,
            commands::detect_sd_webui_path,
            commands::check_sd_server_health,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
