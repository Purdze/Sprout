// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use sysinfo::{Pid, System};
use flate2::read::GzDecoder;
use std::io::Read;
use tauri::{AppHandle, Emitter, State};
use futures_util::StreamExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub id: String,
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub logs: Vec<String>,
    pub status: String,
    #[serde(default)]
    pub cpu: f64,
    #[serde(default)]
    pub memory: f64,
    #[serde(default)]
    pub players: u32,
    #[serde(rename = "maxPlayers", default = "default_max_players")]
    pub max_players: u32,
    #[serde(default)]
    pub tps: f64,
    #[serde(rename = "cpuHistory", default)]
    pub cpu_history: Vec<f64>,
    #[serde(rename = "memoryHistory", default)]
    pub memory_history: Vec<f64>,
    #[serde(rename = "tpsHistory", default)]
    pub tps_history: Vec<f64>,
    #[serde(rename = "playerList", default)]
    pub player_list: Vec<String>,
}

fn default_max_players() -> u32 {
    20
}

pub struct ServerProcess {
    child: Child,
    pid: u32,
}

#[derive(Default)]
pub struct ServerProcesses {
    processes: Mutex<HashMap<String, ServerProcess>>,
}

pub struct SystemMonitor {
    sys: Mutex<System>,
}

fn get_config_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("sprout");
    fs::create_dir_all(&config_dir).ok();
    config_dir.join("servers.json")
}

#[tauri::command]
fn save_config(servers: Vec<Server>) -> Result<(), String> {
    let path = get_config_path();
    let json = serde_json::to_string_pretty(&servers).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn load_config() -> Result<Vec<Server>, String> {
    let path = get_config_path();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let servers: Vec<Server> = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    Ok(servers)
}

#[tauri::command]
async fn start_server(
    id: String,
    path: String,
    app: AppHandle,
    state: State<'_, ServerProcesses>,
) -> Result<(), String> {
    let server_path = PathBuf::from(&path);

    // Find the pumpkin executable
    let exe_name = if cfg!(windows) { "pumpkin.exe" } else { "pumpkin" };
    let exe_path = server_path.join(exe_name);

    if !exe_path.exists() {
        return Err(format!("Server executable not found at {:?}", exe_path));
    }

    let mut child = Command::new(&exe_path)
        .current_dir(&server_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start server: {}", e))?;

    // Capture stdout
    if let Some(stdout) = child.stdout.take() {
        let app_clone = app.clone();
        let id_clone = id.clone();
        std::thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines().map_while(Result::ok) {
                app_clone
                    .emit(
                        "server-log",
                        serde_json::json!({
                            "id": id_clone,
                            "log": line
                        }),
                    )
                    .ok();
            }
        });
    }

    // Capture stderr
    if let Some(stderr) = child.stderr.take() {
        let app_clone = app.clone();
        let id_clone = id.clone();
        std::thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines().map_while(Result::ok) {
                app_clone
                    .emit(
                        "server-log",
                        serde_json::json!({
                            "id": id_clone,
                            "log": line
                        }),
                    )
                    .ok();
            }
        });
    }

    let pid = child.id();
    state.processes.lock().unwrap().insert(id, ServerProcess { child, pid });
    Ok(())
}

#[tauri::command]
fn stop_server(id: String, state: State<'_, ServerProcesses>) -> Result<(), String> {
    let mut processes = state.processes.lock().unwrap();
    if let Some(mut server_proc) = processes.remove(&id) {
        // Try to send "stop" command first for graceful shutdown
        if let Some(stdin) = server_proc.child.stdin.as_mut() {
            writeln!(stdin, "stop").ok();
        }
        // Give it a moment, then kill if still running
        std::thread::sleep(std::time::Duration::from_secs(2));
        server_proc.child.kill().ok();
        server_proc.child.wait().ok();
    }
    Ok(())
}

#[tauri::command]
fn send_command(id: String, command: String, state: State<'_, ServerProcesses>) -> Result<(), String> {
    let mut processes = state.processes.lock().unwrap();
    if let Some(server_proc) = processes.get_mut(&id) {
        if let Some(stdin) = server_proc.child.stdin.as_mut() {
            writeln!(stdin, "{}", command).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
fn get_server_stats(
    id: String,
    state: State<'_, ServerProcesses>,
    monitor: State<'_, SystemMonitor>,
) -> Result<(f64, u64), String> {
    let processes = state.processes.lock().unwrap();
    if let Some(server_proc) = processes.get(&id) {
        let mut sys = monitor.sys.lock().unwrap();
        let pid = Pid::from_u32(server_proc.pid);
        sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]));

        if let Some(process) = sys.process(pid) {
            let cpu = process.cpu_usage() as f64;
            let memory = process.memory();
            return Ok((cpu, memory));
        }
    }
    Ok((0.0, 0))
}

#[tauri::command]
fn list_config_files(path: String, dir: String) -> Result<Vec<String>, String> {
    let target_dir = PathBuf::from(&path).join(&dir);
    let mut files = Vec::new();

    if target_dir.exists() {
        for entry in fs::read_dir(&target_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let file_path = entry.path();
            if file_path.is_file() {
                if let Some(name) = file_path.file_name() {
                    files.push(name.to_string_lossy().to_string());
                }
            }
        }
    }
    files.sort();
    Ok(files)
}

#[tauri::command]
fn read_config_file(path: String, dir: String, file: String) -> Result<String, String> {
    let file_path = PathBuf::from(&path).join(&dir).join(&file);
    if !file_path.exists() {
        return Err(format!("File not found: {:?}", file_path));
    }

    // Check if it's a gzip file
    if file.ends_with(".gz") {
        let file = fs::File::open(&file_path).map_err(|e| e.to_string())?;
        let mut decoder = GzDecoder::new(file);
        let mut content = String::new();
        decoder.read_to_string(&mut content).map_err(|e| e.to_string())?;
        Ok(content)
    } else {
        fs::read_to_string(&file_path).map_err(|e| e.to_string())
    }
}

#[tauri::command]
fn save_config_file(path: String, dir: String, file: String, content: String) -> Result<(), String> {
    let file_path = PathBuf::from(&path).join(&dir).join(&file);
    fs::write(&file_path, content).map_err(|e| e.to_string())
}

fn detect_platform() -> Result<(String, String), String> {
    let os = match std::env::consts::OS {
        "windows" => "Windows",
        "linux" => "Linux",
        "macos" => "macOS",
        other => return Err(format!("Unsupported OS: {}", other)),
    };
    let arch = match std::env::consts::ARCH {
        "x86_64" => "X64",
        "aarch64" => "ARM64",
        other => return Err(format!("Unsupported architecture: {}", other)),
    };
    Ok((arch.to_string(), os.to_string()))
}

#[tauri::command]
fn get_platform_info() -> Result<String, String> {
    let (arch, os) = detect_platform()?;
    Ok(format!("{}-{}", arch, os))
}

#[derive(Clone, Serialize)]
struct DownloadProgress {
    percent: f64,
    downloaded: u64,
    total: u64,
}

// Cloudflare types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareZone {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareSrvData {
    pub service: String,
    pub proto: String,
    pub name: String,
    pub priority: u16,
    pub weight: u16,
    pub port: u16,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareDnsRecord {
    pub id: String,
    #[serde(rename = "type")]
    pub record_type: String,
    pub name: String,
    pub content: String,
    pub data: Option<CloudflareSrvData>,
}

fn get_cloudflare_token_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("sprout");
    fs::create_dir_all(&config_dir).ok();
    config_dir.join("cloudflare_token.json")
}

fn cloudflare_client(token: &str) -> Result<reqwest::Client, String> {
    use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", token)).map_err(|e| e.to_string())?,
    );
    reqwest::Client::builder()
        .user_agent("pumpkin-panel")
        .default_headers(headers)
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))
}

#[tauri::command]
async fn verify_and_save_cf_token(token: String) -> Result<(), String> {
    let client = cloudflare_client(&token)?;
    let resp = client
        .get("https://api.cloudflare.com/client/v4/user/tokens/verify")
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let success = body["success"].as_bool().unwrap_or(false);
    if !success {
        let errors = body["errors"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|e| e["message"].as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_else(|| "Unknown error".to_string());
        return Err(format!("Token verification failed: {}", errors));
    }

    let path = get_cloudflare_token_path();
    let json = serde_json::json!({ "token": token });
    fs::write(&path, serde_json::to_string_pretty(&json).unwrap()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn load_cf_token() -> Result<String, String> {
    let path = get_cloudflare_token_path();
    if !path.exists() {
        return Ok(String::new());
    }
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let val: serde_json::Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    Ok(val["token"].as_str().unwrap_or("").to_string())
}

#[tauri::command]
fn delete_cf_token() -> Result<(), String> {
    let path = get_cloudflare_token_path();
    if path.exists() {
        fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn list_cf_zones(token: String) -> Result<Vec<CloudflareZone>, String> {
    let client = cloudflare_client(&token)?;
    let resp = client
        .get("https://api.cloudflare.com/client/v4/zones?per_page=50&status=active")
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    if !body["success"].as_bool().unwrap_or(false) {
        return Err("Failed to list zones".to_string());
    }

    let zones: Vec<CloudflareZone> = body["result"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|z| {
            Some(CloudflareZone {
                id: z["id"].as_str()?.to_string(),
                name: z["name"].as_str()?.to_string(),
            })
        })
        .collect();
    Ok(zones)
}

#[tauri::command]
async fn list_cf_srv_records(
    token: String,
    zone_id: String,
) -> Result<Vec<CloudflareDnsRecord>, String> {
    let client = cloudflare_client(&token)?;
    let url = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/dns_records?type=SRV",
        zone_id
    );
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    if !body["success"].as_bool().unwrap_or(false) {
        return Err("Failed to list DNS records".to_string());
    }

    let records: Vec<CloudflareDnsRecord> = body["result"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|r| serde_json::from_value(r.clone()).ok())
        .collect();
    Ok(records)
}

#[tauri::command]
async fn create_cf_srv_record(
    token: String,
    zone_id: String,
    zone_name: String,
    subdomain: String,
    target: String,
    port: u16,
) -> Result<CloudflareDnsRecord, String> {
    let client = cloudflare_client(&token)?;
    let url = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
        zone_id
    );

    let name = if subdomain.is_empty() {
        format!("_minecraft._tcp.{}", zone_name)
    } else {
        format!("_minecraft._tcp.{}.{}", subdomain, zone_name)
    };

    let payload = serde_json::json!({
        "type": "SRV",
        "data": {
            "service": "_minecraft",
            "proto": "_tcp",
            "name": if subdomain.is_empty() { zone_name.clone() } else { format!("{}.{}", subdomain, zone_name) },
            "priority": 0,
            "weight": 0,
            "port": port,
            "target": target
        },
        "name": name,
        "ttl": 1
    });

    let resp = client
        .post(&url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    if !body["success"].as_bool().unwrap_or(false) {
        let errors = body["errors"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|e| e["message"].as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_else(|| "Unknown error".to_string());
        return Err(format!("Failed to create record: {}", errors));
    }

    let record: CloudflareDnsRecord =
        serde_json::from_value(body["result"].clone()).map_err(|e| e.to_string())?;
    Ok(record)
}

#[tauri::command]
async fn delete_cf_dns_record(
    token: String,
    zone_id: String,
    record_id: String,
) -> Result<(), String> {
    let client = cloudflare_client(&token)?;
    let url = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
        zone_id, record_id
    );
    let resp = client
        .delete(&url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    if !body["success"].as_bool().unwrap_or(false) {
        return Err("Failed to delete DNS record".to_string());
    }
    Ok(())
}

#[tauri::command]
async fn download_server(path: String, app: AppHandle) -> Result<(), String> {
    let (arch, os) = detect_platform()?;

    // Fetch releases from GitHub API
    let client = reqwest::Client::builder()
        .user_agent("pumpkin-panel")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let releases: Vec<serde_json::Value> = client
        .get("https://api.github.com/repos/Pumpkin-MC/Pumpkin/releases")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch releases: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Failed to parse releases: {}", e))?;

    let release = releases
        .first()
        .ok_or_else(|| "No releases found".to_string())?;

    let assets = release["assets"]
        .as_array()
        .ok_or_else(|| "No assets in release".to_string())?;

    // Find matching asset: pumpkin-{ARCH}-{OS}[.exe]
    let asset_prefix = format!("pumpkin-{}-{}", arch, os);
    let asset = assets
        .iter()
        .find(|a| {
            a["name"]
                .as_str()
                .map(|n| n.starts_with(&asset_prefix))
                .unwrap_or(false)
        })
        .ok_or_else(|| format!("No asset found for {}-{}", arch, os))?;

    let download_url = asset["browser_download_url"]
        .as_str()
        .ok_or_else(|| "No download URL for asset".to_string())?;

    let total_size = asset["size"].as_u64().unwrap_or(0);

    // Download the binary with progress
    let response = client
        .get(download_url)
        .send()
        .await
        .map_err(|e| format!("Failed to download: {}", e))?;

    let total = response.content_length().unwrap_or(total_size);

    let exe_name = if os == "Windows" {
        "pumpkin.exe"
    } else {
        "pumpkin"
    };
    let dest_path = PathBuf::from(&path).join(exe_name);

    let mut file = fs::File::create(&dest_path)
        .map_err(|e| format!("Failed to create file: {}", e))?;

    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Download error: {}", e))?;
        std::io::Write::write_all(&mut file, &chunk)
            .map_err(|e| format!("Failed to write: {}", e))?;
        downloaded += chunk.len() as u64;

        let percent = if total > 0 {
            (downloaded as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        app.emit(
            "download-progress",
            DownloadProgress {
                percent,
                downloaded,
                total,
            },
        )
        .ok();
    }

    // Set executable permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&dest_path)
            .map_err(|e| e.to_string())?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&dest_path, perms).map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(ServerProcesses::default())
        .manage(SystemMonitor {
            sys: Mutex::new(System::new()),
        })
        .invoke_handler(tauri::generate_handler![
            save_config,
            load_config,
            start_server,
            stop_server,
            send_command,
            get_server_stats,
            list_config_files,
            read_config_file,
            save_config_file,
            get_platform_info,
            download_server,
            verify_and_save_cf_token,
            load_cf_token,
            delete_cf_token,
            list_cf_zones,
            list_cf_srv_records,
            create_cf_srv_record,
            delete_cf_dns_record,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
