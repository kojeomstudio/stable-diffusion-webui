use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::config::AppConfig;
use crate::server;

#[derive(Parser, Clone)]
#[command(name = "sd-webui-manager", about = "SD WebUI Server Manager", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<CliCommand>,

    #[arg(long, help = "Path to SD WebUI directory")]
    pub sd_path: Option<String>,

    #[arg(long, default_value = "9786", help = "Management API port")]
    pub api_port: u16,

    #[arg(long, default_value = "7860", help = "SD WebUI server port")]
    pub sd_port: u16,
}

#[derive(Subcommand, Clone)]
pub enum CliCommand {
    #[command(about = "Start the SD WebUI server")]
    Start {
        #[arg(long, help = "Extra command line args for launch.py")]
        args: Option<String>,
    },
    #[command(about = "Stop the SD WebUI server")]
    Stop {},
    #[command(about = "Restart the SD WebUI server")]
    Restart {
        #[arg(long, help = "Extra command line args for launch.py")]
        args: Option<String>,
    },
    #[command(about = "Get server status")]
    Status {},
    #[command(about = "Show recent log output")]
    Logs {
        #[arg(short, long, default_value = "50", help = "Number of lines")]
        n: usize,
    },
    #[command(about = "Check SD WebUI server health")]
    Health {},
    #[command(about = "Auto-detect SD WebUI installation path")]
    Detect {},
    #[command(about = "Show current configuration")]
    Config {},
    #[command(about = "Wait until SD WebUI server is healthy")]
    Wait {
        #[arg(short, long, default_value = "300", help = "Timeout in seconds")]
        timeout: u64,
        #[arg(short, long, default_value = "5", help = "Check interval in seconds")]
        interval: u64,
    },
}

pub async fn run_cli(cli: Cli) -> i32 {
    let config = AppConfig::load().unwrap_or_default();
    let api_port = if cli.api_port != 9786 { cli.api_port } else { config.api_port };
    let sd_port = if cli.sd_port != 7860 { cli.sd_port } else { config.sd_port };

    let sd_path_override = cli.sd_path.clone();
    let command = match cli.command {
        Some(c) => c,
        None => return -1,
    };

    match command {
        CliCommand::Start { args } => cli_start(&config, sd_path_override.as_deref(), args.as_deref()).await,
        CliCommand::Stop {} => cli_api_post(api_port, "stop").await,
        CliCommand::Restart { args: _ } => cli_api_post(api_port, "restart").await,
        CliCommand::Status {} => cli_status(api_port).await,
        CliCommand::Logs { n } => cli_logs(api_port, n).await,
        CliCommand::Health {} => cli_health(sd_port).await,
        CliCommand::Detect {} => cli_detect(),
        CliCommand::Config {} => cli_show_config(&config),
        CliCommand::Wait { timeout, interval } => cli_wait(sd_port, timeout, interval).await,
    }
}

fn resolve_sd_path<'a>(config: &'a AppConfig, cli_override: Option<&'a str>) -> Option<PathBuf> {
    cli_override
        .map(PathBuf::from)
        .or_else(|| config.sd_webui_path.as_ref().map(PathBuf::from))
}

async fn cli_start(config: &AppConfig, sd_path_override: Option<&str>, extra_args: Option<&str>) -> i32 {
    let sd_dir = match resolve_sd_path(config, sd_path_override) {
        Some(p) => p,
        None => {
            eprintln!("Error: SD WebUI directory not configured. Use --sd-path or set in config.");
            return 1;
        }
    };

    let effective_args = extra_args
        .map(String::from)
        .or_else(|| config.commandline_args.clone());

    let launch = server::build_launch_command(&sd_dir, effective_args.as_deref());

    println!("Starting SD WebUI server...");
    println!("  Directory: {}", sd_dir.display());
    if let Some(ref a) = effective_args {
        println!("  Extra args: {}", a);
    }

    match server::spawn_server(&launch).await {
        Ok(child) => {
            let pid = child.id().unwrap_or(0);
            println!("Server started (PID: {})", pid);
            0
        }
        Err(e) => {
            eprintln!("Failed to start: {}", e);
            1
        }
    }
}

async fn cli_api_post(api_port: u16, action: &str) -> i32 {
    let url = format!("http://127.0.0.1:{}/api/{}", api_port, action);
    let client = reqwest::Client::new();

    match client.post(&url).timeout(std::time::Duration::from_secs(10)).send().await {
        Ok(resp) => {
            let status = resp.status();
            match resp.json::<serde_json::Value>().await {
                Ok(json) => {
                    let success = json.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
                    let message = json.get("message").and_then(|v| v.as_str()).unwrap_or("unknown");
                    if success {
                        println!("{}", message);
                        0
                    } else {
                        eprintln!("Error: {}", message);
                        1
                    }
                }
                Err(_) => {
                    eprintln!("HTTP {} - unexpected response", status);
                    1
                }
            }
        }
        Err(e) => {
            if e.is_connect() {
                eprintln!("Error: Manager not running on port {}", api_port);
                eprintln!("Start the manager GUI first, or use 'start' command for direct launch.");
                2
            } else {
                eprintln!("Error: {}", e);
                1
            }
        }
    }
}

async fn cli_status(api_port: u16) -> i32 {
    let url = format!("http://127.0.0.1:{}/api/status", api_port);
    let client = reqwest::Client::new();

    match client.get(&url).timeout(std::time::Duration::from_secs(5)).send().await {
        Ok(resp) => {
            match resp.json::<serde_json::Value>().await {
                Ok(json) => {
                    let status = json.get("status").and_then(|v| v.as_str()).unwrap_or("unknown");
                    let pid = json.get("pid").and_then(|v| v.as_u64());
                    match pid {
                        Some(p) => println!("Status: {} (PID: {})", status, p),
                        None => println!("Status: {}", status),
                    }
                    if status == "running" { 0 } else { 1 }
                }
                Err(e) => {
                    eprintln!("Error parsing response: {}", e);
                    1
                }
            }
        }
        Err(e) => {
            if e.is_connect() {
                println!("Status: manager_not_running");
                2
            } else {
                eprintln!("Error: {}", e);
                1
            }
        }
    }
}

async fn cli_logs(_api_port: u16, _n: usize) -> i32 {
    println!("Logs endpoint not yet available via API. Use the GUI.");
    0
}

async fn cli_health(sd_port: u16) -> i32 {
    let url = format!("http://127.0.0.1:{}/sdapi/v1/sd-models", sd_port);
    let client = reqwest::Client::new();

    match client.get(&url).timeout(std::time::Duration::from_secs(5)).send().await {
        Ok(resp) if resp.status().is_success() => {
            println!("healthy");
            0
        }
        Ok(resp) => {
            println!("unhealthy (HTTP {})", resp.status());
            1
        }
        Err(e) => {
            println!("unreachable ({})", e);
            1
        }
    }
}

fn cli_detect() -> i32 {
    match server::detect_sd_webui_dir() {
        Some(path) => {
            println!("{}", path.display());
            0
        }
        None => {
            eprintln!("Could not auto-detect SD WebUI path");
            1
        }
    }
}

fn cli_show_config(config: &AppConfig) -> i32 {
    match serde_json::to_string_pretty(config) {
        Ok(json) => {
            println!("{}", json);
            0
        }
        Err(e) => {
            eprintln!("Error serializing config: {}", e);
            1
        }
    }
}

async fn cli_wait(sd_port: u16, timeout: u64, interval: u64) -> i32 {
    let url = format!("http://127.0.0.1:{}/sdapi/v1/sd-models", sd_port);
    let client = reqwest::Client::new();
    let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_secs(timeout);

    eprint!("Waiting for SD WebUI on port {} ...", sd_port);

    loop {
        match client.get(&url).timeout(std::time::Duration::from_secs(3)).send().await {
            Ok(resp) if resp.status().is_success() => {
                eprintln!(" OK");
                println!("healthy");
                return 0;
            }
            _ => {
                if tokio::time::Instant::now() >= deadline {
                    eprintln!(" TIMEOUT");
                    eprintln!("Server did not become healthy within {}s", timeout);
                    return 1;
                }
                eprint!(".");
                tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
            }
        }
    }
}
