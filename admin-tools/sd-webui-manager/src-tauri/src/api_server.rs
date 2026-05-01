use crate::server::ServerStatus;
use crate::AppState;
use axum::extract::State as AxumState;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
struct ApiResponse {
    success: bool,
    message: String,
}

#[derive(Serialize)]
struct StatusResponse {
    status: String,
    pid: Option<u32>,
}

async fn api_start(AxumState(state): AxumState<Arc<AppState>>) -> (StatusCode, Json<ApiResponse>) {
    let sd_dir = {
        let dir = state.sd_webui_dir.read().unwrap();
        dir.clone()
    };

    let sd_dir = match sd_dir {
        Some(d) => d,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: false,
                    message: "SD WebUI directory not configured".to_string(),
                }),
            );
        }
    };

    {
        let status = state.status.read().unwrap();
        if !matches!(&*status, ServerStatus::Stopped | ServerStatus::Error(_)) {
            return (
                StatusCode::CONFLICT,
                Json(ApiResponse {
                    success: false,
                    message: format!("Server is already {:?}", *status),
                }),
            );
        }
    }

    let extra_args = {
        let config = state.config.read().unwrap();
        config.commandline_args.clone()
    };

    let launch = crate::server::build_launch_command(&sd_dir, extra_args.as_deref());
    state.set_status(ServerStatus::Starting);

    match crate::server::spawn_server(&launch).await {
        Ok(mut child) => {
            let pid = child.id().unwrap_or(0);
            crate::server::read_stdout(&mut child, state.app_handle.clone());

            {
                let mut guard = state.child.lock().await;
                *guard = Some(child);
            }

            state.push_log(format!("[Manager] Server started via API (PID: {})", pid));
            state.set_status(ServerStatus::Running);

            (
                StatusCode::OK,
                Json(ApiResponse {
                    success: true,
                    message: format!("Server started (PID: {})", pid),
                }),
            )
        }
        Err(e) => {
            state.set_status(ServerStatus::Error(e.to_string()));
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: false,
                    message: format!("Failed to start: {}", e),
                }),
            )
        }
    }
}

async fn api_stop(AxumState(state): AxumState<Arc<AppState>>) -> (StatusCode, Json<ApiResponse>) {
    {
        let status = state.status.read().unwrap();
        if matches!(&*status, ServerStatus::Stopped) {
            return (
                StatusCode::OK,
                Json(ApiResponse {
                    success: true,
                    message: "Already stopped".to_string(),
                }),
            );
        }
    }

    state.set_status(ServerStatus::Stopping);

    let mut guard = state.child.lock().await;
    if let Some(child) = guard.as_mut() {
        let _ = crate::server::kill_process_tree(child).await;
        *guard = None;
    }

    state.push_log("[Manager] Server stopped via API".to_string());
    state.set_status(ServerStatus::Stopped);

    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: "Server stopped".to_string(),
        }),
    )
}

async fn api_restart(
    AxumState(state): AxumState<Arc<AppState>>,
) -> (StatusCode, Json<ApiResponse>) {
    {
        let status = state.status.read().unwrap();
        if !matches!(&*status, ServerStatus::Stopped | ServerStatus::Running | ServerStatus::Error(_)) {
            return (
                StatusCode::CONFLICT,
                Json(ApiResponse {
                    success: false,
                    message: format!("Cannot restart in {:?} state", *status),
                }),
            );
        }
    }

    state.set_status(ServerStatus::Stopping);
    {
        let mut guard = state.child.lock().await;
        if let Some(child) = guard.as_mut() {
            let _ = crate::server::kill_process_tree(child).await;
            *guard = None;
        }
    }
    state.set_status(ServerStatus::Stopped);

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let sd_dir = {
        let dir = state.sd_webui_dir.read().unwrap();
        dir.clone()
    };

    let sd_dir = match sd_dir {
        Some(d) => d,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: false,
                    message: "SD WebUI directory not configured".to_string(),
                }),
            );
        }
    };

    let extra_args = {
        let config = state.config.read().unwrap();
        config.commandline_args.clone()
    };

    let launch = crate::server::build_launch_command(&sd_dir, extra_args.as_deref());
    state.set_status(ServerStatus::Starting);

    match crate::server::spawn_server(&launch).await {
        Ok(mut child) => {
            let pid = child.id().unwrap_or(0);
            crate::server::read_stdout(&mut child, state.app_handle.clone());

            {
                let mut guard = state.child.lock().await;
                *guard = Some(child);
            }

            state.push_log(format!("[Manager] Server restarted via API (PID: {})", pid));
            state.set_status(ServerStatus::Running);

            (
                StatusCode::OK,
                Json(ApiResponse {
                    success: true,
                    message: format!("Server restarted (PID: {})", pid),
                }),
            )
        }
        Err(e) => {
            state.set_status(ServerStatus::Error(e.to_string()));
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: false,
                    message: format!("Restart failed: {}", e),
                }),
            )
        }
    }
}

async fn api_status(AxumState(state): AxumState<Arc<AppState>>) -> Json<StatusResponse> {
    let (status_str, pid) = {
        let status = state.status.read().unwrap();
        let s = match &*status {
            ServerStatus::Stopped => "stopped",
            ServerStatus::Starting => "starting",
            ServerStatus::Running => "running",
            ServerStatus::Stopping => "stopping",
            ServerStatus::Error(_) => "error",
        }
        .to_string();
        let p = {
            let child = state.child.try_lock();
            match child {
                Ok(guard) => guard.as_ref().and_then(|c| c.id()),
                Err(_) => None,
            }
        };
        (s, p)
    };

    Json(StatusResponse {
        status: status_str,
        pid,
    })
}

async fn api_health() -> &'static str {
    "ok"
}

pub async fn start(state: Arc<AppState>) -> Result<(), Box<dyn std::error::Error>> {
    let api_port = {
        let config = state.config.read().unwrap();
        config.api_port
    };

    let app = Router::new()
        .route("/api/health", get(api_health))
        .route("/api/start", post(api_start))
        .route("/api/stop", post(api_stop))
        .route("/api/restart", post(api_restart))
        .route("/api/status", get(api_status))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", api_port)).await?;
    println!("Management API listening on http://127.0.0.1:{}", api_port);
    axum::serve(listener, app).await?;

    Ok(())
}
