// Import necessary modules from the iced GUI library for building the app
use iced::{
    alignment, border, event, time, Application, Background, Border, Color, Command, Element,
    Length, Settings, Subscription, Theme,
};
// Import GUI widgets: Container for styling, Canvas for custom drawing
use iced::widget::{button, canvas, container, tooltip, tooltip::Position, Canvas, Image};
use iced::advanced::image::Handle;
// Import sysinfo to get system information like CPU usage
use sysinfo::{System, ProcessesToUpdate};
// Import HashMap for process history
use std::collections::HashMap;
// Import gfxinfo for GPU information
use gfxinfo::active_gpu;
// Import machine-info for GPU usage
use machine_info::Machine;
// Import Windows API functions for reading registry (to detect dark/light theme)
use windows::Win32::System::Registry::{
    RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY_CURRENT_USER, KEY_READ,
};
// Import for ShellExecute to run as admin
use windows::Win32::UI::Shell::{ShellExecuteW, IsUserAnAdmin};
// Import for hiding console window
use windows::Win32::System::Threading::CREATE_NO_WINDOW;

// Import for subscription recipe
use iced::advanced::subscription::Recipe;
// Import futures for streams
use iced::futures::stream::{self, BoxStream};
// Import for async process
use tokio::process::Command as TokioCommand;
// Embedded files
static TEMP_CS: &[u8] = include_bytes!("../TempMonitor.cs");
static TEMP_CSPROJ: &[u8] = include_bytes!("../TempMonitor.csproj");
static TEMP_DLL: &[u8] = include_bytes!("../LibreHardwareMonitorLib.dll");
// Embedded logos
static AMD_LOGO: &[u8] = include_bytes!("../AMD256.png");
static INTEL_LOGO: &[u8] = include_bytes!("../INTEL256.png");
// Embedded GPU logos
static NVIDIA_LOGO: &[u8] = include_bytes!("../Nvidia_GeForce_256.png");
static AMD_GPU_LOGO: &[u8] = include_bytes!("../AMD_Radeon_256.png");
static INTEL_GPU_LOGO: &[u8] = include_bytes!("../Intel_Arc_256.png");

// Constants for easy configuration
const HISTORY_SIZE: usize = 30; // How many past CPU readings to keep
const BAR_HEIGHT: f32 = 20.0; // Height of each progress bar in pixels
const SPACING: f32 = 20.0; // Spacing between UI elements

// Function to create a black border style for containers around the bars
fn black_border(_theme: &Theme) -> container::Appearance {
    container::Appearance {
        background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.0))), // Transparent background
        border: Border {
            width: 1.0,
            color: Color::BLACK,
            radius: border::Radius::default(),
        }, // 1px black border
        ..Default::default()
    }
}

// Function to create a black filled box with rounded corners for the graphs section
fn black_filled_box(_theme: &Theme) -> container::Appearance {
    container::Appearance {
        background: Some(Background::Color(Color::BLACK)), // Black background
        border: Border {
            width: 0.0,
            color: Color::BLACK,
            radius: border::Radius::from(10.0),
        }, // Rounded corners
        ..Default::default()
    }
}

// Function to create a black background style for tooltips
fn black_tooltip(_theme: &Theme) -> container::Appearance {
    container::Appearance {
        background: Some(Background::Color(Color::BLACK)),
        border: Border { width: 1.0, color: Color::WHITE, radius: border::Radius::from(5.0) },
        ..Default::default()
    }
}

// Function to create a dark grey box with rounded corners
fn dark_grey_box(_theme: &Theme) -> container::Appearance {
    container::Appearance {
        background: Some(Background::Color(Color::from_rgb(0.3, 0.3, 0.3))), // Dark grey
        border: Border { width: 0.0, color: Color::BLACK, radius: border::Radius::from(10.0) }, // Rounded corners
        ..Default::default()
    }
}

// Function to create a grey box with color #242323
fn grey_242323_box(_theme: &Theme) -> container::Appearance {
    container::Appearance {
        background: Some(Background::Color(Color::from_rgb(36.0 / 255.0, 35.0 / 255.0, 35.0 / 255.0))), // #242323
        border: Border { width: 0.0, color: Color::BLACK, radius: border::Radius::from(10.0) }, // Rounded corners
        ..Default::default()
    }
}

// Function to create a light grey horizontal line
fn light_grey_line(_theme: &Theme) -> container::Appearance {
    container::Appearance {
        background: Some(Background::Color(Color::from_rgb(0.75, 0.75, 0.75))), // Light grey
        ..Default::default()
    }
}

// Function to get color based on temperature
fn temp_color(temp: f32) -> Color {
    if temp <= 10.0 {
        Color::from_rgb(1.0, 1.0, 0.0) // Yellow
    } else if temp <= 45.0 {
        let t = (temp - 10.0) / (45.0 - 10.0);
        let g = 1.0 - (1.0 - 165.0 / 255.0) * t;
        Color::from_rgb(1.0, g, 0.0)
    } else if temp <= 88.0 {
        let t = (temp - 45.0) / (88.0 - 45.0);
        let g = (165.0 / 255.0) * (1.0 - t);
        Color::from_rgb(1.0, g, 0.0)
    } else {
        Color::from_rgb(1.0, 0.0, 0.0) // Red
    }
}

// Function to parse temperature from string
fn parse_temp(s: &str) -> Option<f32> {
    use regex::Regex;
    let re = Regex::new(r"(\d+(?:[.,]\d+)?)°C").unwrap();
    let num_str = re.captures(s)?.get(1)?.as_str().replace(',', ".");
    num_str.parse().ok()
}

// Function to check if running as admin
fn is_admin() -> bool {
    unsafe { IsUserAnAdmin().as_bool() }
}

// Struct to hold the CPU usage data for drawing the overlaid bars
#[derive(Debug)]
struct OverlayBarProgram {
    current: f32,  // Current CPU usage percentage
    previous: f32, // Previous CPU usage percentage
    oldest: f32,   // Oldest CPU usage percentage in history
}

// Struct to hold the CPU usage data for drawing the bar chart
#[derive(Debug)]
struct BarChartProgram {
    history: Vec<f32>, // Historical CPU usage percentages
}

// Implement the canvas drawing program for the overlaid bars
impl<Message> canvas::Program<Message> for OverlayBarProgram {
    type State = (); // No state needed for this simple drawing

    // This function is called to draw the bars on the canvas
    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        // Create a drawing frame with the size of the canvas area
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        // Draw bars in order: oldest (dark), previous (grey), current (blue)
        // Oldest at bottom with height -15, previous with height -8, current with height -1
        if self.oldest > 0.0 {
            let width = bounds.width * self.oldest / 100.0;
            let height = bounds.height - 15.0;
            let y = bounds.height - height;
            // Draw a dark rectangle for the oldest usage
            frame.fill_rectangle(
                iced::Point::new(0.0, y),
                iced::Size::new(width, height),
                Color::from_rgba(0.1, 0.1, 0.3, 0.8),
            );
        }

        if self.previous > 0.0 {
            let width = bounds.width * self.previous / 100.0;
            let height = bounds.height - 8.0;
            let y = bounds.height - height;
            // Draw a grey rectangle for the previous usage
            frame.fill_rectangle(
                iced::Point::new(0.0, y),
                iced::Size::new(width, height),
                Color::from_rgba(0.3, 0.3, 0.6, 0.65),
            );
        }

        let current_width = bounds.width * self.current / 100.0;
        let height = bounds.height - 1.0;
        let y = bounds.height - height;
        // Draw a blue rectangle for the current usage
        frame.fill_rectangle(
            iced::Point::new(0.0, y),
            iced::Size::new(current_width, height),
            Color::from_rgba(0.1, 0.1, 1.0, 1.0),
        );

        // Return the drawn frame as geometry for rendering
        vec![frame.into_geometry()]
    }
}

// Implement the canvas drawing program for the bar chart
impl<Message> canvas::Program<Message> for BarChartProgram {
    type State = (); // No state needed for this simple drawing

    // This function is called to draw the bars on the canvas
    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        // Create a drawing frame with the size of the canvas area
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        let bar_width = 0.4;
        let spacing = 0.5;
        let total_width_needed = self.history.len() as f32 * spacing;
        let scale_x = bounds.width / total_width_needed;

        for (i, &usage) in self.history.iter().enumerate() {
            let x = i as f32 * spacing * scale_x;
            let bar_height = (usage / 100.0) * bounds.height;
            let y = bounds.height - bar_height;

            // Draw bar with color similar to PROTOTYPE
            frame.fill_rectangle(
                iced::Point::new(x, y),
                iced::Size::new(bar_width * scale_x, bar_height),
                Color::from_rgb(123.0 / 255.0, 104.0 / 255.0, 238.0 / 255.0), // Medium slate blue
            );

            // Draw stroke
            frame.stroke(
                &canvas::Path::rectangle(
                    iced::Point::new(x, y),
                    iced::Size::new(bar_width * scale_x, bar_height),
                ),
                canvas::Stroke::default()
                    .with_color(Color::from_rgb(25.0 / 255.0, 25.0 / 255.0, 112.0 / 255.0))
                    .with_width(0.5),
            );
        }

        // Return the drawn frame as geometry for rendering
        vec![frame.into_geometry()]
    }
}

// Function to get temperatures by running the .NET app
async fn get_temperatures() -> Vec<String> {
    // Create a temp directory
    let temp_dir = std::env::temp_dir().join("cutemonitor_temp");
    if tokio::fs::create_dir_all(&temp_dir).await.is_err() {
        return vec!["Error creating temp dir".to_string()];
    }

    // Write the files
    let cs_path = temp_dir.join("TempMonitor.cs");
    let csproj_path = temp_dir.join("TempMonitor.csproj");
    let dll_path = temp_dir.join("LibreHardwareMonitorLib.dll");

    if tokio::fs::write(&cs_path, TEMP_CS).await.is_err() {
        return vec!["Error writing cs".to_string()];
    }
    if tokio::fs::write(&csproj_path, TEMP_CSPROJ).await.is_err() {
        return vec!["Error writing csproj".to_string()];
    }
    if tokio::fs::write(&dll_path, TEMP_DLL).await.is_err() {
        return vec!["Error writing dll".to_string()];
    }

    // Run the command
    let output = TokioCommand::new("dotnet")
        .arg("run")
        .arg("--project")
        .arg(&csproj_path)
        .creation_flags(CREATE_NO_WINDOW.0)
        .output()
        .await;

    // Clean up temp files
    let _ = tokio::fs::remove_dir_all(&temp_dir).await;

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.lines().map(|line| line.to_string()).collect()
        }
        _ => vec!["Error getting temperatures".to_string()],
    }
}

// Messages that the app can receive to update its state
#[derive(Debug, Clone)]
enum Message {
    UpdateCores(Vec<f32>),            // Contains new CPU core usages
    UpdateThreads(Vec<f32>),          // Contains new CPU thread usages
    UpdateTopProcesses(Vec<String>),  // Contains top 3 process names by CPU usage
    TimeUpdate(String),               // New time string
    TempUpdate(Vec<String>),          // New temperature data
    GfxStatusUpdate(String),          // New GFX status data
    RequestAdmin,                     // Request to run as admin
}

// Recipe for CPU cores monitoring subscription
struct CpuCoresMonitor;

impl Recipe for CpuCoresMonitor {
    type Output = Message;

    fn hash(&self, state: &mut iced::advanced::Hasher) {
        use std::hash::Hash;
        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: BoxStream<'static, (event::Event, event::Status)>,
    ) -> BoxStream<'static, Self::Output> {
        let stream = stream::unfold((), |()| async {
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            let mut sys = System::new_all();
            sys.refresh_cpu_all();
            let cores = System::physical_core_count().unwrap_or(1);
            let core_usages: Vec<f32> = (0..cores).map(|i| sys.cpus()[i].cpu_usage()).collect();
            Some((Message::UpdateCores(core_usages), ()))
        });
        Box::pin(stream)
    }
}

// Recipe for CPU threads monitoring subscription
struct CpuThreadsMonitor;

impl Recipe for CpuThreadsMonitor {
    type Output = Message;

    fn hash(&self, state: &mut iced::advanced::Hasher) {
        use std::hash::Hash;
        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: BoxStream<'static, (event::Event, event::Status)>,
    ) -> BoxStream<'static, Self::Output> {
        let stream = stream::unfold((), |()| async {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            let mut sys = System::new_all();
            sys.refresh_cpu_all();
            let thread_usages: Vec<f32> = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();
            Some((Message::UpdateThreads(thread_usages), ()))
        });
        Box::pin(stream)
    }
}

// Recipe for temperature monitoring subscription
struct TempMonitor;

impl Recipe for TempMonitor {
    type Output = Message;

    fn hash(&self, state: &mut iced::advanced::Hasher) {
        use std::hash::Hash;
        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: BoxStream<'static, (event::Event, event::Status)>,
    ) -> BoxStream<'static, Self::Output> {
        let stream = stream::unfold((), |()| async {
            tokio::time::sleep(std::time::Duration::from_secs(3)).await; // Update every 3 seconds
            let temps = get_temperatures().await;
            Some((Message::TempUpdate(temps), ()))
        });
        Box::pin(stream)
    }
}

// Recipe for GFX status monitoring subscription
struct GfxMonitor;

impl Recipe for GfxMonitor {
    type Output = Message;

    fn hash(&self, state: &mut iced::advanced::Hasher) {
        use std::hash::Hash;
        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: BoxStream<'static, (event::Event, event::Status)>,
    ) -> BoxStream<'static, Self::Output> {
        let stream = stream::unfold((), |()| async {
            tokio::time::sleep(std::time::Duration::from_millis(1100)).await; // Update every 1.1 seconds
            let machine = Machine::new();
            let graphics = machine.graphics_status();
            let status = if let Some(usage) = graphics.first() {
                format!(
                    "GPU Utilization: {}%\nGPU Memory usage: {} MB\nTemperature: {}°C",
                    usage.gpu,
                    usage.memory_used / 1024 / 1024,
                    usage.temperature
                )
            } else {
                "No GPU detected".to_string()
            };
            Some((Message::GfxStatusUpdate(status), ()))
        });
        Box::pin(stream)
    }
}

// Recipe for top processes monitoring subscription
#[derive(Clone)]
enum Phase {
    Initial,
    Collecting,
}

struct TopProcessesMonitor {
    history: HashMap<String, Vec<f32>>,
    update_count: usize,
    phase: Phase,
    initial_top: Vec<String>,
    last_averaged: Option<Vec<String>>,
}

impl Default for TopProcessesMonitor {
    fn default() -> Self {
        Self {
            history: HashMap::new(),
            update_count: 0,
            phase: Phase::Initial,
            initial_top: vec![],
            last_averaged: None,
        }
    }
}

impl Recipe for TopProcessesMonitor {
    type Output = Message;

    fn hash(&self, state: &mut iced::advanced::Hasher) {
        use std::hash::Hash;
        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: BoxStream<'static, (event::Event, event::Status)>,
    ) -> BoxStream<'static, Self::Output> {
        let stream = stream::unfold(self, |mut monitor| async {
            tokio::time::sleep(std::time::Duration::from_millis(200)).await; // Update every 0.2 seconds for 30 updates = 6 seconds
            let mut sys = System::new_all();
            sys.refresh_processes(ProcessesToUpdate::All, true);
            let system_processes = [
                "System", "System Idle Process", "csrss.exe", "wininit.exe", "services.exe",
                "lsass.exe", "winlogon.exe", "smss.exe", "svchost.exe", "explorer.exe",
                "dwm.exe", "taskhostw.exe", "sihost.exe", "fontdrvhost.exe", "ctfmon.exe",
                "SearchIndexer.exe", "SearchHost.exe", "RuntimeBroker.exe", "StartMenuExperienceHost.exe",
                "ShellExperienceHost.exe", "ApplicationFrameHost.exe", "TextInputHost.exe",
                "LockApp.exe", "WWAHost.exe", "MicrosoftEdge.exe", "MicrosoftEdgeCP.exe",
                "MicrosoftEdgeSH.exe", "msedge.exe", "msedgewebview2.exe", "cutemonitor.exe", "conhost.exe", "eServiceHost.exe", "eOppFrame.exe"
            ];
            let user_processes: Vec<_> = sys.processes().iter()
                .filter(|(_, p)| !system_processes.contains(&p.name().to_string_lossy().as_ref()))
                .collect();

            let mut sorted_processes = user_processes.clone();
            sorted_processes.sort_by(|a, b| b.1.cpu_usage().partial_cmp(&a.1.cpu_usage()).unwrap_or(std::cmp::Ordering::Equal));

            let top_names = match monitor.phase.clone() {
                Phase::Initial => {
                    // Show current top 3 once
                    let top = sorted_processes.iter().take(3).map(|(_, p)| p.name().to_string_lossy().to_string()).collect::<Vec<_>>();
                    monitor.initial_top = top.clone();
                    monitor.phase = Phase::Collecting;
                    top
                }
                Phase::Collecting => {
                    // Collect top 20 processes' usages
                    for (_, p) in sorted_processes.iter().take(20) {
                        let name = p.name().to_string_lossy().to_string();
                        let usage = p.cpu_usage();
                        monitor.history.entry(name).or_insert_with(Vec::new).push(usage);
                    }
                    monitor.update_count += 1;
                    if monitor.update_count >= 30 {
                        // Evaluate top 3 on average
                        let mut averages: Vec<(String, f32)> = monitor.history.iter()
                            .map(|(name, usages)| {
                                let avg = usages.iter().sum::<f32>() / usages.len() as f32;
                                (name.clone(), avg)
                            })
                            .collect();
                        averages.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                        let result = averages.into_iter().take(3).map(|(name, _)| name).collect::<Vec<_>>();
                        monitor.last_averaged = Some(result.clone());
                        // Flush data and restart
                        monitor.history.clear();
                        monitor.update_count = 0;
                        result
                    } else {
                        // Show last averaged if available, else initial top
                        monitor.last_averaged.as_ref().unwrap_or(&monitor.initial_top).clone()
                    }
                }
            };

            Some((Message::UpdateTopProcesses(top_names), monitor))
        });
        Box::pin(stream)
    }
}

// The main app struct that holds all the data and state
struct Cutemonitor {
    model: String,                // CPU model name
    cores: usize,                 // Number of CPU cores
    threads: usize,               // Number of CPU threads
    core_usages: Vec<Vec<f32>>,   // History of CPU core usages (each core has a vec of past usages)
    thread_usages: Vec<Vec<f32>>, // History of CPU thread usages
    top_processes: Vec<String>,   // Top 3 processes by CPU usage
    gpu_model: String,            // GPU model name
    gpu_vram: String,             // GPU VRAM amount
    date: String,                 // Current date string
    time: String,                 // Current time string
    temperatures: Vec<String>,    // CPU temperatures
    gfx_status: String,           // GFX status from nvoclock
}

impl Application for Cutemonitor {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    // Set the window title
    fn title(&self) -> String {
        "Cute Monitor".to_string()
    }

    // Initialize the app: get system info, set up data structures
    fn new(_flags: ()) -> (Self, Command<Message>) {
        // Create a system info object and refresh CPU data
        let mut sys = System::new_all();
        sys.refresh_cpu_all();

        // Get CPU info from the first core
        let cpu = sys.cpus().first().unwrap();
        let brand = cpu.brand();

        // Store CPU model name
        let model = brand.to_string();
        // Get number of physical cores and total threads
        let cores = System::physical_core_count().unwrap_or(1);
        let threads = sys.cpus().len();

        // Initialize usage history vectors with 10% for cores, zeros for threads
        let core_usages = vec![vec![10.0; HISTORY_SIZE]; cores];
        let thread_usages = vec![vec![0.0; HISTORY_SIZE]; threads];

        // Initialize date and time
        let now = chrono::Local::now();
        let date = format!("{} {}", now.format("%A"), now.format("%x"));
        let time = now.format("%H:%M:%S").to_string();

        // Get GPU info
        let gpu_model = match active_gpu() {
            Ok(gpu) => gpu.model().to_string(),
            Err(_) => "Unknown".to_string(),
        };
        let gpu_vram = match active_gpu() {
            Ok(gpu) => {
                let info = gpu.info();
                format!("{} MB", info.total_vram() / 1024 / 1024)
            }
            Err(_) => "Unknown".to_string(),
        };

        // Create the app instance
        let app = Cutemonitor {
            model,
            cores,
            threads,
            core_usages,
            thread_usages,
            top_processes: vec![],
            gpu_model,
            gpu_vram,
            date,
            time,
            temperatures: vec![],
            gfx_status: "".to_string(),
        };

        // Return the app and no initial command
        (app, Command::none())
    }

    // Handle messages to update the app's state
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::UpdateCores(core) => {
                // Update core usage history: add new value at the beginning, keep only HISTORY_SIZE entries
                for (i, &usage) in core.iter().enumerate() {
                    self.core_usages[i].insert(0, usage);
                    self.core_usages[i].truncate(HISTORY_SIZE);
                }
            }
            Message::UpdateThreads(thread) => {
                // Update thread usage history: add new value at the beginning, keep only HISTORY_SIZE entries
                for (i, &usage) in thread.iter().enumerate() {
                    self.thread_usages[i].insert(0, usage);
                    self.thread_usages[i].truncate(HISTORY_SIZE);
                }
            }
            Message::TimeUpdate(new_time) => {
                self.time = new_time;
            }
            Message::TempUpdate(temps) => {
                self.temperatures = temps;
            }
            Message::UpdateTopProcesses(processes) => {
                self.top_processes = processes;
            }
            Message::GfxStatusUpdate(status) => {
                self.gfx_status = status;
            }
            Message::RequestAdmin => {
                // Relaunch as admin and exit current
                if let Ok(exe_path) = std::env::current_exe() {
                    let exe_str = exe_path.to_string_lossy();
                    let exe_wide: Vec<u16> =
                        exe_str.encode_utf16().chain(std::iter::once(0)).collect();
                    unsafe {
                        let result = ShellExecuteW(
                            None,
                            windows::core::w!("runas"),
                            windows::core::PCWSTR::from_raw(exe_wide.as_ptr()),
                            None,
                            None,
                            windows::Win32::UI::WindowsAndMessaging::SHOW_WINDOW_CMD(
                                windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL.0,
                            ),
                        );
                        if result.0 > 32 {
                            // Success, exit current
                            std::process::exit(0);
                        }
                    }
                }
            }
        }
        // No command needed after update
        Command::none()
    }

    // Build the user interface layout
    fn view(&self) -> Element<'_, Message> {
        // Calculate threads per core (assuming hyperthreading)
        let threads_per_core = self.threads / self.cores;

        // Create the CPU logo image
        let is_amd = self.model.to_lowercase().contains("amd");
        let logo = Image::new(Handle::from_memory(if is_amd { AMD_LOGO } else { INTEL_LOGO }))
            .width(128)
            .height(128);

        // Create info text column with CPU details
        let info = iced::widget::Column::new()
            .push(iced::widget::text(&self.model))
            .push(iced::widget::text(format!("Cores: {}", self.cores)))
            .push(iced::widget::text(format!("Threads: {}", self.threads)));

        // Create date and time column
        let date_time = iced::widget::Column::new()
            .push(iced::widget::text(&self.date))
            .push(iced::widget::text(&self.time))
            .align_items(iced::Alignment::Start); // Align to left now

        // CPU table: image and model
        let cpu_table = iced::widget::Row::new()
            .push(logo)
            .push(info)
            .spacing(10)
            .align_items(iced::Alignment::Start); // Left align

        // Left top column: cpu_table, space, date_time
        let left_top = iced::widget::Column::new()
            .push(cpu_table)
            .push(iced::widget::Space::new(Length::Shrink, Length::Fixed(20.0)))
            .push(date_time)
            .width(Length::FillPortion(1)); // 50% width

        // Create headline text
        let headline = iced::widget::text("CPU TEMPERATURES").size(20).horizontal_alignment(alignment::Horizontal::Right);

        // Create temperature display
        let mut temp_column = iced::widget::Column::new().align_items(iced::Alignment::End);
        if !is_admin() && self.temperatures.iter().any(|t| t.contains("Error")) {
            temp_column = temp_column.push(tooltip(button("Run as Admin").on_press(Message::RequestAdmin), "Temperature Sensors Need Admin Privileges", Position::Top).style(black_tooltip));
        } else {
            for temp in &self.temperatures {
                let parsed = parse_temp(temp);
                let color = parsed.map(temp_color).unwrap_or(Color::WHITE);
                // Split into label and value
                let parts: Vec<&str> = temp.splitn(2, ':').collect();
                let label = if parts.len() > 1 { format!("{}:", parts[0]) } else { temp.clone() };
                let value = if parts.len() > 1 { parts[1].trim().to_string() } else { String::new() };
                let label_text = iced::widget::text(label).horizontal_alignment(alignment::Horizontal::Right);
                let value_text = iced::widget::text(value).horizontal_alignment(alignment::Horizontal::Right).style(color);
                let row = iced::widget::Row::new()
                    .push(label_text)
                    .push(iced::widget::Space::new(Length::Fill, Length::Shrink))
                    .push(value_text)
                    .width(Length::Fill);
                temp_column = temp_column.push(row);
            }
        }
        let temp_container = container(temp_column).style(dark_grey_box).padding(10);

        // Right top column: headline, temperatures (no top margin)
        let right_top = iced::widget::Column::new()
            .push(headline)
            .push(temp_container)
            .align_items(iced::Alignment::End) // Align to right
            .width(Length::FillPortion(1)); // 50% width

        // Top row: left_top (50%), right_top (50%)
        let top_row = iced::widget::Row::new()
            .push(left_top)
            .push(right_top)
            .spacing(SPACING)
            .width(Length::Fill);

        // Create the CPU cores section
        let mut cores_column_inner =
            iced::widget::Column::new().push(iced::widget::text("CPU CORES").size(20)); // Title
        for i in 0..self.cores {
            // Get usage history for this core
            let history = self.core_usages[i].clone();
            // Create row with label and chart
            let label = iced::widget::text(format!("Core {}", i));
            let chart = iced::widget::container(
                Canvas::new(BarChartProgram { history })
                    .width(Length::Fill)
                    .height(Length::Fixed(BAR_HEIGHT)),
            )
            .style(iced::theme::Container::Custom(Box::new(black_border)));
            let row = iced::widget::Row::new()
                .push(label)
                .push(chart)
                .spacing(10);
            cores_column_inner = cores_column_inner.push(row);
        }

        // Create the CPU threads section
        let mut threads_column_inner =
            iced::widget::Column::new().push(iced::widget::text("CPU THREADS").size(20)); // Title
        for i in 0..self.cores {
            // For each core, create a row of thread bars
            let mut thread_row = iced::widget::Row::new();
            for j in 0..threads_per_core {
                let idx = i * threads_per_core + j; // Calculate thread index
                                                    // Get usage history for this thread
                let current = self.thread_usages[idx][0];
                let previous = self.thread_usages[idx].get(1).copied().unwrap_or(0.0);
                let oldest = self.thread_usages[idx].get(2).copied().unwrap_or(0.0);
                // Add thread bar to the row
                thread_row = thread_row.push(
                    iced::widget::container(
                        Canvas::new(OverlayBarProgram {
                            current,
                            previous,
                            oldest,
                        })
                        .width(Length::Fill)
                        .height(Length::Fixed(BAR_HEIGHT)),
                    )
                    .style(iced::theme::Container::Custom(Box::new(black_border))),
                );
            }
            // Add the row of threads for this core to the column
            threads_column_inner = threads_column_inner.push(thread_row);
        }

        // Make columns fill available width, create row of graphs, then main column layout
        let cores_column = cores_column_inner.width(Length::FillPortion(68)); // 68% width
        let threads_column = threads_column_inner.width(Length::FillPortion(32)); // 32% width
        let graphs_row = iced::widget::Row::new()
            .push(cores_column)
            .push(threads_column)
            .spacing(SPACING); // Side by side
        let graphs_container = container(graphs_row)
            .style(iced::theme::Container::Custom(Box::new(black_filled_box)))
            .padding(10); // Black filled box with rounded corners and 10px margin on all sides

        // Create the top processes section
        let headline = iced::widget::text("Top User Threads").size(20);
        let horizontal_line = container(iced::widget::Space::new(Length::Fill, Length::Fixed(1.0)))
            .style(iced::theme::Container::Custom(Box::new(light_grey_line)));
        let mut table_row = iced::widget::Row::new().width(Length::Fill);
        for (i, process) in self.top_processes.iter().enumerate() {
            let text = format!("{}: {}", i + 1, process);
            let column = iced::widget::text(text).width(Length::FillPortion(1));
            table_row = table_row.push(column);
        }
        let top_processes_column = iced::widget::Column::new()
            .push(headline)
            .push(iced::widget::Space::new(Length::Shrink, Length::Fixed(4.0)))
            .push(horizontal_line)
            .push(iced::widget::Space::new(Length::Shrink, Length::Fixed(10.0)))
            .push(table_row);
        let top_processes_container = container(top_processes_column)
            .style(iced::theme::Container::Custom(Box::new(dark_grey_box)))
            .padding(10);

        // Create the GFX Card section
        let gpu_logo = if self.gpu_model.to_lowercase().contains("nvidia") {
            Image::new(Handle::from_memory(NVIDIA_LOGO))
        } else if self.gpu_model.to_lowercase().contains("amd") {
            Image::new(Handle::from_memory(AMD_GPU_LOGO))
        } else if self.gpu_model.to_lowercase().contains("intel") {
            Image::new(Handle::from_memory(INTEL_GPU_LOGO))
        } else {
            Image::new(Handle::from_memory(NVIDIA_LOGO)) // Default
        }
        .width(128)
        .height(128);

        let gpu_info = iced::widget::Column::new()
            .push(iced::widget::text(format!("Model: {}", self.gpu_model)))
            .push(iced::widget::text(format!("VRAM: {}", self.gpu_vram)));

        let gpu_table = iced::widget::Row::new()
            .push(gpu_logo)
            .push(gpu_info)
            .spacing(10)
            .align_items(iced::Alignment::Center);

        let gpu_container = container(gpu_table)
            .style(iced::theme::Container::Custom(Box::new(grey_242323_box)))
            .padding(10);

        // Create GFX Monitor section
        let mut gfx_monitor_column = iced::widget::Column::new()
            .push(iced::widget::text("GFX Monitor").size(20));
        if !self.gfx_status.is_empty() {
            gfx_monitor_column = gfx_monitor_column.push(iced::widget::text(&self.gfx_status));
        }
        let gfx_monitor_container = container(gfx_monitor_column)
            .style(iced::theme::Container::Custom(Box::new(grey_242323_box)))
            .padding(10);

        // Bottom row: GFX section on left, GFX Monitor on right
        let bottom_row = iced::widget::Row::new()
            .push(gpu_container)
            .push(iced::widget::Space::new(Length::Fill, Length::Shrink))
            .push(gfx_monitor_container);

        // Final layout: top row, then graphs container, then top processes, then bottom row, with spacing
        let main_column = iced::widget::Column::new()
            .push(top_row)
            .push(graphs_container)
            .push(top_processes_container)
            .push(bottom_row)
            .spacing(SPACING);
        container(main_column).padding([0, 10, 0, 10]).into() // 10px padding on left and right from app edges
    }

    // Set up a background task to monitor CPU usage and send updates
    fn subscription(&self) -> Subscription<Message> {
        iced::Subscription::batch(vec![
            iced::Subscription::from_recipe(CpuCoresMonitor),
            iced::Subscription::from_recipe(CpuThreadsMonitor),
            iced::Subscription::from_recipe(TempMonitor),
            iced::Subscription::from_recipe(TopProcessesMonitor::default()),
            iced::Subscription::from_recipe(GfxMonitor),
            time::every(std::time::Duration::from_secs(1)).map(|_| {
                let now = chrono::Local::now();
                Message::TimeUpdate(now.format("%H:%M:%S").to_string())
            }),
        ])
    }

    // Choose the app theme based on Windows system theme
    fn theme(&self) -> Theme {
        unsafe {
            // Open Windows registry key for theme settings
            let mut key = windows::Win32::System::Registry::HKEY::default();
            let path = windows::core::w!(
                "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"
            );
            if RegOpenKeyExW(HKEY_CURRENT_USER, path, 0, KEY_READ, &mut key).is_ok() {
                // Read the "AppsUseLightTheme" value
                let mut value: u32 = 0;
                let mut size: u32 = std::mem::size_of::<u32>() as u32;
                let value_name = windows::core::w!("AppsUseLightTheme");
                if RegQueryValueExW(
                    key,
                    value_name,
                    None,
                    None,
                    Some(&mut value as *mut _ as *mut u8),
                    Some(&mut size),
                )
                .is_ok()
                    && value == 0
                {
                    // 0 means dark theme
                    return Theme::Dark;
                }
                // Close the registry key
                let _ = RegCloseKey(key);
            }
        }
        // Default to light theme if can't read or error
        Theme::Light
    }
}

// Main function: start the app with default settings
fn main() -> iced::Result {
    Cutemonitor::run(Settings::default())
}
