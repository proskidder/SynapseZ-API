# SynapseZAPI Class

This API was developed in .NET-Framework 4.8!
You might need to adjust it to make it work for your version.

## How to initialize:
```cs
using SynapseZ; // thats it lol
```

## How to inject:
```cs
// u dont inject lol, its already injected when u run roblox
```

## How to execute: (OLD API)
```cs
/**
 * Return values:
 * 0 - Execution successful
 * 1 - Bin Folder not found
 * 2 - Scheduler Folder not found
 * 3 - No access to write file
*/

SynapseZAPI.Execute({{SCRIPTHERE}});

// OR IF YOU HAVE THE PID OF THE ROBLOX PROCESS:

SynapseZAPI.Execute({{SCRIPTHERE}}, {{PID}});
```

## Information:

Incase you get an output which is not the desired output, to get the error msg associated with it, you can always use:
```cs
SynapseZAPI.GetLatestErrorMessage()
```
which will return the error message which was captured in any of those functions.


# SynapseZAPI2 Class

This class communicates directly with the clients, instead of using filesystem. It uses pipes, etc. If you don't want this, use the stripped version.

So that the internal session checker is started, use:
```cs
SynapseZAPI2.StartInstancesTimer();
```


This also has events when sessions are added and removed, see:

```cs
SynapseZAPI2.SessionAdded += SynapseZAPI2_SessionAdded;
SynapseZAPI2.SessionRemoved += SynapseZAPI2_SessionRemoved;

private void SynapseZAPI2_SessionOutput(SynapseZAPI2.SynapseSession e, int type, string output)
{
    Console.WriteLine("Console Output: " + type + " " + output);
}

private void SynapseZAPI2_SessionRemoved(SynapseZAPI2.SynapseSession e)
{
    Console.WriteLine("Session Removed: " + e.Pid);
}
```

and for the part most people want, console redirection:

```cs
SynapseZAPI2.SessionOutput += SynapseZAPI2_SessionOutput;

private void SynapseZAPI2_SessionAdded(SynapseZAPI2.SynapseSession e)
{
    Console.WriteLine("Session Added: " + e.Pid);
}
```

Here are the output types:
```
0: print
1: info
2: warn
3: error
```

## How to execute (NEW API; USES SESSIONS)
```cs
SynapseZAPI.Execute({{SCRIPTHERE}});

// OR IF YOU HAVE THE PID OF THE ROBLOX PROCESS:
SynapseZAPI.Execute({{SCRIPTHERE}}, {{PID}});
```