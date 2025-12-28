"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.GetLatestErrorMessage = GetLatestErrorMessage;
exports.Execute = Execute;
exports.ExecuteAsync = ExecuteAsync;
exports.GetExpireDate = GetExpireDate;
exports.Redeem = Redeem;
exports.ResetHwid = ResetHwid;
exports.GetAccountKey = GetAccountKey;
exports.GetAccountKeyAsync = GetAccountKeyAsync;
exports.GetRobloxProcesses = GetRobloxProcesses;
exports.GetSynzRobloxInstances = GetSynzRobloxInstances;
exports.IsSynz = IsSynz;
exports.AreAllInstancesSynz = AreAllInstancesSynz;
const fs = require("node:fs");
const path = require("node:path");
const SynzNativeApi = require("./synznativeapi.node");
let LatestErrorMsg = "";
// paths
const LocalAppData = process.env.LOCALAPPDATA;
const MainPath = path.join(LocalAppData, "Synapse Z");
const BinPath = path.join(MainPath, "bin");
const SchedulerPath = path.join(BinPath, "scheduler");
const AccountKeyPath = path.join(LocalAppData, "auth_v2.syn");
function GetLatestErrorMessage() {
    return LatestErrorMsg;
}
function GetExecutionPath(PID) {
    if (!fs.existsSync(BinPath)) {
        LatestErrorMsg = "Could not find the Bin Folder!";
        return 1;
    }
    if (!fs.existsSync(SchedulerPath)) {
        LatestErrorMsg = "Could not find the Scheduler Folder!";
        return 2;
    }
    let RandomFileName = RandomString(10) + ".lua";
    let FilePath = PID == 0 ? path.join(SchedulerPath, RandomFileName) : path.join(SchedulerPath, "PID" + PID + "_" + RandomFileName);
    return FilePath;
}
/**
 * Return values:
 * 0 - Execution successful
 * 1 - Bin Folder not found
 * 2 - Scheduler Folder not found
 * 3 - No access to write file
*/
function Execute(Script, PID) {
    let ExecutionPath = GetExecutionPath(PID);
    if (typeof ExecutionPath === "number") {
        return ExecutionPath;
    }
    fs.writeFileSync(ExecutionPath, Script + "@@FileFullyWritten@@");
    return 0;
}
/**
 * Return values:
 * 0 - Execution successful
 * 1 - Bin Folder not found
 * 2 - Scheduler Folder not found
 * 3 - No access to write file
*/
async function ExecuteAsync(Script, PID) {
    let ExecutionPath = GetExecutionPath(PID);
    if (typeof ExecutionPath === "number") {
        return ExecutionPath;
    }
    await fs.promises.writeFile(ExecutionPath, Script + "@@FileFullyWritten@@");
    return 0;
}
/**
 * Return values:
 * Date - Expire Date in Unix Seconds
 * null - Could not find Account Key
 * null - API Error
*/
async function GetExpireDate() {
    let AccountKey = await GetAccountKeyAsync();
    if (AccountKey == null) {
        LatestErrorMsg = "Could not find Account Key";
        return null;
    }
    let res = await fetch(`https://z-api.synapse.do/info`, {
        method: "GET",
        headers: {
            "key": AccountKey,
            "USER-AGENT": "SYNZ-SERVICE"
        }
    });
    if (res.status != 418) {
        LatestErrorMsg = `API Error: ${res.status}`;
        return null;
    }
    let data = await res.text();
    let expireDate = new Date(parseInt(data) * 1000);
    return expireDate;
}
/**
 * Return values:
 * 0 - Successfull
 * -1 - Could not find Account Key
 * -2 - API Error
 * -3 - Invalid License
*/
async function Redeem(license) {
    let AccountKey = await GetAccountKeyAsync();
    if (AccountKey == null) {
        LatestErrorMsg = "Could not find Account Key";
        return -1;
    }
    let res = await fetch("https://z-api.synapse.do/redeem", {
        method: "POST",
        headers: {
            "key": AccountKey,
            "USER-AGENT": "SYNZ-SERVICE",
            "license": license
        }
    });
    if (res.status == 418) {
        let body = await res.text();
        if (body.startsWith("Added")) {
            return 0;
        }
        else {
            LatestErrorMsg = "Invalid License";
            return -3;
        }
    }
    else if (res.status == 403) {
        LatestErrorMsg = "Invalid License";
        return -3;
    }
    LatestErrorMsg = "API Error: " + res.status;
    return -2;
}
/**
 * Return values:
 * 0 - Successfull
 * -1 - Could not find Account Key
 * -2 - API Error
 * -3 - Cooldown
 * -4 - Blacklisted
*/
async function ResetHwid() {
    let AccountKey = await GetAccountKeyAsync();
    if (AccountKey == null) {
        LatestErrorMsg = "Could not find Account Key";
        return -1;
    }
    let res = await fetch("https://z-api.synapse.do/resethwid", {
        method: "POST",
        headers: {
            "key": AccountKey,
            "USER-AGENT": "SYNZ-SERVICE",
        }
    });
    switch (res.status) {
        case 418:
            return 0;
        case 429:
            LatestErrorMsg = "Cooldown";
            return -3;
        case 403:
            LatestErrorMsg = "Blacklisted";
            return -4;
        default:
            LatestErrorMsg = "API Error: " + res.status;
            return -2;
    }
}
function GetAccountKey() {
    if (!fs.existsSync(AccountKeyPath))
        return "";
    return fs.readFileSync(AccountKeyPath).toString();
}
async function GetAccountKeyAsync() {
    if (!fs.existsSync(AccountKeyPath))
        return "";
    return (await fs.promises.readFile(AccountKeyPath)).toString();
}
function GetRobloxProcesses() {
    return SynzNativeApi.GetRobloxProcesses();
}
function GetSynzRobloxInstances() {
    let processes = GetRobloxProcesses();
    let res = [];
    for (let p of processes) {
        if (SynzNativeApi.IsSynzInstance(Number(p.pid))) {
            res.push(p);
        }
    }
    return res;
}
function IsSynz(PID) {
    return SynzNativeApi.IsSynzInstance(PID);
}
function AreAllInstancesSynz() {
    let processes = GetRobloxProcesses();
    for (let p of processes) {
        if (!SynzNativeApi.IsSynzInstance(Number(p.pid))) {
            return false;
        }
    }
    return true;
}
/**
 * Yeah you can ignore everything after this part
*/
const randomChars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
function RandomString(length) {
    let r = '';
    let l = randomChars.length;
    for (let i = 0; i < length; i++) {
        r += randomChars.charAt(Math.floor(Math.random() * l));
    }
    return r;
}
