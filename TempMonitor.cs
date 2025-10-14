using LibreHardwareMonitor.Hardware;
using System.Diagnostics;

partial class Program
{
    static void Main()
    {
        var process = Process.Start(new ProcessStartInfo
        {
            FileName = "dotnet",
            Arguments = "--list-runtimes",
            RedirectStandardOutput = true,
            UseShellExecute = false
        });
        if (process != null)
        {
            process.WaitForExit();
            string output = process.StandardOutput.ReadToEnd();
            if (!output.Contains("Microsoft.NETCore.App 8."))
            {
                var installProcess = Process.Start(new ProcessStartInfo
                {
                    FileName = "winget",
                    Arguments = "install dotnet-runtime-8",
                    UseShellExecute = true
                });
                if (installProcess != null)
                {
                    installProcess.WaitForExit();
                }
            }
        }

        Computer computer = new Computer
        {
            IsCpuEnabled = true
        };

        computer.Open();

        foreach (IHardware hardware in computer.Hardware)
        {
            if (hardware.HardwareType == HardwareType.Cpu)
            {
                hardware.Update();
                foreach (ISensor sensor in hardware.Sensors)
                {
                    if (sensor.SensorType == SensorType.Temperature)
                    {
                        Console.WriteLine($"{sensor.Name}: {sensor.Value}°C");
                    }
                }
            }
        }

        computer.Close();
    }
}