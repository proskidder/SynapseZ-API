using Microsoft.Win32.SafeHandles;
using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Net.Http;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;
using System.Timers;

namespace SynapseZ
{
    public class SynapseZAPI
    {
        private static string LatestErrorMsg = "";

        /**
         * Returns the latest error message from any action.
        */
        public static string GetLatestErrorMessage()
        {
            return LatestErrorMsg;
        }

        /**
         * Return values:
         * 0 - Execution successful
         * 1 - Bin Folder not found
         * 2 - Scheduler Folder not found
         * 3 - No access to write file
        */
        public static int Execute(string Script, int PID = 0)
        {
            string MainPath = Path.Combine(Environment.ExpandEnvironmentVariables("%LOCALAPPDATA%"), "Synapse Z");
            string BinPath = Path.Combine(MainPath, "bin");

            if (!Directory.Exists(BinPath))
            {
                LatestErrorMsg = "Bin Folder not found";
                return 1;
            }

            string SchedulerPath = Path.Combine(BinPath, "scheduler");

            if (!Directory.Exists(SchedulerPath))
            {
                LatestErrorMsg = "Scheduler Folder not found";
                return 2;
            }

            string RandomFileName = RandomString(10) + ".lua";
            string FilePath = PID == 0 ? Path.Combine(SchedulerPath, RandomFileName) : Path.Combine(SchedulerPath, "PID" + PID + "_" + RandomFileName);

            try
            {
                File.WriteAllText(FilePath, Script + "@@FileFullyWritten@@");
            }
            catch (Exception e)
            {
                LatestErrorMsg = e.Message;
                return 3;
            }

            return 0;
        }

        /**
         * Return values:
         * Date - Expire Date in Unix Seconds
         * null - Could not find Account Key
         * null - API Error
        */
        public static Nullable<DateTime> GetExpireDate()
        {
            String accKey = GetAccountKey();

            if (accKey == "")
            {
                LatestErrorMsg = "Could not find Account Key";
                return null;
            }

            HttpClient client = new HttpClient();
            client.DefaultRequestHeaders.Add("User-Agent", "SYNZ-SERVICE");
            client.DefaultRequestHeaders.Add("key", accKey);

            HttpResponseMessage response = client.GetAsync("https://z-api.synapse.do/info").Result;

            if (response.StatusCode.ToString() != "418")
            {
                LatestErrorMsg = "API Error: " + response.StatusCode.ToString();
                return null;
            }

            string responseBody = response.Content.ReadAsStringAsync().Result;
            int expireDate = int.Parse(responseBody);

            return DateTimeOffset.FromUnixTimeSeconds(expireDate).UtcDateTime;
        }

        /**
         * Return values:
         * 0 - Successfull
         * -1 - Could not find Account Key
         * -2 - API Error
         * -3 - Invalid License
        */
        public static int Redeem(String license)
        {
            String accKey = GetAccountKey();

            if (accKey == "")
            {
                LatestErrorMsg = "Could not find Account Key";
                return -1;
            }

            HttpClient client = new HttpClient();
            client.DefaultRequestHeaders.Add("User-Agent", "SYNZ-SERVICE");
            client.DefaultRequestHeaders.Add("key", accKey);
            client.DefaultRequestHeaders.Add("license", license);

            HttpResponseMessage response = client.PostAsync("https://z-api.synapse.do/redeem", null).Result;

            if (response.StatusCode.ToString() != "418")
            {
                if (response.StatusCode.ToString() == "Forbidden")
                {
                    LatestErrorMsg = "Invalid License";
                    return -3;
                }

                LatestErrorMsg = "API Error: " + response.StatusCode.ToString();
                return -2;
            }

            string responseBody = response.Content.ReadAsStringAsync().Result;

            if (responseBody.StartsWith("Added"))
                return 0;


            LatestErrorMsg = "Invalid License";
            return -3;
        }

        /**
         * Return values:
         * 0 - Successfull
         * -1 - Could not find Account Key
         * -2 - API Error
         * -3 - Cooldown
         * -4 - Blacklisted
        */
        public static int ResetHwid()
        {
            String accKey = GetAccountKey();

            if (accKey == "")
            {
                LatestErrorMsg = "Could not find Account Key";
                return -1;
            }

            HttpClient client = new HttpClient();
            client.DefaultRequestHeaders.Add("User-Agent", "SYNZ-SERVICE");
            client.DefaultRequestHeaders.Add("key", accKey);

            HttpResponseMessage response = client.PostAsync("https://z-api.synapse.do/resethwid", null).Result;

            switch (response.StatusCode.ToString())
            {
                case "418":
                    return 0;
                case "429":
                    LatestErrorMsg = "Cooldown";
                    return -3;
                case "Forbidden":
                    LatestErrorMsg = "Blacklisted";
                    return -4;
                default:
                    LatestErrorMsg = "API Error: " + response.StatusCode.ToString();
                    return -2;
            }
        }

        /**
         * Return values:
         * System.Diagnostics.Process[] - Roblox Processes
        */
        public static System.Diagnostics.Process[] GetRobloxProcesses()
        {
            return Process.GetProcessesByName("RobloxPlayerBeta");
        }

        /**
         * Return values:
         * List<Process> - SynZ Instances
        */
        public static List<Process> GetSynzRobloxInstances()
        {
            Process[] processes = GetRobloxProcesses();
            List<Process> injectedProcesses = new List<Process>();

            for (int i = 0; i < processes.Length; i++)
            {
                if (IsSynz(processes[i].Id))
                {
                    injectedProcesses.Add(processes[i]);
                }
            }

            return injectedProcesses;
        }

        /**
            * Return values:
            * bool - If the Instance is a SynZ Instance
        */
        public static bool IsSynz(int PID)
        {
            try
            {
                Process process = Process.GetProcessById(PID);
                string path = process.MainModule.FileName;

                byte[] buffer = new byte[0x1000];
                using (FileStream stream = new FileStream(path, FileMode.Open, FileAccess.Read, FileShare.ReadWrite))
                {
                    stream.Read(buffer, 0, buffer.Length);
                }

                string fileContent = System.Text.Encoding.Default.GetString(buffer);

                return fileContent.Contains(".grh");
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Error checking process: {ex.Message}");
                return false;
            }
        }

        /**
            * Return values:
            * bool - If all Roblox Instances are SynZ Instances
        */
        public static bool AreAllInstancesSynz()
        {
            Process[] processes = GetRobloxProcesses();
            if (processes.Length == 0) return false;

            return GetSynzRobloxInstances().Count == processes.Length;
        }

        public static string GetAccountKey()
        {
            string path = Environment.ExpandEnvironmentVariables("%LOCALAPPDATA%\\auth_v2.syn");

            if (!File.Exists(path))
                return "";

            return File.ReadAllText(path);
        }

        /**
         * Yeah you can ignore everything after this part
        */
        private static Random random = new Random();

        // Generate the random string for File Name in Execute();
        private static string RandomString(int length)
        {
            const string chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
            return new string(Enumerable.Repeat(chars, length)
                .Select(s => s[random.Next(s.Length)]).ToArray());
        }
    }

    public class SynapseZAPI2
    {
        private static System.Timers.Timer Timer;
        private static ConcurrentDictionary<uint, SynapseSession> Sessions = new ConcurrentDictionary<uint, SynapseSession>();

        public delegate void SynapseSessionEventHandler(SynapseSession e);
        public static event SynapseSessionEventHandler SessionAdded;
        public static event SynapseSessionEventHandler SessionRemoved;

        public delegate void SynapseConsoleEventHandler(SynapseSession e, int type, string output);
        public static event SynapseConsoleEventHandler SessionOutput;


        public class SynapseSession
        {
            #region Win32 API Constants & Imports

            private const uint GENERIC_READ = 0x80000000;
            private const uint GENERIC_WRITE = 0x40000000;
            private const uint OPEN_EXISTING = 3;
            private const uint FILE_SHARE_READ = 0x00000001;
            private const uint FILE_SHARE_WRITE = 0x00000002;
            private const uint PIPE_TYPE_MESSAGE = 0x00000004;
            private const uint PIPE_READMODE_MESSAGE = 0x00000002;

            [DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Auto)]
            private static extern SafeFileHandle CreateFile(
                string lpFileName, uint dwDesiredAccess, uint dwShareMode,
                IntPtr lpSecurityAttributes, uint dwCreationDisposition,
                uint dwFlagsAndAttributes, IntPtr hTemplateFile);

            [DllImport("kernel32.dll", SetLastError = true)]
            private static extern bool WaitNamedPipe(string name, int timeout);

            [DllImport("kernel32.dll", SetLastError = true)]
            private static extern bool SetNamedPipeHandleState(
                SafeFileHandle hNamedPipe, ref uint lpMode,
                IntPtr lpMaxCollectionCount, IntPtr lpCollectDataTimeout);

            [DllImport("kernel32.dll", SetLastError = true)]
            private static extern bool PeekNamedPipe(
                SafeFileHandle hNamedPipe, byte[] lpBuffer, uint nBufferSize,
                IntPtr lpBytesRead, out uint lpTotalBytesAvail, IntPtr lpBytesLeftThisMessage);

            [DllImport("kernel32.dll", SetLastError = true)]
            private static extern bool WriteFile(
                SafeFileHandle hFile, byte[] lpBuffer, uint nNumberOfBytesToWrite,
                out uint lpNumberOfBytesWritten, IntPtr lpOverlapped);

            [DllImport("kernel32.dll", SetLastError = true)]
            private static extern bool ReadFile(
                SafeFileHandle hFile, byte[] lpBuffer, uint nNumberOfBytesToRead,
                out uint lpNumberOfBytesRead, IntPtr lpOverlapped);

            #endregion

            public uint Pid { get; private set; } = 0;
            public string PipeName { get; private set; } = "";

            private readonly List<string> _pendingCommandQueue = new List<string>();
            private readonly List<Action<string, string, int>> _onMessageCallbacks = new List<Action<string, string, int>>();
            private readonly object _cycleLock = new object();
            private SynchronizationContext _callerContext = SynchronizationContext.Current;

            public SynapseSession()
            {
                // Caller Context is only valid in WPF / WinForm apps. If you are running a pure C# app, you need to implement your own Background - Main mechanism
                if (_callerContext != null)
                {
                    AddOnMessageCallback(ConsoleOutput);
                } else
                {
                    AddOnMessageCallback(ConsoleOutput__internal);
                }
            }

            public void QueueCommand(string command)
            {
                lock (_cycleLock)
                {
                    _pendingCommandQueue.Add(command);
                }
            }

            public void Execute(string source) => QueueCommand($"execute {source}");

            public void ReloadSettingsInInternalUI() => QueueCommand("reload_settings");

            public void AddOnMessageCallback(Action<string, string, int> callback)
            {
                lock (_cycleLock)
                {
                    _onMessageCallbacks.Add(callback);
                }
            }

            private void ConsoleOutput__internal(string command, string data, int i)
            {
                if (command != "read") return;

                char[] seperator = " ".ToCharArray();
                string[] splitted = data.Split(seperator, count: 2, StringSplitOptions.None);
                command = splitted[0];
                data = splitted[1];

                if (command == "output")
                {
                    string[] splitted2 = data.Split(seperator, count: 2, StringSplitOptions.None);
                    int type = int.Parse(splitted2[0]);
                    string output = splitted2[1];

                    SessionOutput?.Invoke(this, type, output);
                } else if (command == "error")
                {
                    SessionOutput?.Invoke(this, 3, data);
                }
            }

            private void ConsoleOutput(string command, string data, int i)
            {
                _callerContext.Post(_ =>
                {
                    ConsoleOutput__internal(command, data, i);
                }, null);
            }

            public bool Init(uint pid)
            {
                this.Pid = pid;
                string initialPipe = $@"\\.\pipe\synz-{pid}";

                if (!WaitNamedPipe(initialPipe, 10))
                    return false;

                SafeFileHandle handle = CreateFile(initialPipe, GENERIC_READ | GENERIC_WRITE, FILE_SHARE_READ | FILE_SHARE_WRITE, IntPtr.Zero, OPEN_EXISTING, 0, IntPtr.Zero);

                if (handle.IsInvalid) return false;

                uint mode = PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE;
                SetNamedPipeHandleState(handle, ref mode, IntPtr.Zero, IntPtr.Zero);

                byte[] newCmd = Encoding.UTF8.GetBytes("new");
                WriteFile(handle, newCmd, (uint)newCmd.Length, out _, IntPtr.Zero);
                ReadFile(handle, null, 0, out _, IntPtr.Zero);

                uint totalBytesAvail = 0;
                if (PeekNamedPipe(handle, null, 0, IntPtr.Zero, out totalBytesAvail, IntPtr.Zero) && totalBytesAvail > 0)
                {
                    byte[] responseBuffer = new byte[totalBytesAvail];
                    ReadFile(handle, responseBuffer, totalBytesAvail, out _, IntPtr.Zero);
                    PipeName = Encoding.UTF8.GetString(responseBuffer);
                    
                    Thread runner = new Thread(() => SessionLoop());
                    runner.IsBackground = true;
                    runner.Start();

                    handle.Close();
                    return true;
                }

                handle.Close();
                return false;
            }

            private void SessionLoop()
            {
                try
                {
                    while (true)
                    {

                        if (!WaitNamedPipe(PipeName, -1)) // INFINITE = -1
                            continue;

                        using (SafeFileHandle pipe = CreateFile(PipeName, GENERIC_READ | GENERIC_WRITE, FILE_SHARE_READ | FILE_SHARE_WRITE, IntPtr.Zero, OPEN_EXISTING, 0, IntPtr.Zero))
                        {
                            if (pipe.IsInvalid) continue;

                            uint mode = PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE;
                            SetNamedPipeHandleState(pipe, ref mode, IntPtr.Zero, IntPtr.Zero);

                            while (true)
                            {
                                List<string> commandQueue;

                                lock (_cycleLock)
                                {
                                    commandQueue = new List<string>(_pendingCommandQueue);
                                    _pendingCommandQueue.Clear();
                                }

                                commandQueue.Add("read");

                                string encoded = commandQueue.Count.ToString();
                                byte[] encodedBytes = Encoding.UTF8.GetBytes(encoded);

                                if (!WriteFile(pipe, encodedBytes, (uint)encodedBytes.Length, out _, IntPtr.Zero))
                                {
                                    return;
                                }


                                foreach (var cmd in commandQueue)
                                {
                                    byte[] cmdBytes = Encoding.UTF8.GetBytes(cmd);
                                    WriteFile(pipe, cmdBytes, (uint)cmdBytes.Length, out _, IntPtr.Zero);
                                    ReadFile(pipe, null, 0, out _, IntPtr.Zero);

                                    uint size = 0;
                                    if (PeekNamedPipe(pipe, null, 0, IntPtr.Zero, out size, IntPtr.Zero))
                                    {
                                        byte[] tempBuffer = new byte[size];
                                        ReadFile(pipe, tempBuffer, size, out _, IntPtr.Zero);
                                        string tempStr = Encoding.UTF8.GetString(tempBuffer);

                                        if (ulong.TryParse(tempStr, out ulong numResponses))
                                        {
                                            for (int i = 0; i < (int)numResponses; i++)
                                            {
                                                uint dataSize = 0;

                                                ReadFile(pipe, null, 0, out _, IntPtr.Zero);
                                                if (!PeekNamedPipe(pipe, null, 0, IntPtr.Zero, out dataSize, IntPtr.Zero))
                                                    break;


                                                byte[] dataBuffer = new byte[dataSize];
                                                ReadFile(pipe, dataBuffer, dataSize, out _, IntPtr.Zero);
                                                string data = Encoding.UTF8.GetString(dataBuffer);

                                                lock (_cycleLock)
                                                {
                                                    foreach (var callback in _onMessageCallbacks)
                                                    {
                                                        callback(cmd, data, i);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                Thread.Sleep(5);
                            }
                        }
                    }
                } finally
                {
                    SynapseZAPI2.RemoveSession(this.Pid);
                }
            }
        }

        public static void StartInstancesTimer()
        {
            if (Timer != null) return;

            Timer = new System.Timers.Timer(2000);
            Timer.Elapsed += InstancesTimerTick;
            Timer.AutoReset = true;
            Timer.Enabled = true;
        }

        public static void StopInstancesTimer()
        {
            if (Timer != null) return;

            Timer.Enabled = false;
        }

        private static void InstancesTimerTick(Object source, ElapsedEventArgs e)
        {
            Process[] processes = Process.GetProcessesByName("RobloxPlayerBeta");

            for (int i = 0; i < processes.Length; i++) { 
                Process process = processes[i];
                if (process == null) continue;

                if (Sessions.ContainsKey((uint)process.Id)) continue;

                if (!SynapseZAPI.IsSynz(process.Id)) continue;

                // Add new instance

                SynapseSession session = new SynapseSession();

                Sessions.TryAdd((uint) process.Id, session);

                session.Init((uint) process.Id);

                SessionAdded?.Invoke(session);
            }
        }

        public static void Execute(string source, uint pid = 0)
        {
            if (pid == 0)
            {
                foreach (KeyValuePair<uint, SynapseSession> pair in Sessions)
                {
                    pair.Value.Execute(source);
                }
            } else
            {
                SynapseSession session = Sessions[pid];
                if (session == null) return;

                session.Execute(source);
            }
        }

        public static ConcurrentDictionary<uint, SynapseSession> GetInstances()
        {
            return Sessions;
        }

        internal static void RemoveSession(uint pid)
        {
            if (Sessions.TryRemove(pid, out SynapseSession session))
            {
                SessionRemoved?.Invoke(session);
            }
        }
    }
}