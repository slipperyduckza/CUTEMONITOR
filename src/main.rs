// Import necessary modules from the iced GUI library for building the app
use iced::{
    alignment, border, event, time, Application, Background, Border, Color, Command, Element,
    Length, Settings, Subscription, Theme,
};
// Import GUI widgets: Container for styling, Canvas for custom drawing
use iced::widget::{button, canvas, container, tooltip, tooltip::Position, Canvas, Image};
use iced::advanced::image::Handle;
// Import sysinfo to get system information like CPU usage
use sysinfo::System;
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

// Constants for easy configuration
const HISTORY_SIZE: usize = 3; // How many past CPU readings to keep
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

// Function to get temperatures by running the .NET app
async fn get_temperatures() -> Vec<String> {
    // Create a temp directory
    let temp_dir = std::env::temp_dir().join("cutemonitor_temp");
    if let Err(_) = tokio::fs::create_dir_all(&temp_dir).await {
        return vec!["Error creating temp dir".to_string()];
    }

    // Write the files
    let cs_path = temp_dir.join("TempMonitor.cs");
    let csproj_path = temp_dir.join("TempMonitor.csproj");
    let dll_path = temp_dir.join("LibreHardwareMonitorLib.dll");

    if let Err(_) = tokio::fs::write(&cs_path, TEMP_CS).await {
        return vec!["Error writing cs".to_string()];
    }
    if let Err(_) = tokio::fs::write(&csproj_path, TEMP_CSPROJ).await {
        return vec!["Error writing csproj".to_string()];
    }
    if let Err(_) = tokio::fs::write(&dll_path, TEMP_DLL).await {
        return vec!["Error writing dll".to_string()];
    }

    // Run the command
    let output = TokioCommand::new("dotnet")
        .arg("run")
        .arg("--project")
        .arg(&csproj_path)
        .creation_flags(CREATE_NO_WINDOW.0 as u32)
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
    UpdateUsages(Vec<f32>, Vec<f32>), // Contains new CPU core usages and thread usages
    TimeUpdate(String),               // New time string
    TempUpdate(Vec<String>),          // New temperature data
    RequestAdmin,                     // Request to run as admin
}

// Recipe for CPU monitoring subscription
struct CpuMonitor;

impl Recipe for CpuMonitor {
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
            let cores = System::physical_core_count().unwrap_or(1);
            let core_usages: Vec<f32> = (0..cores).map(|i| sys.cpus()[i].cpu_usage()).collect();
            let thread_usages: Vec<f32> = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();
            Some((Message::UpdateUsages(core_usages, thread_usages), ()))
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

// The main app struct that holds all the data and state
struct Cutemonitor {
    model: String,                // CPU model name
    cores: usize,                 // Number of CPU cores
    threads: usize,               // Number of CPU threads
    core_usages: Vec<Vec<f32>>,   // History of CPU core usages (each core has a vec of past usages)
    thread_usages: Vec<Vec<f32>>, // History of CPU thread usages
    date: String,                 // Current date string
    time: String,                 // Current time string
    temperatures: Vec<String>,    // CPU temperatures
}

impl Application for Cutemonitor {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    // Set the window title
    fn title(&self) -> String {
        "CPU Monitor".to_string()
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

        // Initialize usage history vectors with zeros
        let core_usages = vec![vec![0.0; HISTORY_SIZE]; cores];
        let thread_usages = vec![vec![0.0; HISTORY_SIZE]; threads];

        // Initialize date and time
        let now = chrono::Local::now();
        let date = format!("{} {}", now.format("%A"), now.format("%x"));
        let time = now.format("%H:%M:%S").to_string();

        // Create the app instance
        let app = Cutemonitor {
            model,
            cores,
            threads,
            core_usages,
            thread_usages,
            date,
            time,
            temperatures: vec![],
        };

        // Return the app and no initial command
        (app, Command::none())
    }

    // Handle messages to update the app's state
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::UpdateUsages(core, thread) => {
                // Update core usage history: add new value at the beginning, keep only HISTORY_SIZE entries
                for (i, &usage) in core.iter().enumerate() {
                    self.core_usages[i].insert(0, usage);
                    self.core_usages[i].truncate(HISTORY_SIZE);
                }
                // Same for thread usages
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
            let current = self.core_usages[i][0]; // Most recent
            let previous = self.core_usages[i].get(1).copied().unwrap_or(0.0); // One step back
            let oldest = self.core_usages[i].get(2).copied().unwrap_or(0.0); // Two steps back
                                                                             // Add a progress bar with overlaid historical data
            cores_column_inner = cores_column_inner.push(
                iced::widget::container(
                    // Container for styling
                    Canvas::new(OverlayBarProgram {
                        current,
                        previous,
                        oldest,
                    }) // Custom canvas for bars
                    .width(Length::Fill)
                    .height(Length::Fixed(BAR_HEIGHT)), // Fill width, fixed height
                )
                .style(iced::theme::Container::Custom(Box::new(black_border))), // Add black border
            );
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
        let cores_column = cores_column_inner.width(Length::Fill); // Expand to fill width
        let threads_column = threads_column_inner.width(Length::Fill);
        let graphs_row = iced::widget::Row::new()
            .push(cores_column)
            .push(threads_column)
            .spacing(SPACING); // Side by side
        let graphs_container = container(graphs_row)
            .style(iced::theme::Container::Custom(Box::new(black_filled_box)))
            .padding(10); // Black filled box with rounded corners and 10px margin on all sides

        // Final layout: top row, then graphs container, with spacing
        let main_column = iced::widget::Column::new()
            .push(top_row)
            .push(graphs_container)
            .spacing(SPACING);
        container(main_column).padding([0, 10, 0, 10]).into() // 10px padding on left and right from app edges
    }

    // Set up a background task to monitor CPU usage and send updates
    fn subscription(&self) -> Subscription<Message> {
        iced::Subscription::batch(vec![
            iced::Subscription::from_recipe(CpuMonitor),
            iced::Subscription::from_recipe(TempMonitor),
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
