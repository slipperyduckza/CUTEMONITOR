using System;
using System.Collections.Generic;
using LibreHardwareMonitor.Hardware;
using System.Diagnostics;
using System.Threading;
using Newtonsoft.Json;
using System.Runtime.InteropServices;

class HardwareData
{
    public string MotherboardModel { get; set; }
    public float CpuTemp { get; set; }
    public List<float?> CcdTemperatures { get; set; } = new List<float?>();
    public float? CpuVoltage { get; set; }
    public float? CpuPower { get; set; }
    public float? ChipsetTemp { get; set; }
    public float MemoryUsage { get; set; }
    public float? MemoryTemp { get; set; }
    public int TotalMemoryMB { get; set; }
    public int MemorySpeedMTS { get; set; }
}

partial class Program
{
    [DllImport("kernel32.dll")]
    private static extern IntPtr CreateToolhelp32Snapshot(uint dwFlags, uint th32ProcessID);

    [DllImport("kernel32.dll")]
    private static extern bool Process32First(IntPtr hSnapshot, ref PROCESSENTRY32 lppe);

    [DllImport("kernel32.dll")]
    private static extern bool Process32Next(IntPtr hSnapshot, ref PROCESSENTRY32 lppe);

    [DllImport("kernel32.dll")]
    private static extern bool CloseHandle(IntPtr hObject);

    [DllImport("kernel32.dll")]
    private static extern IntPtr OpenProcess(uint dwDesiredAccess, bool bInheritHandle, uint dwProcessId);

    [DllImport("kernel32.dll")]
    private static extern bool GetExitCodeProcess(IntPtr hProcess, out uint lpExitCode);

    [StructLayout(LayoutKind.Sequential)]
    public struct PROCESSENTRY32
    {
        public uint dwSize;
        public uint cntUsage;
        public uint th32ProcessID;
        public IntPtr th32DefaultHeapID;
        public uint th32ModuleID;
        public uint cntThreads;
        public uint th32ParentProcessID;
        public int pcPriClassBase;
        public uint dwFlags;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 260)]
        public string szExeFile;
    }

    const uint TH32CS_SNAPPROCESS = 0x00000002;
    const uint PROCESS_QUERY_INFORMATION = 0x0400;
    const uint STILL_ACTIVE = 259;

    static uint GetParentProcessId(uint pid)
    {
        IntPtr snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if (snapshot == IntPtr.Zero) return 0;
        PROCESSENTRY32 entry = new PROCESSENTRY32();
        entry.dwSize = (uint)Marshal.SizeOf(typeof(PROCESSENTRY32));
        if (Process32First(snapshot, ref entry))
        {
            do
            {
                if (entry.th32ProcessID == pid)
                {
                    CloseHandle(snapshot);
                    return entry.th32ParentProcessID;
                }
            } while (Process32Next(snapshot, ref entry));
        }
        CloseHandle(snapshot);
        return 0;
    }

    static bool IsProcessAlive(uint pid)
    {
        IntPtr handle = OpenProcess(PROCESS_QUERY_INFORMATION, false, pid);
        if (handle == IntPtr.Zero) return false;
        bool result = GetExitCodeProcess(handle, out uint exitCode);
        CloseHandle(handle);
        return result && exitCode == STILL_ACTIVE;
    }

    static void Main()
    {
        uint myPid = (uint)Process.GetCurrentProcess().Id;
        uint parentPid = GetParentProcessId(myPid);
        uint grandParentPid = GetParentProcessId(parentPid);
        Computer computer = new Computer
        {
            IsCpuEnabled = true,
            IsMemoryEnabled = true,
            IsMotherboardEnabled = true
        };

        try
        {
            computer.Open();
        }
        catch
        {
            // If motherboard fails, disable it and try again
            computer = new Computer
            {
                IsCpuEnabled = true,
                IsMemoryEnabled = true,
                IsMotherboardEnabled = false
            };
            computer.Open();
        }

        string motherboardModel = "Unknown";
        if (computer.SMBios.Board != null)
        {
            motherboardModel = (computer.SMBios.Board.ManufacturerName ?? "") + " " + (computer.SMBios.Board.ProductName ?? "");
            motherboardModel = motherboardModel.Trim();
            if (string.IsNullOrEmpty(motherboardModel))
            {
                motherboardModel = "Unknown";
            }
        }

        int totalMemoryMB = 0;
        int maxMemorySpeed = 0;
        foreach (var memDevice in computer.SMBios.MemoryDevices)
        {
            totalMemoryMB += (int)memDevice.Size;
            if (memDevice.Speed > maxMemorySpeed)
            {
                maxMemorySpeed = (int)memDevice.Speed;
            }
        }

        while (true)
        {
            if (!IsProcessAlive(parentPid)) {
                break;
            }
            var data = new HardwareData();
            data.MotherboardModel = motherboardModel;
            data.TotalMemoryMB = totalMemoryMB;
            data.MemorySpeedMTS = maxMemorySpeed;

            foreach (IHardware hardware in computer.Hardware)
            {
                hardware.Update();
                if (hardware.HardwareType == HardwareType.Cpu)
                {
                    foreach (ISensor sensor in hardware.Sensors)
                    {
                        if (sensor.SensorType == SensorType.Temperature)
                        {
                            data.CpuTemp = sensor.Value ?? 0;
                        }
                        else if (sensor.SensorType == SensorType.Temperature && sensor.Name.StartsWith("CCD") && sensor.Name.EndsWith("(Tdie)"))
                        {
                            data.CcdTemperatures.Add(sensor.Value);
                        }
                        else if (sensor.SensorType == SensorType.Voltage && sensor.Name == "Core (SVI2 TFN)")
                        {
                            data.CpuVoltage = sensor.Value;
                        }
                        else if (sensor.SensorType == SensorType.Power && sensor.Name == "Package")
                        {
                            data.CpuPower = sensor.Value;
                        }
                    }
                }
                else if (hardware.HardwareType == HardwareType.Memory)
                {
                    foreach (ISensor sensor in hardware.Sensors)
                    {
                        if (sensor.SensorType == SensorType.Load)
                        {
                            data.MemoryUsage = sensor.Value ?? 0;
                        }
                        else if (sensor.SensorType == SensorType.Temperature)
                        {
                            data.MemoryTemp = sensor.Value;
                        }
                    }
                    foreach (IHardware subHardware in hardware.SubHardware)
                    {
                        subHardware.Update();
                        foreach (ISensor sensor in subHardware.Sensors)
                        {
                            if (sensor.SensorType == SensorType.Temperature && data.MemoryTemp == null)
                            {
                                data.MemoryTemp = sensor.Value;
                            }
                        }
                    }
                }
                else if (hardware.HardwareType == HardwareType.Motherboard)
                {
                    foreach (ISensor sensor in hardware.Sensors)
                    {
                        if (sensor.SensorType == SensorType.Temperature && data.ChipsetTemp == null)
                        {
                            data.ChipsetTemp = sensor.Value;
                        }
                    }
                    foreach (IHardware subHardware in hardware.SubHardware)
                    {
                        subHardware.Update();
                        foreach (ISensor sensor in subHardware.Sensors)
                        {
                            if (sensor.SensorType == SensorType.Temperature && data.ChipsetTemp == null)
                            {
                                data.ChipsetTemp = sensor.Value;
                            }
                        }
                    }
                }
            }

            try
            {
                Console.WriteLine(JsonConvert.SerializeObject(data));
                Console.Out.Flush();
            }
            catch
            {
                break;
            }
            Thread.Sleep(500);
        }

        // computer.Close(); // never reached
    }
}