# CuteMonitor

A lightweight hardware monitoring application built with Rust and Iced GUI framework. Provides real-time monitoring of CPU, GPU, and system temperatures with visual charts and progress bars.

## Features

- Real-time CPU and GPU monitoring
- Temperature visualization with color-coded data
- Historical data charts (30 data points)
- Manufacturer logos for CPU/GPU identification
- Windows system tray integration
- Requires administrator privileges for hardware access

## Requirements

- Windows 10/11
- Administrator privileges
- Rust toolchain (for building from source)

## Installation

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
- LibreHardwareMonitor - Hardware monitoring library

## License

[Add license information here]

## Contributing

[Add contribution guidelines here]