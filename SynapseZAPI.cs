using System;
using System.Diagnostics;
using System.IO;
using System.Linq;

namespace SynapseZAPI
{
    public class SynapseZAPI
    {
        private string LatestErrorMsg = "";

        /**
         * Return values:
         * not-empty string -> Path to the launcher
         * empty string -> Couldn't find the launcher
        */
        public string FindLauncher(string FolderPath)
        {
            foreach (var file in Directory.GetFiles(FolderPath))
            {
                string content = File.ReadAllText(file);
                if (content.Contains(".grh0") && content.Contains(".grh1") && content.Contains(".grh2"))
                {
                    return file;
                }
            }

            return String.Empty;
        }

        /**
         * Return values:
         * 0 - Injection successful
         * 1 - MainPath is not a valid Directory
         * 2 - launcher not found
         * 3 - Couldn't start the launcher
        */
        public int Inject(string MainPath)
        {
            if (!Directory.Exists(MainPath))
            {
                LatestErrorMsg = "MainPath doesnt lead to a directory.";
                return 1;
            }

            string LauncherPath = FindLauncher(MainPath);

            if (LauncherPath == String.Empty)
            {
                LatestErrorMsg = "Could not find the Launcher!";
                return 2;
            }

            Process process = new Process();
            process.StartInfo.FileName = LauncherPath;

            try
            {
                process.Start();
                return 0;
            }
            catch (Exception e)
            {
                LatestErrorMsg = e.Message;
                return 3;
            }
        }

        /**
         * Returns the latest error message from any action.
        */
        public string GetLatestErrorMessage()
        {
            return LatestErrorMsg;
        }

        /**
         * Return values:
         * 0 - Execution successful
         * 1 - MainPath is not a valid Directory
         * 2 - Bin Folder not found
         * 3 - Scheduler Folder not found
         * 4 - No access to write file
        */
        public int Execute(string MainPath, string Script, int PID = 0)
        {
            if (!Directory.Exists(MainPath))
            {
                LatestErrorMsg = "MainPath doesnt lead to a directory.";
                return 1;
            }

            string BinPath = Path.Combine(MainPath, "bin");

            if (!Directory.Exists(BinPath))
            {
                LatestErrorMsg = "Could not find the Bin Folder!";
                return 2;
            }

            string SchedulerPath = Path.Combine(BinPath, "scheduler");

            if (!Directory.Exists(SchedulerPath))
            {
                LatestErrorMsg = "Could not find the Scheduler Folder!";
                return 3;
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
                return 4;
            }

            return 0;
        }

        private static Random random = new Random();
        private static string RandomString(int length)
        {
            const string chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
            return new string(Enumerable.Repeat(chars, length)
                .Select(s => s[random.Next(s.Length)]).ToArray());
        }
    }
}
