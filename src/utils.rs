// This module contains utility functions that are used throughout the application

// Import Windows API functions for checking administrator privileges
use windows::Win32::Security::{
    GetTokenInformation, TOKEN_ELEVATION, TOKEN_INFORMATION_CLASS, TOKEN_QUERY,
};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows::Win32::System::Registry::*;

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

pub fn save_window_position(x: i32, y: i32) {
    unsafe {
        let mut key: HKEY = HKEY::default();
        let subkey = windows::core::PCWSTR::from_raw("Software\\Cutemonitor\0".encode_utf16().collect::<Vec<_>>().as_ptr());
        if RegCreateKeyExW(HKEY_CURRENT_USER, subkey, 0, windows::core::PCWSTR::null(), REG_OPTION_NON_VOLATILE, KEY_WRITE, None, &mut key, None).is_ok() {
            let value_name = windows::core::PCWSTR::from_raw("WindowPosition\0".encode_utf16().collect::<Vec<_>>().as_ptr());
            let data = format!("{},{}", x, y);
            let data_bytes = data.as_bytes();
            let _ = RegSetValueExW(key, value_name, 0, REG_SZ, Some(data_bytes));
            let _ = RegCloseKey(key);
        }
    }
}

pub fn load_window_position() -> Option<(i32, i32)> {
    unsafe {
        let mut key: HKEY = HKEY::default();
        let subkey = windows::core::PCWSTR::from_raw("Software\\Cutemonitor\0".encode_utf16().collect::<Vec<_>>().as_ptr());
        if RegOpenKeyExW(HKEY_CURRENT_USER, subkey, 0, KEY_READ, &mut key).is_ok() {
            let value_name = windows::core::PCWSTR::from_raw("WindowPosition\0".encode_utf16().collect::<Vec<_>>().as_ptr());
            let mut data_type: REG_VALUE_TYPE = REG_VALUE_TYPE::default();
            let mut data_size: u32 = 0;
            if RegQueryValueExW(key, value_name, None, Some(&mut data_type), None, Some(&mut data_size)).is_ok() && data_type == REG_SZ {
                let mut buffer = vec![0u8; data_size as usize];
                if RegQueryValueExW(key, value_name, None, Some(&mut data_type), Some(buffer.as_mut_ptr()), Some(&mut data_size)).is_ok() {
                    if let Ok(s) = String::from_utf8(buffer[..(data_size as usize).saturating_sub(2)].to_vec()) { // -2 for null terminator
                        if let Some((x_str, y_str)) = s.split_once(',') {
                            if let (Ok(x), Ok(y)) = (x_str.parse::<i32>(), y_str.parse::<i32>()) {
                                let _ = RegCloseKey(key);
                                return Some((x, y));
                            }
                        }
                    }
                }
            }
            let _ = RegCloseKey(key);
        }
    }
    None
}