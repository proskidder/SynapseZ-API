# SynapseZ-API

This API was developed in .NET-Framework 4.8!
You might need to adjust it to make it work for your version.

## How to initialize:
```cs
using SynapseZAPI;
public SynapseZAPI.SynapseZAPI synapseZAPI = new SynapseZAPI.SynapseZAPI();
```

## How to inject:
```cs
// u dont inject lol, its already injected when u run roblox
```

## How to execute:
```cs
/**
 * Return values:
 * 0 - Execution successful
 * 1 - Bin Folder not found
 * 2 - Scheduler Folder not found
 * 3 - No access to write file
*/

synapseZAPI.Execute({{SCRIPTHERE}});

// OR IF YOU HAVE THE PID OF THE ROBLOX PROCESS:

synapseZAPI.Execute({{SCRIPTHERE}}, {{PID}});
```

## Information:

Incase you get an output which is not the desired output, to get the error msg associated with it, you can always use:
```cs
GetLatestErrorMessage()
```
which will return the error message which was captured in any of those functions.