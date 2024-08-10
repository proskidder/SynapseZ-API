# SynapseZ-API

This API uses .NET-Framework 4.8!

## How to initialize:
```cs
using SynapseZAPI;
public SynapseZAPI.SynapseZAPI synapseZAPI = new SynapseZAPI.SynapseZAPI();
```

## How to inject:
```cs
/**
 * Return values:
 * 0 - Injection successful
 * 1 - MainPath is not a valid Directory
 * 2 - Launcher not found in MainPath
 * 3 - Couldn't start the launcher
*/
synapseZAPI.Inject(Directory.GetCurrentDirectory() /*<- MainPath*/);
```

## How to execute:
```cs
/**
 * Return values:
 * 0 - Execution successful
 * 1 - MainPath is not a valid Directory
 * 2 - Bin Folder not found
 * 3 - Scheduler Folder not found
 * 4 - No access to write file
*/

synapseZAPI.Execute(Directory.GetCurrentDirectory() /*<- MainPath*/, {{SCRIPTHERE}});

// OR IF YOU HAVE THE PID OF THE ROBLOX PROCESS:

synapseZAPI.Execute(Directory.GetCurrentDirectory() /*<- MainPath*/, {{SCRIPTHERE}}, {{PID}});
```

## Information:

Incase you get the output of 4 from execution or 3 from the injector, you can always use: 
```cs
GetLatestErrorMessage()
```
which will return the error message which was captured in any of those functions.


Lets say you want the path of the Launcher, then you can use:
```cs
/**
 * Return values:
 * not-empty string -> Path to the launcher
 * empty string -> Couldn't find the launcher
*/

FindLauncher(FolderPath)
```
Which will try to find a launcher in the specified Folder.
