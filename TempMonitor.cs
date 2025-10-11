using LibreHardwareMonitor.Hardware;

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