use std::fs::{self, File};
use std::io::{Read};
use std::path::{PathBuf};
use std::env;

use reqwest::blocking::Client;
use reqwest::StatusCode;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct RbxProcess {
    pub name: String,
    pub pid: u32,
    pub path: String,
}

static mut LATEST_ERROR_MSG: String = String::new();

fn set_latest_error(msg: &str) {
    unsafe { LATEST_ERROR_MSG = msg.to_string() }
}

pub fn get_latest_error_message() -> String {
    unsafe { LATEST_ERROR_MSG.clone() }
}

fn local_app_data() -> PathBuf {
    env::var("LOCALAPPDATA").unwrap_or_default().into()
}

fn main_path() -> PathBuf {
    local_app_data().join("Synapse Z")
}

fn bin_path() -> PathBuf {
    main_path().join("bin")
}

fn scheduler_path() -> PathBuf {
    bin_path().join("scheduler")
}

fn account_key_path() -> PathBuf {
    local_app_data().join("auth_v2.syn")
}

fn random_string(len: usize) -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    format!("{:0>width$x}", nanos, width = len)
}

fn get_execution_path(pid: u32) -> Result<PathBuf, u8> {
    if !bin_path().exists() {
        set_latest_error("Could not find the Bin Folder!");
        return Err(1);
    }
    if !scheduler_path().exists() {
        set_latest_error("Could not find the Scheduler Folder!");
        return Err(2);
    }

    let file_name = format!("{}.lua", random_string(10));
    let file_path = if pid == 0 {
        scheduler_path().join(file_name)
    } else {
        scheduler_path().join(format!("PID{}_{}", pid, file_name))
    };

    Ok(file_path)
}

pub fn execute(script: &str, pid: u32) -> u8 {
    match get_execution_path(pid) {
        Ok(path) => {
            if let Err(_) = fs::write(&path, script.to_owned() + "@@FileFullyWritten@@") {
                set_latest_error("No access to write file");
                return 3;
            }
            0
        }
        Err(code) => code,
    }
}

pub fn get_account_key() -> Option<String> {
    let path = account_key_path();
    if !path.exists() {
        return None;
    }
    Some(fs::read_to_string(path).unwrap_or_default())
}

pub fn get_roblox_processes() -> Vec<RbxProcess> {
    let mut sys = sysinfo::System::new_all();
    sys.refresh_processes();

    sys.processes()
        .values()
        .filter(|p| p.name() == "RobloxPlayerBeta.exe")
        .map(|p| RbxProcess {
            name: p.name().to_string(),
            pid: p.pid().as_u32(),
            path: p.exe().unwrap().to_string_lossy().to_string(),
        })
        .collect()
}

pub fn is_synz(pid: u32) -> bool {
    let process = get_roblox_processes().into_iter().find(|p| p.pid == pid);
    if process.is_none() { return false; }
    let path = &process.unwrap().path;

    if let Ok(mut file) = File::open(path) {
        let mut buffer = vec![0u8; 0x600];
        if let Ok(_) = file.read_exact(&mut buffer) {
            let content = String::from_utf8_lossy(&buffer);
            return content.contains(".grh");
        }
    }
    false
}

pub fn get_synz_roblox_instances() -> Vec<RbxProcess> {
    get_roblox_processes().into_iter()
        .filter(|p| is_synz(p.pid))
        .collect()
}

pub fn are_all_instances_synz() -> bool {
    get_roblox_processes().into_iter().all(|p| is_synz(p.pid))
}

pub fn get_expire_date() -> Option<SystemTime> {
    let key = get_account_key()?;
    let client = Client::new();
    let res = client.get("https://z-api.synapse.do/info")
        .header("key", key)
        .header("USER-AGENT", "SYNZ-SERVICE")
        .send()
        .ok()?;

    if res.status() != StatusCode::IM_A_TEAPOT {
        set_latest_error(&format!("API Error: {}", res.status()));
        return None;
    }

    let text = res.text().ok()?;
    let timestamp: u64 = text.parse().ok()?;
    Some(UNIX_EPOCH + Duration::from_secs(timestamp))
}

pub fn create_account(license: String, create_file: bool) -> Option<String> {
    let client = Client::new();
    let res = client.get("https://z-api.synapse.do/createaccount")
        .header("license", license)
        .header("hwid", "0")
        .header("USER-AGENT", "SYNZ-SERVICE")
        .send()
        .ok()?;

    if res.status() != StatusCode::IM_A_TEAPOT {
        set_latest_error(&format!("API Error: {}", res.status()));
        return None;
    }

    let text = res.text().ok()?;

    if text.len() != 128 {
        if text == "0" {
            set_latest_error("Malformed License");
        } else if text == "2" {
            set_latest_error("License doesn't exist");
        } else if text == "1" { // do this last because this is unlikely
            set_latest_error("API Error: Server assumes HWID 0 is blacklisted");
        }

        return None;
    }

    if create_file {
        fs::write(account_key_path(), &text).ok()?;
    }

    Some(text)
}


pub fn redeem(license: &str) -> i8 {
    let key = match get_account_key() {
        Some(k) => k,
        None => { set_latest_error("Could not find Account Key"); return -1; }
    };

    let client = Client::new();
    let res = client.post("https://z-api.synapse.do/redeem")
        .header("key", key)
        .header("USER-AGENT", "SYNZ-SERVICE")
        .header("license", license)
        .send();

    if let Ok(resp) = res {
        match resp.status() {
            StatusCode::IM_A_TEAPOT => {
                if let Ok(body) = resp.text() {
                    if body.starts_with("Added") { return 0; }
                    set_latest_error("Invalid License");
                    return -3;
                }
            }
            StatusCode::FORBIDDEN => { set_latest_error("Invalid License"); return -3; }
            _ => { set_latest_error(&format!("API Error: {}", resp.status())); return -2; }
        }
    }
    -2
}

pub fn reset_hwid() -> i8 {
    let key = match get_account_key() {
        Some(k) => k,
        None => { set_latest_error("Could not find Account Key"); return -1; }
    };

    let client = Client::new();
    let res = client.post("https://z-api.synapse.do/resethwid")
        .header("key", key)
        .header("USER-AGENT", "SYNZ-SERVICE")
        .send();

    if let Ok(resp) = res {
        match resp.status() {
            StatusCode::IM_A_TEAPOT => 0,
            StatusCode::TOO_MANY_REQUESTS => { set_latest_error("Cooldown"); -3 }
            StatusCode::FORBIDDEN => { set_latest_error("Blacklisted"); -4 }
            _ => { set_latest_error(&format!("API Error: {}", resp.status())); -2 }
        }
    } else { -2 }
}
