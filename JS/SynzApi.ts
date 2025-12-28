import * as fs from 'node:fs';
import * as path from 'node:path';

export interface RbxProcess {
    name : string,
    pid : string,
    path : string
}

const SynzNativeApi : {
    GetRobloxProcesses: () => RbxProcess[]
    IsSynzInstance: (PID : number) => boolean
} = require("./synznativeapi.node")

let LatestErrorMsg = "";

// paths
const LocalAppData : string = process.env.LOCALAPPDATA as string;
const MainPath = path.join(LocalAppData, "Synapse Z");
const BinPath = path.join(MainPath, "bin");
const SchedulerPath = path.join(BinPath, "scheduler");
const AccountKeyPath = path.join(LocalAppData, "auth_v2.syn");

export function GetLatestErrorMessage() : String {
    return LatestErrorMsg;
}

function GetExecutionPath(PID : number) : string | number {
    if (!fs.existsSync(BinPath)) {
        LatestErrorMsg = "Could not find the Bin Folder!";
        return 1;
    }

    if (!fs.existsSync(SchedulerPath)) {
        LatestErrorMsg = "Could not find the Scheduler Folder!";
        return 2;
    }

    let RandomFileName : string = RandomString(10) + ".lua";
    let FilePath : string = PID == 0 ? path.join(SchedulerPath, RandomFileName) : path.join(SchedulerPath, "PID" + PID + "_" + RandomFileName);
    
    return FilePath;
}

/**
 * Return values:
 * 0 - Execution successful
 * 1 - Bin Folder not found
 * 2 - Scheduler Folder not found
 * 3 - No access to write file
*/
export function Execute(Script : string, PID : number) : number {
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
export async function ExecuteAsync(Script : string, PID : number) : Promise<number> {
    let ExecutionPath = GetExecutionPath(PID);

    if (typeof ExecutionPath === "number") {
        return ExecutionPath;
    }

    await fs.promises.writeFile(ExecutionPath, Script);

    return 0;
}

/**
 * Return values:
 * Date - Expire Date in Unix Seconds
 * null - Could not find Account Key
 * null - API Error
*/
export async function GetExpireDate() : Promise<Date | null> {
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

    let data = await res.text()

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
export async function Redeem(license : string) : Promise<number> {
    let AccountKey = await GetAccountKeyAsync()

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
    })

    if (res.status == 418) {
        let body = await res.text();

        if (body.startsWith("Added")) {
            return 0;
        } else {
            LatestErrorMsg = "Invalid License"
            return -3;
        }
    } else if (res.status == 403) {
        LatestErrorMsg = "Invalid License";
        return -3;
    }

    LatestErrorMsg = "API Error: " + res.status
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
export async function ResetHwid() : Promise<number> {
    let AccountKey = await GetAccountKeyAsync()

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
    })

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
            LatestErrorMsg = "API Error: " + res.status
            return -2;
    }
}

export function GetAccountKey() : string | null {
    if (!fs.existsSync(AccountKeyPath))
        return "";

    return fs.readFileSync(AccountKeyPath).toString();
}

export async function GetAccountKeyAsync() : Promise<string | null> {
    if (!fs.existsSync(AccountKeyPath))
        return "";

    return (await fs.promises.readFile(AccountKeyPath)).toString();
}

export function GetRobloxProcesses() : RbxProcess[] {
    return SynzNativeApi.GetRobloxProcesses()
}

export function GetSynzRobloxInstances() : RbxProcess[] {
    let processes : RbxProcess[] = GetRobloxProcesses()
    let res : RbxProcess[] = []
    
    for (let p of processes) {
        if (SynzNativeApi.IsSynzInstance(Number(p.pid))) {
            res.push(p)
        }
    }

    return res
}

export function IsSynz(PID : number) : boolean {
    return SynzNativeApi.IsSynzInstance(PID);
}


export function AreAllInstancesSynz() : boolean {
    let processes = GetRobloxProcesses()
    
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

function RandomString(length : number) : string {
    let r = '';
    let l = randomChars.length;

    for ( let i = 0; i < length; i++ ) {
        r += randomChars.charAt(Math.floor(Math.random() * l));
    }

    return r;
}
