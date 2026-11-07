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

use windows::core::PCSTR;
use windows::Win32::Foundation::{CloseHandle, GENERIC_READ, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::Storage::FileSystem::{
    CreateFileA, ReadFile, WriteFile, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows::Win32::System::Pipes::{
    PeekNamedPipe, SetNamedPipeHandleState, WaitNamedPipeA, PIPE_READMODE_MESSAGE, PIPE_TYPE_MESSAGE,
};

lazy_static! {
    static ref LATEST_ERROR_MSG: Mutex<String> = Mutex::new(String::new());
}

fn set_error(msg: &str) {
    if let Ok(mut err) = LATEST_ERROR_MSG.lock() {
        *err = msg.to_string();
    }
}

struct SafeHandle(HANDLE);

impl SafeHandle {
    fn get(&self) -> HANDLE { self.0 }
    fn is_invalid(&self) -> bool { self.0.is_invalid() || self.0 == INVALID_HANDLE_VALUE }
}

impl Drop for SafeHandle {
    fn drop(&mut self) {
        if !self.is_invalid() {
            unsafe { let _ = CloseHandle(self.0); }
        }
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
            .get("https://api.vmexit.cx/info")
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
            .get("https://api.vmexit.cx/info")
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
            .post("https://api.vmexit.cx/redeem")
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
            .post("https://api.vmexit.cx/redeem")
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
            .post("https://api.vmexit.cx/resethwid")
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
            .post("https://api.vmexit.cx/resethwid")
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

type SessionCallback = Arc<dyn Fn(Arc<SynapseSession>) + Send + Sync>;
type ConsoleCallback = Arc<dyn Fn(Arc<SynapseSession>, i32, String) + Send + Sync>;
type LocalMessageCallback = Arc<dyn Fn(&str, &str, i32) + Send + Sync>;

lazy_static! {
    static ref SESSIONS: Arc<Mutex<HashMap<u32, Arc<SynapseSession>>>> =
        Arc::new(Mutex::new(HashMap::new()));
    static ref TIMER_RUNNING: Mutex<bool> = Mutex::new(false);

    static ref SESSION_ADDED_EVENTS: Mutex<Vec<SessionCallback>> = Mutex::new(Vec::new());
    static ref SESSION_REMOVED_EVENTS: Mutex<Vec<SessionCallback>> = Mutex::new(Vec::new());
    static ref SESSION_OUTPUT_EVENTS: Mutex<Vec<ConsoleCallback>> = Mutex::new(Vec::new());
}

pub struct SynapseSession {
    pub pid: u32,
    pub pipe_name: RwLock<String>,
    pending_command_queue: Mutex<Vec<String>>,
    on_message_callbacks: Mutex<Vec<LocalMessageCallback>>,
}

impl SynapseSession {
    pub fn new(pid: u32) -> Arc<Self> {
        let session = Arc::new(Self {
            pid,
            pipe_name: RwLock::new(String::new()),
            pending_command_queue: Mutex::new(Vec::new()),
            on_message_callbacks: Mutex::new(Vec::new()),
        });

        let session_clone = Arc::clone(&session);
        session.add_on_message_callback(move |command, data, _| {
            session_clone.console_output_internal(command, data);
        });

        session
    }

    pub fn queue_command(&self, command: &str) {
        if let Ok(mut queue) = self.pending_command_queue.lock() {
            queue.push(command.to_string());
        }
    }

    pub fn execute(&self, source: &str) {
        self.queue_command(&format!("execute {}", source));
    }

    pub fn add_on_message_callback<F>(&self, callback: F)
    where
        F: Fn(&str, &str, i32) + Send + Sync + 'static,
    {
        if let Ok(mut callbacks) = self.on_message_callbacks.lock() {
            callbacks.push(Arc::new(callback));
        }
    }

    fn console_output_internal(&self, command: &str, data: &str) {
        if command != "read" { return; }

        if let Some((cmd, payload)) = data.split_once(' ') {
            if cmd == "output" {
                if let Some((type_str, output)) = payload.split_once(' ') {
                    if let Ok(t) = type_str.parse::<i32>() {
                        SynapseZAPI2::trigger_session_output(self.pid, t, output.to_string());
                    }
                }
            } else if cmd == "error" {
                SynapseZAPI2::trigger_session_output(self.pid, 3, payload.to_string());
            }
        }
    }

    pub fn init(self: &Arc<Self>) -> bool {
        let path = CString::new(format!(r"\\.\pipe\synz-{}", self.pid)).unwrap();
        unsafe {
            if WaitNamedPipeA(PCSTR::from_raw(path.as_ptr() as _), 10).is_err() { return false; }
            let h = SafeHandle(CreateFileA(PCSTR::from_raw(path.as_ptr() as _), GENERIC_READ.0|GENERIC_WRITE.0, FILE_SHARE_READ|FILE_SHARE_WRITE, None, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, None).unwrap_or(INVALID_HANDLE_VALUE));
            if h.is_invalid() { return false; }
            let mut mode = PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE;
            let _ = SetNamedPipeHandleState(h.get(), Some(&mode), None, None);
            let _ = WriteFile(h.get(), Some(b"new"), None, None);
            let _ = ReadFile(h.get(), None, None, None);
            let mut avail = 0;
            if PeekNamedPipe(h.get(), None, 0, None, Some(&mut avail), None).is_ok() && avail > 0 {
                let mut buf = vec![0u8; avail as usize];
                let _ = ReadFile(h.get(), Some(&mut buf), None, None);
                if let Ok(name) = String::from_utf8(buf) {
                    *self.pipe_name.write().unwrap() = name.trim_matches(char::from(0)).to_string();
                    let s = Arc::clone(self); thread::spawn(move || s.session_loop());
                    return true;
                }
            }
            false
        }
    }

    fn session_loop(&self) {
        let path = CString::new(self.pipe_name.read().unwrap().clone()).unwrap();
        let pcstr = PCSTR::from_raw(path.as_ptr() as _);
        loop {
            unsafe {
                if WaitNamedPipeA(pcstr, 0xffffffff).is_err() { thread::sleep(Duration::from_millis(10)); continue; }
                let h = SafeHandle(CreateFileA(pcstr, GENERIC_READ.0|GENERIC_WRITE.0, FILE_SHARE_READ|FILE_SHARE_WRITE, None, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, None).unwrap_or(INVALID_HANDLE_VALUE));
                if h.is_invalid() { continue; }
                let mode = PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE;
                let _ = SetNamedPipeHandleState(h.get(), Some(&mode), None, None);
                loop {
                    let mut q = Vec::new(); { std::mem::swap(&mut q, &mut self.pending_command_queue.lock().unwrap()); }
                    q.push("read".to_string());
                    if WriteFile(h.get(), Some(q.len().to_string().as_bytes()), None, None).is_err() { SynapseZAPI2::remove_session(self.pid); return; }
                    for cmd in q {
                        let _ = WriteFile(h.get(), Some(cmd.as_bytes()), None, None);
                        let _ = ReadFile(h.get(), None, None, None);
                        let mut size = 0;
                        if PeekNamedPipe(h.get(), None, 0, None, Some(&mut size), None).is_ok() && size > 0 {
                            let mut b = vec![0u8; size as usize];
                            let _ = ReadFile(h.get(), Some(&mut b), None, None);
                            if let Ok(s) = String::from_utf8(b) {
                                if let Ok(n) = s.trim_matches(char::from(0)).parse::<u64>() {
                                    for i in 0..n {
                                        let mut ds = 0;
                                        let _ = ReadFile(h.get(), None, None, None);
                                        if PeekNamedPipe(h.get(), None, 0, None, Some(&mut ds), None).is_ok() && ds > 0 {
                                            let mut db = vec![0u8; ds as usize];
                                            let _ = ReadFile(h.get(), Some(&mut db), None, None);
                                            if let Ok(ds_str) = String::from_utf8(db) {
                                                for cb in self.on_message_callbacks.lock().unwrap().iter() { cb(&cmd, ds_str.trim_matches(char::from(0)), i as i32); }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    thread::sleep(Duration::from_millis(5));
                }
            }
        }
    }
}

pub struct SynapseZAPI2;

impl SynapseZAPI2 {

    pub fn on_session_added<F: Fn(Arc<SynapseSession>) + Send + Sync + 'static>(callback: F) {
        SESSION_ADDED_EVENTS.lock().unwrap().push(Arc::new(callback));
    }

    pub fn on_session_removed<F: Fn(Arc<SynapseSession>) + Send + Sync + 'static>(callback: F) {
        SESSION_REMOVED_EVENTS.lock().unwrap().push(Arc::new(callback));
    }

    pub fn on_session_output<F: Fn(Arc<SynapseSession>, i32, String) + Send + Sync + 'static>(callback: F) {
        SESSION_OUTPUT_EVENTS.lock().unwrap().push(Arc::new(callback));
    }

    pub fn start_instances_timer() {
        let mut running = TIMER_RUNNING.lock().unwrap();
        if *running { return; }
        *running = true;

        thread::spawn(|| {
            let mut sys = System::new();
            loop {
                if !*TIMER_RUNNING.lock().unwrap() { break; }
                Self::instances_timer_tick(&mut sys);
                thread::sleep(Duration::from_millis(2000));
            }
        });
    }

    pub fn stop_instances_timer() {
        *TIMER_RUNNING.lock().unwrap() = false;
    }

    fn instances_timer_tick(sys: &mut System) {
        let processes = SynapseZAPI::get_roblox_processes(sys);
        let mut sessions_to_init = Vec::new();

        {
            let mut sessions_map = SESSIONS.lock().unwrap();
            for (pid, _) in processes {
                if sessions_map.contains_key(&pid) { continue; }
                if !SynapseZAPI::is_synz(pid, sys) { continue; }

                let session = SynapseSession::new(pid);
                sessions_map.insert(pid, Arc::clone(&session));
                sessions_to_init.push(session);
            }
        }

        for session in sessions_to_init {
            if session.init() {
                let events = SESSION_ADDED_EVENTS.lock().unwrap().clone();
                for event in events {
                    event(Arc::clone(&session));
                }
            } else {
                SESSIONS.lock().unwrap().remove(&session.pid);
            }
        }
    }

    pub fn execute(source: &str, pid: u32) {
        let sessions = SESSIONS.lock().unwrap();
        if pid == 0 {
            for session in sessions.values() { session.execute(source); }
        } else if let Some(session) = sessions.get(&pid) {
            session.execute(source);
        }
    }

    pub fn get_instances() -> HashMap<u32, Arc<SynapseSession>> {
        SESSIONS.lock().unwrap().clone()
    }

    pub(crate) fn remove_session(pid: u32) {
        if let Some(session) = SESSIONS.lock().unwrap().remove(&pid) {
            let events = SESSION_REMOVED_EVENTS.lock().unwrap().clone();
            for event in events {
                event(Arc::clone(&session));
            }
        }
    }

    pub(crate) fn trigger_session_output(pid: u32, out_type: i32, output: String) {
        let session_opt = SESSIONS.lock().unwrap().get(&pid).cloned();

        if let Some(session) = session_opt {
            let events = SESSION_OUTPUT_EVENTS.lock().unwrap().clone();
            for event in events {
                event(Arc::clone(&session), out_type, output.clone());
            }
        }
    }
}