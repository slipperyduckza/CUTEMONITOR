# CuteMonitor

A lightweight hardware monitoring application built with Rust and Iced GUI framework. Provides real-time monitoring of CPU, GPU, and system temperatures with visual charts and progress bars.

<img width="800" height="793" alt="CUTEMONITOR" src="https://github.com/user-attachments/assets/fddc9a3d-716a-48ad-9409-b91cf15ff0a4" />

## Features

- Real-time CPU and GPU monitoring
- Temperature visualization with color-coded data
- Historical data charts (30 data points)
- Manufacturer logos for CPU/GPU identification
- Network Bandwidth autoscale graph and upload/download data
- Requires administrator privileges for hardware access

## Requirements

- Windows 10/11
- Administrator privileges
- Rust toolchain (for building from source)

## Installation

[Download Latest Release](https://github.com/slipperyduckza/CUTEMONITOR/releases/latest)


Alternatively you can build this yourself, see below:

1. Clone the repository:
   ```bash
   git clone https://github.com/slipperyduckza/CUTEMONITOR.git
   cd CUTEMONITOR
   ```

2. Build the application:
   ```bash
   cargo build --release
   ```

3. Run as administrator:
   ```bash
   .\target\release\cutemonitor.exe
   ```

## Usage

- Launch the application with administrator privileges
- View real-time hardware metrics in the GUI
- Monitor CPU usage, temperatures, and GPU stats
- Charts update automatically with new data

## Dependencies

- [Iced](https://github.com/iced-rs/iced) - GUI framework
- [Sysinfo](https://github.com/GuillaumeGomez/sysinfo) - System information
- [LibreHardwareMonitor](https://github.com/LibreHardwareMonitor/LibreHardwareMonitor) - Hardware monitoring library

## License

CuteMonitor is free and open souce software licensed under MPL 2.0, CPU temperature and Montherboard model information is attained using LibreHardwareMonitor.dll 
LibreHardwareMonitor is free and open source software licensed under MPL 2.0. Some parts of LibreHardwareMonitor are licensed under different terms, see [THIRD-PARTY-LICENSES](https://github.com/LibreHardwareMonitor/LibreHardwareMonitor/blob/master/THIRD-PARTY-NOTICES.txt).

## Contributing

Feel free to fork or contact if you would like to contribute.
