using System;
using System.Diagnostics;
using System.IO;

class Program
{
    static void Main()
    {
        Console.Title = "OmniFormatter Publisher";
        
        // Find the powershell script relative to the exe
        string exeDir = AppDomain.CurrentDomain.BaseDirectory;
        string scriptPath = Path.Combine(exeDir, "scripts", "publish-extension.ps1");
        
        if (!File.Exists(scriptPath))
        {
            // Try parent directory in case it is placed inside scripts
            scriptPath = Path.Combine(exeDir, "..", "scripts", "publish-extension.ps1");
            if (!File.Exists(scriptPath))
            {
                scriptPath = Path.Combine(exeDir, "publish-extension.ps1");
            }
        }
        
        if (!File.Exists(scriptPath))
        {
            Console.ForegroundColor = ConsoleColor.Red;
            Console.WriteLine("Error: Could not find 'publish-extension.ps1'.");
            Console.WriteLine("Please ensure 'publish-extension.ps1' is inside a 'scripts' folder next to this executable.");
            Console.ResetColor();
            Console.WriteLine("\nPress any key to exit...");
            Console.ReadKey();
            return;
        }

        Console.ForegroundColor = ConsoleColor.Cyan;
        Console.WriteLine("Launching OmniFormatter Release Workflow...");
        Console.ResetColor();
        Console.WriteLine();

        ProcessStartInfo startInfo = new ProcessStartInfo();
        startInfo.FileName = "powershell.exe";
        // Run with bypass execution policy so it works even if script execution is restricted
        startInfo.Arguments = "-NoProfile -ExecutionPolicy Bypass -File \"" + scriptPath + "\"";
        startInfo.UseShellExecute = false;
        
        int exitCode = -1;
        try
        {
            using (Process process = Process.Start(startInfo))
            {
                process.WaitForExit();
                exitCode = process.ExitCode;
            }
        }
        catch (Exception ex)
        {
            Console.ForegroundColor = ConsoleColor.Red;
            Console.WriteLine("An error occurred while launching PowerShell: " + ex.Message);
            Console.ResetColor();
        }
        
        if (exitCode != 0)
        {
            Console.ForegroundColor = ConsoleColor.Red;
            Console.WriteLine("\nWorkflow failed or was aborted. Press any key to close this window...");
            Console.ResetColor();
            Console.ReadKey();
        }
        else
        {
            Console.ForegroundColor = ConsoleColor.Green;
            Console.WriteLine("\nWorkflow completed successfully! Closing in 3 seconds...");
            Console.ResetColor();
            System.Threading.Thread.Sleep(3000);
        }
    }
}
