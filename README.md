# Cute Monitor

Cute Monitor is a simple Windows Monitoring app. Written in Rust using VSCodium and assisted with opencode AI. Temperature data thanks to LibreHardwareMonitor project link: https://github.com/LibreHardwareMonitor/LibreHardwareMonitor making use of LibreHardwareMonitorLib.dll with a sub-routine using .NET via Rust 'tokio' crate command to poll sensor data. I'm working on adding GPU monitoring in my next version.

## Features

- CPU usage monitoring with current usage indicators
- CPU temperature display with color-coded gradients
- Administrator privilege handling for hardware access (temperature sensor data)
- Added Evaluated (over 10 seconds) Top 3 user processes
- Added GPU processor and memory usage and GPU Temperature Monitor
- Clean, responsive GUI built with Iced

## Recent Changes (v0.1.4)

- Detect missing .NET 8 runtime and prompt to winget install official .NET 8 runtime
- Added virtual machine detection (Hyper-V, QEMU, KVM) with appropriate fallbacks for hardware monitoring
- Changed application launch to require admin privileges for CPU temperature monitoring rather than a relaunch button due to inconsistency problems
- Application icon has been integrated

## Requirements

- Windows 10/11
- .NET 8 Desktop Runtime (for temperature monitoring; auto-install prompt if missing)

## Building

```bash
cargo build --release
```

## Running

Run the executable from `target/release/cutemonitor.exe`. Administrator privileges may be required for temperature monitoring.

## License

[Add license here]
