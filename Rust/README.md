# Dependencies

```rust
reqwest = { version = "0.13.2", features = ["blocking"] }
sysinfo = "0.38.4"
lazy_static = "1.4"
rand = "0.10.0"
chrono = "0.4"
windows = { version = "0.62.2", features = ["Win32_System_Pipes"] }
```

You might need to manually edit the api if you plan on using another version.
If you are still using win-api, consider switching to windows since its officialy by microsoft.

windows crate is not needed when using stripped version

# Importing

```rust
mod synapsezapi;
```

# Usage

### SynapseZAPI Class
This class is the class which you use to communicate with the SynZ API.

The execute in this class uses the old format by saving it to the scheduler folder, queueing it for execute.

### SynapseZAPI2 Class
This class communicates directly with the clients, instead of using filesystem. It uses pipes, etc. If you don't want this, use the stripped version.

So that the internal session checker is started, use:

```rust
SynapseZAPI2::start_instances_timer();
SynapseZAPI2::stop_instances_timer(); // <- or to stop it
```

This also has events when sessions are added and removed, see:

```rust
SynapseZAPI2::on_session_added(|session| {
    println!("Session added {}", session.pid);
});

SynapseZAPI2::on_session_removed(|session| {
    println!("Session removed {}", session.pid);
});
```

and for the part most people want, console redirection:

```rust
SynapseZAPI2::on_session_output(|session, output_type, content| {
    println!("Session output {} {}", output_type, content);
});
```

Here are the output types:
```
0: print
1: info
2: warn
3: error
```

### How to execute (NEW API; USES SESSIONS)
```rust
SynapseZAPI2::execute({{SCRIPTHERE}}, 0);

// OR IF YOU HAVE THE PID OF THE ROBLOX PROCESS:
SynapseZAPI2::execute({{SCRIPTHERE}}, {{PID}});
```
