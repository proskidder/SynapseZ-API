using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Net.Http;

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
                Process process = processes[i];
                string path = process.MainModule.FileName;

                FileStream stream = File.Open(path, FileMode.Open, FileAccess.Read, FileShare.ReadWrite);
                byte[] array = new byte[0x600];
                stream.BeginRead(array, 0, 0x600, null, null); // Read the first 600 bytes -> thats where the .grh section should be mentioned!
                stream.Close();

                string fileContent = System.Text.Encoding.Default.GetString(array);
                if (fileContent.Contains(".grh"))
                {
                    injectedProcesses.Add(process);
                }
            }

            return injectedProcesses;
        }

        /**
            * Return values:
            * bool - If the Instance is a SynZ Instance
        */
        public static bool IsSynz(int PID = 0)
        {
            List<Process> injectedProcesses = GetSynzRobloxInstances();

            if (PID != 0)
            {
                return injectedProcesses.Exists((process) => process.Id == PID);
            }
            else
            {
                return injectedProcesses.Count != 0;
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
}
