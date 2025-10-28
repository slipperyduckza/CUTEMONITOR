// This module contains utility functions that are used throughout the application

// Import Windows API functions for checking administrator privileges
use windows::Win32::Security::{
    GetTokenInformation, TOKEN_ELEVATION, TOKEN_INFORMATION_CLASS, TOKEN_QUERY,
};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

/// Checks if the current process is running with administrator privileges
/// This is required because hardware monitoring libraries need elevated permissions
/// Returns true if running as admin, false otherwise
pub fn is_admin() -> bool {
    // This function uses unsafe Windows API calls to check process privileges
    unsafe {
        // Create a handle to store the process token
        let mut token = std::mem::zeroed();

        // Get the access token for the current process
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false; // Failed to get token, assume not admin
        }

        // Structure to hold elevation information
        let mut elevation = TOKEN_ELEVATION::default();
        let mut size = 0;

        // Query the token for elevation information
        if GetTokenInformation(
            token, // The token to query
            TOKEN_INFORMATION_CLASS(20), // TokenElevation class
            Some(&mut elevation as *mut _ as *mut std::ffi::c_void), // Buffer to fill
            std::mem::size_of::<TOKEN_ELEVATION>() as u32, // Buffer size
            &mut size, // Size returned
        )
        .is_err()
        {
            return false; // Failed to get elevation info, assume not admin
        }

        // Check if the token is elevated (non-zero means elevated/admin)
        elevation.TokenIsElevated != 0
    }
}