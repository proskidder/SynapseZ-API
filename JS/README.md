# Importing
U have to be in a NodeJS environment. BrowserJS does not support it.

Put the SynzApi.js or SynzApi.ts into anywhere you want, but also put synznativeapi.node into the same folder. The node (native addon) gives the Environment the ability to interact with processes (getting & checking roblox processes)

If there are future updates to the javascript / typescript, also update the .node thanks.

# Usage
The functions are pretty much the same just like in the C# API.

```js
// JS - Classic
require("SynzApi.js")

// TS - Classic
import * as SynzAPI from "SynzApi.ts"

// ESM
import { createRequire } from 'module'; 
import path from 'path'; 
const modulesPath = path.resolve(process.cwd(), 'PUT FOLDER CONTAINING SYNZAPI HERE');
const localRequire = createRequire(modulesPath);
const SynzAPI = localRequire('SynzApi.js'); 
```

then yeah just use the functions it provides ig like


### Example:
```js
SynzAPI.Execute("print('Hello World')")
```