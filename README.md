# Cute Monitor

Cute Monitor is a simple Windows Monitoring app. Written in Rust using VSCodium and assisted with opencode AI. Temperature data thanks to LibreHardwareMonitor project link: https://github.com/LibreHardwareMonitor/LibreHardwareMonitor making use of LibreHardwareMonitorLib.dll with a sub-routine using .NET via Rust 'tokio' crate command to poll sensor data. I'm working on adding GPU monitoring in my next version.

## Features

- CPU usage monitoring with current usage indicators
- CPU temperature display with color-coded gradients
- Administrator privilege handling for hardware access (temperature sensor data)
- Clean, responsive GUI built with Iced

## Requirements

- Windows 10/11
- .NET runtime (for temperature monitoring)

## Building

```bash
cargo build --release
```

## Running

Run the executable from `target/release/cutemonitor.exe`. Administrator privileges may be required for temperature monitoring.

## License

[Add license here]