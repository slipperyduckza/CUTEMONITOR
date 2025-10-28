// This attribute tells Windows to hide the console window when the application runs
#![windows_subsystem = "windows"]

// Import necessary crates and modules
use iced::advanced::image::Handle; // For handling image data
use iced::window::icon; // For setting the window icon
use windows::core::PCSTR; // Windows string type for C-style strings
use windows::Win32::Foundation::HWND; // Windows handle type
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxA, MB_ICONERROR}; // Windows message box functions

use std::io::Cursor; // For reading data from memory

// Import our custom modules
use crate::state::State; // Our application state
use crate::utils::is_admin; // Function to check if running as administrator

// Embedded logos - these are compiled into the binary at build time
// CPU manufacturer logos
static AMD_LOGO: &[u8] = include_bytes!("../AMD256.png");
static INTEL_LOGO: &[u8] = include_bytes!("../INTEL256.png");
static VM_LOGO: &[u8] = include_bytes!("../VM_PC256.png"); // For virtual machines

// GPU manufacturer logos
static NVIDIA_LOGO: &[u8] = include_bytes!("../Nvidia_GeForce_256.png");
static AMD_GPU_LOGO: &[u8] = include_bytes!("../AMD_Radeon_256.png");
static INTEL_GPU_LOGO: &[u8] = include_bytes!("../Intel_Arc_256.png");

// Declare our modules - these contain the actual implementation
mod canvas; // Canvas drawing programs for charts
mod data_colouring; // Functions to color-code data based on values
mod hardware_checker; // Hardware monitoring and data collection
mod state; // Application state management
mod styles; // UI styling functions
mod subscriptions; // Asynchronous data streams
mod utils; // Utility functions
mod what_cpu_check; // CPU information detection

// Constants for easy configuration - these can be changed to customize the app
pub const HISTORY_SIZE: usize = 30; // How many past CPU readings to keep in memory
pub const BAR_HEIGHT: f32 = 24.0; // Height of each progress bar in pixels

// The main entry point of our application
pub fn main() -> iced::Result {
    // Check if we're running as administrator (required for hardware monitoring)
    if !is_admin() {
        // Show an error message box if not running as admin
        unsafe {
            MessageBoxA(
                HWND::default(), // Default window handle
                PCSTR::from_raw(b"This program requires administrator privileges.\0".as_ptr()), // Message text
                PCSTR::from_raw(b"Administrator Required\0".as_ptr()), // Window title
                MB_ICONERROR, // Error icon
            );
        }
        // Exit the program with error code 1
        std::process::exit(1);
    }

    // Load and prepare the window icon
    let icon = {
        // Read the icon file into memory at compile time
        let bytes = include_bytes!("../cutemonitor.ico");
        // Parse the ICO file format
        let icon_dir = ico::IconDir::read(Cursor::new(bytes)).unwrap();
        // Decode the first icon in the file
        let image = icon_dir.entries()[0].decode().unwrap();
        // Extract the RGBA pixel data
        let rgba = image.rgba_data().to_vec();
        let width = image.width();
        let height = image.height();
        // Create an Iced icon from the RGBA data
        icon::from_rgba(rgba, width, height).unwrap()
    };

    // Create and run the Iced application
    iced::application("LibreHardware Prototype", State::update, State::view)
        .subscription(State::subscription) // Set up data subscriptions
        .window(iced::window::Settings {
            icon: Some(icon), // Set the window icon
            size: (1120.0, 800.0).into(), // Set initial window size
            ..Default::default() // Use default settings for everything else
        })
        .run() // Start the application event loop
}