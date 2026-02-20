// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Cursor, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use sysinfo::{Pid, System};
use flate2::read::GzDecoder;
use std::io::Read;
use tauri::{AppHandle, Emitter, Manager, State};
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

// NBT deserialization types (for reading player .dat files)
#[derive(Debug, serde::Deserialize)]
struct PlayerDatNbt {
    #[serde(rename = "Health", default)]
    health: Option<f32>,
    #[serde(rename = "foodLevel", default)]
    food_level: Option<i32>,
    #[serde(rename = "XpLevel", default)]
    xp_level: Option<i32>,
    #[serde(rename = "playerGameType", default)]
    player_game_type: Option<i8>,
    #[serde(rename = "Inventory", default)]
    inventory: Option<Vec<NbtInventoryItem>>,
    #[serde(rename = "EnderItems", default)]
    ender_items: Option<Vec<NbtInventoryItem>>,
    #[serde(default)]
    equipment: Option<NbtEquipment>,
    #[serde(rename = "Dimension", default)]
    dimension: Option<String>,
    #[serde(rename = "Pos", default)]
    pos: Option<Vec<f64>>,
    #[serde(rename = "SpawnX", default)]
    spawn_x: Option<i32>,
    #[serde(rename = "SpawnY", default)]
    spawn_y: Option<i32>,
    #[serde(rename = "SpawnZ", default)]
    spawn_z: Option<i32>,
    #[serde(rename = "SpawnDimension", default)]
    spawn_dimension: Option<String>,
    #[serde(rename = "LastDeathLocation", default)]
    last_death_location: Option<NbtDeathLocation>,
}

#[derive(Debug, serde::Deserialize)]
struct NbtInventoryItem {
    #[serde(rename = "Slot")]
    slot: i8,
    id: String,
    #[serde(default)]
    count: Option<i32>,
}

#[derive(Debug, serde::Deserialize)]
struct NbtEquipment {
    #[serde(default)]
    head: Option<NbtItem>,
    #[serde(default)]
    chest: Option<NbtItem>,
    #[serde(default)]
    legs: Option<NbtItem>,
    #[serde(default)]
    feet: Option<NbtItem>,
    #[serde(default)]
    offhand: Option<NbtItem>,
}

#[derive(Debug, serde::Deserialize)]
struct NbtDeathLocation {
    dimension: Option<String>,
    pos: Option<Vec<i32>>,
}

// Stats file deserialization (world/stats/{uuid}.json)
#[derive(Debug, serde::Deserialize)]
struct StatsFile {
    stats: Option<StatsCategories>,
}

#[derive(Debug, serde::Deserialize)]
struct StatsCategories {
    #[serde(rename = "minecraft:custom", default)]
    custom: Option<HashMap<String, u64>>,
    #[serde(rename = "minecraft:picked_up", default)]
    picked_up: Option<HashMap<String, u64>>,
    #[serde(rename = "minecraft:used", default)]
    used: Option<HashMap<String, u64>>,
}

#[derive(Debug, serde::Deserialize)]
struct NbtItem {
    id: String,
    #[serde(default)]
    count: Option<i32>,
}

// Response types for player inventory
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerDetails {
    pub name: String,
    pub uuid: String,
    pub health: f64,
    pub max_health: f64,
    pub food: u32,
    pub xp_level: u32,
    pub game_mode: String,
    pub inventory: Vec<InventorySlot>,
    pub ender_chest: Vec<InventorySlot>,
    pub dimension: String,
    pub pos_x: f64,
    pub pos_y: f64,
    pub pos_z: f64,
    pub last_slept: Option<String>,
    pub last_death: Option<String>,
    pub playtime_ticks: u64,
    pub deaths: u32,
    pub player_kills: u32,
    pub mob_kills: u32,
    pub items_picked_up: u64,
    pub items_used: u64,
    pub distance_cm: u64,
    pub is_op: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySlot {
    pub slot: i32,
    pub id: String,
    pub count: u32,
    pub name: String,
}

#[derive(Debug, Deserialize)]
struct UserCacheEntry {
    name: String,
    uuid: String,
}

// For reading features.toml
#[derive(Debug, Deserialize)]
struct FeaturesConfig {
    #[serde(default)]
    player_data: PlayerDataSection,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct PlayerDataSection {
    save_player_cron_interval: u64,
}

impl Default for PlayerDataSection {
    fn default() -> Self {
        Self {
            save_player_cron_interval: 300,
        }
    }
}

// RCON config returned to frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RconConfig {
    pub enabled: bool,
    pub port: u16,
    pub password: String,
}

fn parse_rcon_config(server_path: &str) -> RconConfig {
    let features_path = PathBuf::from(server_path).join("config").join("features.toml");
    if let Ok(content) = fs::read_to_string(&features_path) {
        // Parse as generic Value to avoid failures from unrelated sections
        if let Ok(val) = content.parse::<toml::Value>() {
            if let Some(rcon) = val.get("networking").and_then(|n| n.get("rcon")) {
                let enabled = rcon
                    .get("enabled")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let address = rcon
                    .get("address")
                    .and_then(|v| v.as_str())
                    .unwrap_or("0.0.0.0:25575");
                let password = rcon
                    .get("password")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let port = address
                    .rsplit_once(':')
                    .and_then(|(_, p)| p.parse::<u16>().ok())
                    .unwrap_or(25575);
                return RconConfig {
                    enabled,
                    port,
                    password: password.to_string(),
                };
            }
        }
    }
    RconConfig {
        enabled: false,
        port: 25575,
        password: String::new(),
    }
}

#[tauri::command]
fn get_rcon_config(path: String) -> Result<RconConfig, String> {
    let features_path = PathBuf::from(&path).join("config").join("features.toml");
    eprintln!("[RCON debug] Looking for: {:?} exists={}", features_path, features_path.exists());
    if let Ok(content) = fs::read_to_string(&features_path) {
        if let Ok(val) = content.parse::<toml::Value>() {
            eprintln!("[RCON debug] Top-level keys: {:?}", val.as_table().map(|t| t.keys().collect::<Vec<_>>()));
            if let Some(net) = val.get("networking") {
                eprintln!("[RCON debug] networking keys: {:?}", net.as_table().map(|t| t.keys().collect::<Vec<_>>()));
            } else {
                eprintln!("[RCON debug] no 'networking' key found");
            }
        } else {
            eprintln!("[RCON debug] TOML parse failed");
        }
    } else {
        eprintln!("[RCON debug] Could not read file");
    }
    Ok(parse_rcon_config(&path))
}

// ── Inline RCON client (Source RCON protocol) ──
// Packet: i32 length, i32 request_id, i32 type, body (null-terminated), pad byte

const RCON_AUTH: i32 = 3;
const RCON_EXEC: i32 = 2;

async fn rcon_send_packet(
    stream: &mut tokio::net::TcpStream,
    request_id: i32,
    packet_type: i32,
    body: &str,
) -> Result<(), String> {
    use tokio::io::AsyncWriteExt;
    let body_bytes = body.as_bytes();
    let length = 4 + 4 + body_bytes.len() as i32 + 2; // id + type + body + 2 nulls
    let mut buf = Vec::with_capacity(length as usize + 4);
    buf.extend_from_slice(&length.to_le_bytes());
    buf.extend_from_slice(&request_id.to_le_bytes());
    buf.extend_from_slice(&packet_type.to_le_bytes());
    buf.extend_from_slice(body_bytes);
    buf.push(0); // body null terminator
    buf.push(0); // pad byte
    stream
        .write_all(&buf)
        .await
        .map_err(|e| format!("RCON write error: {}", e))
}

async fn rcon_read_packet(
    stream: &mut tokio::net::TcpStream,
) -> Result<(i32, i32, String), String> {
    use tokio::io::AsyncReadExt;
    let mut len_buf = [0u8; 4];
    stream
        .read_exact(&mut len_buf)
        .await
        .map_err(|e| format!("RCON read length error: {}", e))?;
    let length = i32::from_le_bytes(len_buf) as usize;
    if !(10..=4096 * 4).contains(&length) {
        return Err(format!("RCON invalid packet length: {}", length));
    }
    let mut payload = vec![0u8; length];
    stream
        .read_exact(&mut payload)
        .await
        .map_err(|e| format!("RCON read payload error: {}", e))?;
    let mut cursor = Cursor::new(&payload);
    let mut id_buf = [0u8; 4];
    let mut type_buf = [0u8; 4];
    std::io::Read::read_exact(&mut cursor, &mut id_buf).map_err(|e| e.to_string())?;
    std::io::Read::read_exact(&mut cursor, &mut type_buf).map_err(|e| e.to_string())?;
    let request_id = i32::from_le_bytes(id_buf);
    let packet_type = i32::from_le_bytes(type_buf);
    let body_len = length - 10; // subtract id(4) + type(4) + 2 null bytes
    let body = if body_len > 0 {
        String::from_utf8_lossy(&payload[8..8 + body_len]).to_string()
    } else {
        String::new()
    };
    Ok((request_id, packet_type, body))
}

async fn rcon_connect_and_auth(
    addr: &str,
    password: &str,
) -> Result<tokio::net::TcpStream, String> {
    let mut stream = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        tokio::net::TcpStream::connect(addr),
    )
    .await
    .map_err(|_| "RCON connection timed out".to_string())?
    .map_err(|e| format!("RCON connect failed: {}", e))?;

    rcon_send_packet(&mut stream, 1, RCON_AUTH, password).await?;
    let (id, _ptype, _body) = rcon_read_packet(&mut stream).await?;
    // Some servers send an empty packet before the auth response
    if id == -1 {
        return Err("RCON authentication failed (bad password)".to_string());
    }
    // Read auth response — may get an extra empty packet first
    if _ptype != 2 {
        let (id2, _ptype2, _body2) = rcon_read_packet(&mut stream).await?;
        if id2 == -1 {
            return Err("RCON authentication failed".to_string());
        }
    }
    Ok(stream)
}

async fn rcon_command(stream: &mut tokio::net::TcpStream, cmd: &str) -> Result<String, String> {
    rcon_send_packet(stream, 2, RCON_EXEC, cmd).await?;
    let (_id, _ptype, body) = rcon_read_packet(stream).await?;
    Ok(body)
}

fn get_config_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("sprout");
    fs::create_dir_all(&config_dir).ok();
    config_dir.join("servers.json")
}

fn get_server_data_dir(server_id: &str) -> PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("sprout")
        .join("servers")
        .join(server_id);
    fs::create_dir_all(&dir).ok();
    dir
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

    let mut cmd = Command::new(&exe_path);
    cmd.current_dir(&server_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::piped());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    let mut child = cmd
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

fn format_item_name(id: &str) -> String {
    id.trim_start_matches("minecraft:")
        .replace('_', " ")
        .split_whitespace()
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(c) => {
                    let upper: String = c.to_uppercase().collect();
                    upper + chars.as_str()
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_dimension(dim: &str) -> String {
    let name = dim.trim_start_matches("minecraft:");
    match name {
        "overworld" => "Overworld".to_string(),
        "the_nether" => "The Nether".to_string(),
        "the_end" => "The End".to_string(),
        _ => format_item_name(dim),
    }
}

fn format_location(x: i32, y: i32, z: i32, dim: &str) -> String {
    format!("{}, {}, {} ({})", x, y, z, format_dimension(dim))
}

struct PlayerStats {
    playtime_ticks: u64,
    deaths: u32,
    player_kills: u32,
    mob_kills: u32,
    items_picked_up: u64,
    items_used: u64,
    distance_cm: u64,
}

fn read_player_stats(server_path: &str, uuid: &str) -> PlayerStats {
    let zero = PlayerStats {
        playtime_ticks: 0,
        deaths: 0,
        player_kills: 0,
        mob_kills: 0,
        items_picked_up: 0,
        items_used: 0,
        distance_cm: 0,
    };

    let stats_path = PathBuf::from(server_path)
        .join("world")
        .join("stats")
        .join(format!("{}.json", uuid));

    if !stats_path.exists() {
        return zero;
    }

    let content = match fs::read_to_string(&stats_path) {
        Ok(c) => c,
        Err(_) => return zero,
    };

    let stats: StatsFile = match serde_json::from_str(&content) {
        Ok(s) => s,
        Err(_) => return zero,
    };

    let categories = match stats.stats {
        Some(c) => c,
        None => return zero,
    };

    let custom = categories.custom.unwrap_or_default();

    let playtime_ticks = custom
        .get("minecraft:play_time")
        .or_else(|| custom.get("minecraft:play_one_minute"))
        .copied()
        .unwrap_or(0);
    let deaths = custom.get("minecraft:deaths").copied().unwrap_or(0) as u32;
    let player_kills = custom.get("minecraft:player_kills").copied().unwrap_or(0) as u32;
    let mob_kills = custom.get("minecraft:mob_kills").copied().unwrap_or(0) as u32;

    let distance_cm: u64 = custom
        .iter()
        .filter(|(k, _)| k.ends_with("_one_cm"))
        .map(|(_, v)| v)
        .sum();

    let items_picked_up: u64 = categories
        .picked_up
        .as_ref()
        .map(|m| m.values().sum())
        .unwrap_or(0);

    let items_used: u64 = categories
        .used
        .as_ref()
        .map(|m| m.values().sum())
        .unwrap_or(0);

    PlayerStats {
        playtime_ticks,
        deaths,
        player_kills,
        mob_kills,
        items_picked_up,
        items_used,
        distance_cm,
    }
}

fn is_player_op(server_path: &str, player_name: &str) -> bool {
    let ops_path = PathBuf::from(server_path).join("data").join("ops.json");
    if !ops_path.exists() {
        return false;
    }
    let content = match fs::read_to_string(&ops_path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let entries: Vec<serde_json::Value> = match serde_json::from_str(&content) {
        Ok(e) => e,
        Err(_) => return false,
    };
    entries.iter().any(|e| {
        e.get("name")
            .and_then(|n| n.as_str())
            .map(|n| n.eq_ignore_ascii_case(player_name))
            .unwrap_or(false)
    })
}

fn sprout_players_path(server_id: &str) -> PathBuf {
    get_server_data_dir(server_id).join("known_players.json")
}

#[tauri::command]
async fn get_known_players(id: String) -> Result<Vec<String>, String> {
    let cache_path = sprout_players_path(&id);
    if !cache_path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&cache_path)
        .map_err(|e| format!("Failed to read player cache: {}", e))?;
    let names: Vec<String> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse player cache: {}", e))?;
    Ok(names)
}

#[tauri::command]
async fn update_known_players(id: String, online: Vec<String>) -> Result<(), String> {
    let cache_path = sprout_players_path(&id);
    let mut known: Vec<String> = if cache_path.exists() {
        fs::read_to_string(&cache_path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let mut changed = false;
    for name in &online {
        if !known.iter().any(|k| k.eq_ignore_ascii_case(name)) {
            known.push(name.clone());
            changed = true;
        }
    }

    if changed {
        let json = serde_json::to_string_pretty(&known)
            .map_err(|e| format!("Failed to serialize player cache: {}", e))?;
        fs::write(&cache_path, json)
            .map_err(|e| format!("Failed to write player cache: {}", e))?;
    }

    Ok(())
}

async fn resolve_player_uuid(server_path: &str, player_name: &str) -> Result<String, String> {
    // Try usercache.json first
    let cache_path = PathBuf::from(server_path).join("usercache.json");
    if cache_path.exists() {
        if let Ok(content) = fs::read_to_string(&cache_path) {
            if let Ok(entries) = serde_json::from_str::<Vec<UserCacheEntry>>(&content) {
                for entry in entries {
                    if entry.name.eq_ignore_ascii_case(player_name) {
                        return Ok(entry.uuid);
                    }
                }
            }
        }
    }

    // Fall back to Mojang API
    let client = reqwest::Client::builder()
        .user_agent("sprout-panel")
        .build()
        .map_err(|e| e.to_string())?;

    let url = format!(
        "https://api.mojang.com/users/profiles/minecraft/{}",
        player_name
    );

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Mojang API error: {}", e))?;

    if resp.status().is_success() {
        let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        if let Some(id) = body["id"].as_str() {
            if id.len() == 32 {
                let uuid = format!(
                    "{}-{}-{}-{}-{}",
                    &id[0..8],
                    &id[8..12],
                    &id[12..16],
                    &id[16..20],
                    &id[20..32]
                );
                return Ok(uuid);
            }
        }
    }

    Err(format!(
        "Could not resolve UUID for player '{}'",
        player_name
    ))
}

#[tauri::command]
async fn get_player_inventory(
    _id: String,
    path: String,
    player_name: String,
) -> Result<PlayerDetails, String> {
    let uuid = resolve_player_uuid(&path, &player_name).await?;

    // Try to find the player data file
    let dat_path = PathBuf::from(&path)
        .join("world")
        .join("playerdata")
        .join(format!("{}.dat", uuid));

    if !dat_path.exists() {
        return Err(format!(
            "Player data file not found for '{}' (looked in world/playerdata/)",
            player_name
        ));
    }

    // Read and decompress gzip NBT
    let file =
        fs::File::open(&dat_path).map_err(|e| format!("Failed to open player data: {}", e))?;
    let mut decoder = GzDecoder::new(file);
    let mut data = Vec::new();
    decoder
        .read_to_end(&mut data)
        .map_err(|e| format!("Failed to decompress player data: {}", e))?;

    // Parse NBT
    let nbt: PlayerDatNbt =
        fastnbt::from_bytes(&data).map_err(|e| format!("Failed to parse player NBT: {}", e))?;

    // Build inventory slots
    let mut inventory = Vec::new();

    // Main inventory (slots 0-35)
    if let Some(items) = &nbt.inventory {
        for item in items {
            if item.id != "minecraft:air" {
                inventory.push(InventorySlot {
                    slot: item.slot as i32,
                    id: item.id.clone(),
                    count: item.count.unwrap_or(1).max(0) as u32,
                    name: format_item_name(&item.id),
                });
            }
        }
    }

    // Equipment (Pumpkin stores armor/offhand in a named compound)
    if let Some(equip) = &nbt.equipment {
        let equipment_map: [(i32, &Option<NbtItem>); 5] = [
            (103, &equip.head),
            (102, &equip.chest),
            (101, &equip.legs),
            (100, &equip.feet),
            (-106, &equip.offhand),
        ];
        for (slot, item_opt) in equipment_map {
            if let Some(item) = item_opt {
                if item.id != "minecraft:air" {
                    inventory.push(InventorySlot {
                        slot,
                        id: item.id.clone(),
                        count: item.count.unwrap_or(1).max(0) as u32,
                        name: format_item_name(&item.id),
                    });
                }
            }
        }
    }

    // Build ender chest slots
    let mut ender_chest = Vec::new();
    if let Some(items) = &nbt.ender_items {
        for item in items {
            if item.id != "minecraft:air" {
                ender_chest.push(InventorySlot {
                    slot: item.slot as i32,
                    id: item.id.clone(),
                    count: item.count.unwrap_or(1).max(0) as u32,
                    name: format_item_name(&item.id),
                });
            }
        }
    }

    let game_mode = match nbt.player_game_type.unwrap_or(0) {
        0 => "Survival",
        1 => "Creative",
        2 => "Adventure",
        3 => "Spectator",
        _ => "Unknown",
    };

    // Extract position
    let (pos_x, pos_y, pos_z) = match &nbt.pos {
        Some(p) if p.len() >= 3 => (p[0], p[1], p[2]),
        _ => (0.0, 0.0, 0.0),
    };

    // Extract dimension
    let dimension = nbt.dimension.clone().unwrap_or_else(|| "minecraft:overworld".to_string());

    // Last slept (spawn point set by bed)
    let last_slept = match (nbt.spawn_x, nbt.spawn_y, nbt.spawn_z) {
        (Some(x), Some(y), Some(z)) => {
            let dim = nbt.spawn_dimension.as_deref().unwrap_or("minecraft:overworld");
            Some(format_location(x, y, z, dim))
        }
        _ => None,
    };

    // Last death location
    let last_death = nbt.last_death_location.as_ref().and_then(|loc| {
        let dim = loc.dimension.as_deref()?;
        let pos = loc.pos.as_ref()?;
        if pos.len() >= 3 {
            Some(format_location(pos[0], pos[1], pos[2], dim))
        } else {
            None
        }
    });

    // Stats from file
    let stats = read_player_stats(&path, &uuid);
    let is_op = is_player_op(&path, &player_name);

    Ok(PlayerDetails {
        name: player_name,
        uuid,
        health: nbt.health.unwrap_or(20.0) as f64,
        max_health: 20.0,
        food: nbt.food_level.unwrap_or(20).max(0) as u32,
        xp_level: nbt.xp_level.unwrap_or(0).max(0) as u32,
        game_mode: game_mode.to_string(),
        inventory,
        ender_chest,
        dimension,
        pos_x,
        pos_y,
        pos_z,
        last_slept,
        last_death,
        playtime_ticks: stats.playtime_ticks,
        deaths: stats.deaths,
        player_kills: stats.player_kills,
        mob_kills: stats.mob_kills,
        items_picked_up: stats.items_picked_up,
        items_used: stats.items_used,
        distance_cm: stats.distance_cm,
        is_op,
    })
}

// ── SNBT value extractors (for parsing /data get entity responses) ──

/// Strip the "Player has the following entity data: " prefix from RCON responses
fn extract_snbt_value(response: &str) -> &str {
    // Vanilla: "Player has the following entity data: <value>"
    // Also handle responses like "<player> has the following entity data: <value>"
    if let Some(idx) = response.find("following entity data: ") {
        &response[idx + "following entity data: ".len()..]
    } else {
        response.trim()
    }
}

/// Parse an SNBT number, stripping type suffixes like f, d, b, s, L
fn parse_snbt_number(s: &str) -> f64 {
    let s = s.trim();
    let s = s.trim_end_matches(['f', 'd', 'b', 's', 'L']);
    s.parse::<f64>().unwrap_or(0.0)
}

/// Parse an SNBT inventory list like [{Slot:0b,id:"minecraft:stone",count:64},...] into InventorySlots
fn parse_snbt_inventory(snbt: &str) -> Vec<InventorySlot> {
    let snbt = snbt.trim();
    if snbt == "[]" || snbt.is_empty() {
        return Vec::new();
    }

    // Strip outer brackets
    let inner = if snbt.starts_with('[') && snbt.ends_with(']') {
        &snbt[1..snbt.len() - 1]
    } else {
        snbt
    };

    // Bracket-aware split on commas between items (top-level {} groups)
    let mut items = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;
    for (i, ch) in inner.char_indices() {
        match ch {
            '{' | '[' => depth += 1,
            '}' | ']' => depth -= 1,
            ',' if depth == 0 => {
                let segment = inner[start..i].trim();
                if !segment.is_empty() {
                    items.push(segment);
                }
                start = i + 1;
            }
            _ => {}
        }
    }
    let last = inner[start..].trim();
    if !last.is_empty() {
        items.push(last);
    }

    let mut result = Vec::new();
    for item in items {
        // Each item looks like {Slot:0b,id:"minecraft:stone",count:64}  or {Slot:0b,id:"minecraft:stone",Count:1b}
        let inner_item = item.trim().trim_start_matches('{').trim_end_matches('}');
        let mut slot: i32 = -1;
        let mut id = String::new();
        let mut count: u32 = 1;

        // Parse key:value pairs — bracket-aware
        let mut kv_depth = 0i32;
        let mut kv_start = 0;
        let pairs: Vec<&str> = {
            let mut p = Vec::new();
            for (i, ch) in inner_item.char_indices() {
                match ch {
                    '{' | '[' => kv_depth += 1,
                    '}' | ']' => kv_depth -= 1,
                    ',' if kv_depth == 0 => {
                        p.push(inner_item[kv_start..i].trim());
                        kv_start = i + 1;
                    }
                    _ => {}
                }
            }
            p.push(inner_item[kv_start..].trim());
            p
        };

        for pair in pairs {
            if let Some((key, val)) = pair.split_once(':') {
                let key = key.trim();
                let val = val.trim();
                match key {
                    "Slot" => slot = parse_snbt_number(val) as i32,
                    "id" => id = val.trim_matches('"').to_string(),
                    "count" | "Count" => count = parse_snbt_number(val).max(0.0) as u32,
                    _ => {}
                }
            }
        }

        if !id.is_empty() && id != "minecraft:air" {
            result.push(InventorySlot {
                slot,
                id: id.clone(),
                count,
                name: format_item_name(&id),
            });
        }
    }
    result
}

/// Parse Pumpkin's equipment compound: {head:{id:"...",count:1b},chest:{...},...}
fn parse_snbt_equipment(snbt: &str) -> Vec<InventorySlot> {
    let snbt = snbt.trim();
    if snbt == "{}" || snbt.is_empty() {
        return Vec::new();
    }

    let inner = if snbt.starts_with('{') && snbt.ends_with('}') {
        &snbt[1..snbt.len() - 1]
    } else {
        snbt
    };

    // Map equipment slot names to slot numbers
    let slot_map: &[(&str, i32)] = &[
        ("head", 103),
        ("chest", 102),
        ("legs", 101),
        ("feet", 100),
        ("offhand", -106),
    ];

    let mut result = Vec::new();

    for &(slot_name, slot_num) in slot_map {
        // Find "head:{...}" pattern
        let pattern = format!("{}:", slot_name);
        if let Some(start) = inner.find(&pattern) {
            let after_key = start + pattern.len();
            let rest = &inner[after_key..];
            // Find the matching brace
            if rest.starts_with('{') {
                let mut depth = 0i32;
                let mut end = 0;
                for (i, ch) in rest.char_indices() {
                    match ch {
                        '{' => depth += 1,
                        '}' => {
                            depth -= 1;
                            if depth == 0 {
                                end = i + 1;
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                if end > 0 {
                    let compound = &rest[1..end - 1]; // inner content without braces
                    let mut id = String::new();
                    let mut count: u32 = 1;

                    // Parse key:value pairs
                    let mut kv_depth = 0i32;
                    let mut kv_start = 0;
                    let mut pairs = Vec::new();
                    for (i, ch) in compound.char_indices() {
                        match ch {
                            '{' | '[' => kv_depth += 1,
                            '}' | ']' => kv_depth -= 1,
                            ',' if kv_depth == 0 => {
                                pairs.push(compound[kv_start..i].trim());
                                kv_start = i + 1;
                            }
                            _ => {}
                        }
                    }
                    pairs.push(compound[kv_start..].trim());

                    for pair in pairs {
                        if let Some((key, val)) = pair.split_once(':') {
                            let key = key.trim();
                            let val = val.trim();
                            match key {
                                "id" => id = val.trim_matches('"').to_string(),
                                "count" | "Count" => {
                                    count = parse_snbt_number(val).max(0.0) as u32
                                }
                                _ => {}
                            }
                        }
                    }

                    if !id.is_empty() && id != "minecraft:air" {
                        result.push(InventorySlot {
                            slot: slot_num,
                            id: id.clone(),
                            count,
                            name: format_item_name(&id),
                        });
                    }
                }
            }
        }
    }

    result
}

#[tauri::command]
async fn get_player_inventory_rcon(
    _id: String,
    path: String,
    player_name: String,
) -> Result<PlayerDetails, String> {
    let config = parse_rcon_config(&path);
    if !config.enabled {
        return Err("RCON is not enabled in server.properties".to_string());
    }

    let addr = format!("127.0.0.1:{}", config.port);
    let mut stream = rcon_connect_and_auth(&addr, &config.password).await?;

    // Test if /data get entity is supported by querying Health
    let health_resp = rcon_command(
        &mut stream,
        &format!("data get entity {} Health", player_name),
    )
    .await?;

    // If unsupported command, fall back: save-all then read from file
    if health_resp.contains("Unknown or incomplete command")
        || health_resp.contains("Unknown command")
    {
        // Trigger a world save
        rcon_command(&mut stream, "save-all").await.ok();
        drop(stream);
        // Wait for save to flush
        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
        // Use existing file-based reader
        return get_player_inventory(_id, path, player_name).await;
    }

    let health = parse_snbt_number(extract_snbt_value(&health_resp));

    // Query remaining stats
    let food_resp = rcon_command(
        &mut stream,
        &format!("data get entity {} foodLevel", player_name),
    )
    .await
    .unwrap_or_default();
    let food = parse_snbt_number(extract_snbt_value(&food_resp));

    let xp_resp = rcon_command(
        &mut stream,
        &format!("data get entity {} XpLevel", player_name),
    )
    .await
    .unwrap_or_default();
    let xp_level = parse_snbt_number(extract_snbt_value(&xp_resp));

    let mode_resp = rcon_command(
        &mut stream,
        &format!("data get entity {} playerGameType", player_name),
    )
    .await
    .unwrap_or_default();
    let game_type = parse_snbt_number(extract_snbt_value(&mode_resp)) as i8;

    // Query inventory
    let inv_resp = rcon_command(
        &mut stream,
        &format!("data get entity {} Inventory", player_name),
    )
    .await
    .unwrap_or_default();
    let mut inventory = parse_snbt_inventory(extract_snbt_value(&inv_resp));

    // Query equipment (Pumpkin-specific)
    let equip_resp = rcon_command(
        &mut stream,
        &format!("data get entity {} equipment", player_name),
    )
    .await
    .unwrap_or_default();
    let equip_val = extract_snbt_value(&equip_resp);
    if equip_val.starts_with('{') {
        inventory.extend(parse_snbt_equipment(equip_val));
    }

    // Query ender chest
    let ender_resp = rcon_command(
        &mut stream,
        &format!("data get entity {} EnderItems", player_name),
    )
    .await
    .unwrap_or_default();
    let ender_chest = parse_snbt_inventory(extract_snbt_value(&ender_resp));

    // Query dimension
    let dim_resp = rcon_command(
        &mut stream,
        &format!("data get entity {} Dimension", player_name),
    )
    .await
    .unwrap_or_default();
    let dimension = {
        let raw = extract_snbt_value(&dim_resp);
        let trimmed = raw.trim().trim_matches('"');
        if trimmed.is_empty() {
            "minecraft:overworld".to_string()
        } else {
            trimmed.to_string()
        }
    };

    // Query position
    let pos_resp = rcon_command(
        &mut stream,
        &format!("data get entity {} Pos", player_name),
    )
    .await
    .unwrap_or_default();
    let (pos_x, pos_y, pos_z) = {
        let raw = extract_snbt_value(&pos_resp)
            .trim()
            .trim_start_matches('[')
            .trim_end_matches(']');
        let parts: Vec<f64> = raw.split(',').map(parse_snbt_number).collect();
        (
            parts.first().copied().unwrap_or(0.0),
            parts.get(1).copied().unwrap_or(0.0),
            parts.get(2).copied().unwrap_or(0.0),
        )
    };

    // Query spawn point (bed location)
    let spawn_x_resp = rcon_command(&mut stream, &format!("data get entity {} SpawnX", player_name)).await.unwrap_or_default();
    let spawn_y_resp = rcon_command(&mut stream, &format!("data get entity {} SpawnY", player_name)).await.unwrap_or_default();
    let spawn_z_resp = rcon_command(&mut stream, &format!("data get entity {} SpawnZ", player_name)).await.unwrap_or_default();
    let spawn_dim_resp = rcon_command(&mut stream, &format!("data get entity {} SpawnDimension", player_name)).await.unwrap_or_default();

    let last_slept = {
        let sx = parse_snbt_number(extract_snbt_value(&spawn_x_resp));
        let sy = parse_snbt_number(extract_snbt_value(&spawn_y_resp));
        let sz = parse_snbt_number(extract_snbt_value(&spawn_z_resp));
        let sd = extract_snbt_value(&spawn_dim_resp).trim().trim_matches('"').to_string();
        // If all coords are 0 and dimension is empty, there's no spawn set
        if sd.is_empty() && sx == 0.0 && sy == 0.0 && sz == 0.0 {
            None
        } else {
            let dim = if sd.is_empty() { "minecraft:overworld" } else { &sd };
            Some(format_location(sx as i32, sy as i32, sz as i32, dim))
        }
    };

    let uuid = resolve_player_uuid(&path, &player_name).await.unwrap_or_default();

    // Stats from file (RCON doesn't expose stats directly)
    let stats = read_player_stats(&path, &uuid);

    // Last death from file as well (complex SNBT compound)
    let last_death = {
        let dat_path = PathBuf::from(&path)
            .join("world")
            .join("playerdata")
            .join(format!("{}.dat", uuid));
        if dat_path.exists() {
            fs::File::open(&dat_path).ok().and_then(|file| {
                let mut decoder = GzDecoder::new(file);
                let mut data = Vec::new();
                decoder.read_to_end(&mut data).ok()?;
                let nbt: PlayerDatNbt = fastnbt::from_bytes(&data).ok()?;
                nbt.last_death_location.as_ref().and_then(|loc| {
                    let dim = loc.dimension.as_deref()?;
                    let pos = loc.pos.as_ref()?;
                    if pos.len() >= 3 {
                        Some(format_location(pos[0], pos[1], pos[2], dim))
                    } else {
                        None
                    }
                })
            })
        } else {
            None
        }
    };

    let game_mode = match game_type {
        0 => "Survival",
        1 => "Creative",
        2 => "Adventure",
        3 => "Spectator",
        _ => "Unknown",
    };
    let is_op = is_player_op(&path, &player_name);

    Ok(PlayerDetails {
        name: player_name,
        uuid,
        health,
        max_health: 20.0,
        food: food.max(0.0) as u32,
        xp_level: xp_level.max(0.0) as u32,
        game_mode: game_mode.to_string(),
        inventory,
        ender_chest,
        dimension,
        pos_x,
        pos_y,
        pos_z,
        last_slept,
        last_death,
        playtime_ticks: stats.playtime_ticks,
        deaths: stats.deaths,
        player_kills: stats.player_kills,
        mob_kills: stats.mob_kills,
        items_picked_up: stats.items_picked_up,
        items_used: stats.items_used,
        distance_cm: stats.distance_cm,
        is_op,
    })
}

#[tauri::command]
fn get_save_interval(path: String) -> Result<u64, String> {
    let features_path = PathBuf::from(&path).join("config").join("features.toml");
    if !features_path.exists() {
        return Ok(300);
    }
    let content = fs::read_to_string(&features_path).map_err(|e| e.to_string())?;
    let config: FeaturesConfig = toml::from_str(&content).map_err(|e| e.to_string())?;
    Ok(config.player_data.save_player_cron_interval)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WindowState {
    width: f64,
    height: f64,
    x: i32,
    y: i32,
}

fn window_state_path() -> PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("sprout");
    let _ = fs::create_dir_all(&dir);
    dir.join("window-state.json")
}

fn load_window_state() -> Option<WindowState> {
    let data = fs::read_to_string(window_state_path()).ok()?;
    serde_json::from_str(&data).ok()
}

fn save_window_state(state: &WindowState) {
    if let Ok(json) = serde_json::to_string(state) {
        let _ = fs::write(window_state_path(), json);
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(ServerProcesses::default())
        .manage(SystemMonitor {
            sys: Mutex::new(System::new()),
        })
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            if let Some(state) = load_window_state() {
                let _ = window.set_size(tauri::LogicalSize::new(state.width, state.height));
                let _ = window.set_position(tauri::LogicalPosition::new(state.x as f64, state.y as f64));
            }

            let w = window.clone();
            window.on_window_event(move |event| {
                use tauri::WindowEvent;
                match event {
                    WindowEvent::Resized(_) | WindowEvent::Moved(_) => {
                        if let (Ok(size), Ok(pos)) = (w.inner_size(), w.outer_position()) {
                            let scale = w.scale_factor().unwrap_or(1.0);
                            save_window_state(&WindowState {
                                width: size.width as f64 / scale,
                                height: size.height as f64 / scale,
                                x: pos.x,
                                y: pos.y,
                            });
                        }
                    }
                    _ => {}
                }
            });

            Ok(())
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
            get_known_players,
            update_known_players,
            get_player_inventory,
            get_player_inventory_rcon,
            get_rcon_config,
            get_save_interval,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
