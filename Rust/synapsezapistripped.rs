use chrono::{DateTime, TimeZone, Utc};
use lazy_static::lazy_static;
use rand::{Rng, RngExt};
use std::collections::HashMap;
use std::ffi::CString;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;
use sysinfo::{Pid, Process, ProcessesToUpdate, System};

lazy_static! {
    static ref LATEST_ERROR_MSG: Mutex<String> = Mutex::new(String::new());
}

fn set_error(msg: &str) {
    if let Ok(mut err) = LATEST_ERROR_MSG.lock() {
        *err = msg.to_string();
    }
}

pub struct SynapseZAPI;

impl SynapseZAPI {
    pub fn get_latest_error_message() -> String {
        LATEST_ERROR_MSG.lock().unwrap().clone()
    }

    pub fn execute(script: &str, pid: u32) -> i32 {
        let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();
        let main_path = PathBuf::from(&local_app_data).join("Synapse Z");
        let bin_path = main_path.join("bin");

        if !bin_path.exists() {
            set_error("Bin Folder not found");
            return 1;
        }

        let scheduler_path = bin_path.join("scheduler");
        if !scheduler_path.exists() {
            set_error("Scheduler Folder not found");
            return 2;
        }

        let random_file_name = format!("{}.lua", Self::random_string(10));
        let file_path = if pid == 0 {
            scheduler_path.join(random_file_name)
        } else {
            scheduler_path.join(format!("PID{}_{}", pid, random_file_name))
        };

        match std::fs::write(&file_path, format!("{}@@FileFullyWritten@@", script)) {
            Ok(_) => 0,
            Err(e) => {
                set_error(&e.to_string());
                3
            }
        }
    }

    pub fn get_expire_date() -> Option<DateTime<Utc>> {
        let acc_key = Self::get_account_key();
        if acc_key.is_empty() {
            set_error("Could not find Account Key");
            return None;
        }

        let client = reqwest::blocking::Client::new();
        let res = client
            .get("https://z-api.synapse.do/info")
            .header("User-Agent", "SYNZ-SERVICE")
            .header("key", &acc_key)
            .send();

        match res {
            Ok(response) => {
                if response.status().as_u16() != 418 {
                    set_error(&format!("API Error: {}", response.status()));
                    return None;
                }

                if let Ok(body) = response.text() {
                    if let Ok(expire_sec) = body.trim_matches(char::from(0)).trim().parse::<i64>() {
                        return Utc.timestamp_opt(expire_sec, 0).single();
                    }
                }

                set_error("API Error: Invalid response format");
                None
            }
            Err(_) => {
                set_error("API Error: Failed to connect");
                None
            }
        }
    }

    pub async fn get_expire_date_async() -> Option<DateTime<Utc>> {
        let acc_key = Self::get_account_key();
        if acc_key.is_empty() {
            set_error("Could not find Account Key");
            return None;
        }

        let client = reqwest::Client::new();
        let res = client
            .get("https://z-api.synapse.do/info")
            .header("User-Agent", "SYNZ-SERVICE")
            .header("key", &acc_key)
            .send()
            .await;

        match res {
            Ok(response) => {
                if response.status().as_u16() != 418 {
                    set_error(&format!("API Error: {}", response.status()));
                    return None;
                }

                if let Ok(body) = response.text().await {
                    if let Ok(expire_sec) = body.trim_matches(char::from(0)).trim().parse::<i64>() {
                        return Utc.timestamp_opt(expire_sec, 0).single();
                    }
                }

                set_error("API Error: Invalid response format");
                None
            }
            Err(_) => {
                set_error("API Error: Failed to connect");
                None
            }
        }
    }

    pub fn redeem(license: &str) -> i32 {
        let acc_key = Self::get_account_key();
        if acc_key.is_empty() {
            set_error("Could not find Account Key");
            return -1;
        }

        let client = reqwest::blocking::Client::new();
        match client
            .post("https://z-api.synapse.do/redeem")
            .header("User-Agent", "SYNZ-SERVICE")
            .header("key", &acc_key)
            .header("license", license)
            .send()
        {
            Ok(response) => {
                let status = response.status().as_u16();
                if status != 418 {
                    if status == 403 {
                        set_error("Invalid License");
                        return -3;
                    }
                    set_error(&format!("API Error: {}", status));
                    return -2;
                }

                if let Ok(body) = response.text() {
                    if body.starts_with("Added") {
                        return 0;
                    }
                }
                set_error("Invalid License");
                -3
            }
            Err(_) => {
                set_error("API Error: Failed to connect");
                -2
            }
        }
    }

    pub async fn redeem_async(license: &str) -> i32 {
        let acc_key = Self::get_account_key();
        if acc_key.is_empty() {
            set_error("Could not find Account Key");
            return -1;
        }

        let client = reqwest::Client::new();
        match client
            .post("https://z-api.synapse.do/redeem")
            .header("User-Agent", "SYNZ-SERVICE")
            .header("key", &acc_key)
            .header("license", license)
            .send()
            .await
        {
            Ok(response) => {
                let status = response.status().as_u16();
                if status != 418 {
                    if status == 403 {
                        set_error("Invalid License");
                        return -3;
                    }
                    set_error(&format!("API Error: {}", status));
                    return -2;
                }

                if let Ok(body) = response.text().await {
                    if body.starts_with("Added") {
                        return 0;
                    }
                }
                set_error("Invalid License");
                -3
            }
            Err(_) => {
                set_error("API Error: Failed to connect");
                -2
            }
        }
    }

    pub fn reset_hwid() -> i32 {
        let acc_key = Self::get_account_key();
        if acc_key.is_empty() {
            set_error("Could not find Account Key");
            return -1;
        }

        let client = reqwest::blocking::Client::new();
        match client
            .post("https://z-api.synapse.do/resethwid")
            .header("User-Agent", "SYNZ-SERVICE")
            .header("key", &acc_key)
            .send()
        {
            Ok(response) => match response.status().as_u16() {
                418 => 0,
                429 => { set_error("Cooldown"); -3 },
                403 => { set_error("Blacklisted"); -4 },
                status => { set_error(&format!("API Error: {}", status)); -2 }
            },
            Err(_) => {
                set_error("API Error: Failed to connect");
                -2
            }
        }
    }

    pub async fn reset_hwid_async() -> i32 {
        let acc_key = Self::get_account_key();
        if acc_key.is_empty() {
            set_error("Could not find Account Key");
            return -1;
        }

        let client = reqwest::Client::new();
        match client
            .post("https://z-api.synapse.do/resethwid")
            .header("User-Agent", "SYNZ-SERVICE")
            .header("key", &acc_key)
            .send()
            .await
        {
            Ok(response) => match response.status().as_u16() {
                418 => 0,
                429 => { set_error("Cooldown"); -3 },
                403 => { set_error("Blacklisted"); -4 },
                status => { set_error(&format!("API Error: {}", status)); -2 }
            },
            Err(_) => {
                set_error("API Error: Failed to connect");
                -2
            }
        }
    }

    pub fn get_roblox_processes(sys: &mut System) -> Vec<(u32, PathBuf)> {
        sys.refresh_processes(ProcessesToUpdate::All, true);
        let mut processes = Vec::new();

        for (pid, process) in sys.processes() {
            if process.name().eq_ignore_ascii_case("RobloxPlayerBeta.exe") {
                if let Some(exe_path) = process.exe() {
                    processes.push((pid.as_u32(), exe_path.to_path_buf()));
                }
            }
        }
        processes
    }

    pub fn is_synz(pid: u32, sys: &mut System) -> bool {
        let target_pid = Pid::from_u32(pid);
        sys.refresh_processes(ProcessesToUpdate::Some(&[target_pid]), true);

        if let Some(process) = sys.process(target_pid) {
            if let Some(exe_path) = process.exe() {
                Self::is_synz_path(exe_path)
            } else {
                false
            }
        } else {
            false
        }
    }

    fn is_synz_path(path: &std::path::Path) -> bool {
        if path.as_os_str().is_empty() { return false; }

        match File::open(path) {
            Ok(mut file) => {
                let mut buffer =[0u8; 0x1000];
                if let Ok(bytes_read) = file.read(&mut buffer) {
                    buffer[..bytes_read].windows(4).any(|window| window == b".grh")
                } else {
                    false
                }
            }
            Err(_) => false,
        }
    }

    pub fn get_account_key() -> String {
        let path = PathBuf::from(std::env::var("LOCALAPPDATA").unwrap_or_default())
            .join("auth_v2.syn");
        std::fs::read_to_string(path).unwrap_or_default().trim().to_string()
    }

    fn random_string(length: usize) -> String {
        const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut rng = rand::rng();
        (0..length).map(|_| CHARS[rng.random_range(0..CHARS.len())] as char).collect()
    }
}