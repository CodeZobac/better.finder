/// Everything SDK FFI bindings for Windows file search
/// 
/// This module provides Rust bindings to the Everything SDK DLL,
/// which enables ultra-fast file searching on Windows.

use crate::error::{LauncherError, Result};
use std::path::PathBuf;

#[cfg(windows)]
use std::ffi::OsStr;

#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;

#[cfg(windows)]
use tracing::{debug, error, info, warn};

#[cfg(windows)]
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};

#[cfg(windows)]
use windows::core::PCWSTR;

// Everything SDK constants
#[cfg(windows)]
const EVERYTHING_REQUEST_FILE_NAME: u32 = 0x00000001;
#[cfg(windows)]
const EVERYTHING_REQUEST_PATH: u32 = 0x00000002;
#[cfg(windows)]
const EVERYTHING_REQUEST_FULL_PATH_AND_FILE_NAME: u32 = 0x00000004;
#[cfg(windows)]
const EVERYTHING_REQUEST_EXTENSION: u32 = 0x00000008;
#[cfg(windows)]
const EVERYTHING_REQUEST_SIZE: u32 = 0x00000010;
#[cfg(windows)]
const EVERYTHING_REQUEST_DATE_MODIFIED: u32 = 0x00000040;

#[cfg(windows)]
const EVERYTHING_SORT_NAME_ASCENDING: u32 = 1;
#[cfg(windows)]
#[allow(dead_code)]
const EVERYTHING_SORT_NAME_DESCENDING: u32 = 2;
#[cfg(windows)]
#[allow(dead_code)]
const EVERYTHING_SORT_PATH_ASCENDING: u32 = 3;
#[cfg(windows)]
#[allow(dead_code)]
const EVERYTHING_SORT_SIZE_ASCENDING: u32 = 5;
#[cfg(windows)]
#[allow(dead_code)]
const EVERYTHING_SORT_DATE_MODIFIED_DESCENDING: u32 = 12;

// Everything SDK FFI function types
#[cfg(windows)]
type EverythingSetSearchW = unsafe extern "C" fn(*const u16);
#[cfg(windows)]
type EverythingSetRequestFlags = unsafe extern "C" fn(u32);
#[cfg(windows)]
type EverythingSetMax = unsafe extern "C" fn(u32);
#[cfg(windows)]
type EverythingSetSort = unsafe extern "C" fn(u32);
#[cfg(windows)]
type EverythingQueryW = unsafe extern "C" fn(bool) -> bool;
#[cfg(windows)]
type EverythingGetNumResults = unsafe extern "C" fn() -> u32;
#[cfg(windows)]
type EverythingGetResultFileNameW = unsafe extern "C" fn(u32) -> *const u16;
#[cfg(windows)]
type EverythingGetResultPathW = unsafe extern "C" fn(u32) -> *const u16;
#[cfg(windows)]
type EverythingGetResultFullPathNameW = unsafe extern "C" fn(u32, *mut u16, u32) -> u32;
#[cfg(windows)]
type EverythingGetResultSize = unsafe extern "C" fn(u32, *mut u32) -> u32;
#[cfg(windows)]
type EverythingGetResultDateModified = unsafe extern "C" fn(u32, *mut u32) -> u32;
#[cfg(windows)]
type EverythingGetLastError = unsafe extern "C" fn() -> u32;
#[cfg(windows)]
type EverythingIsDBLoaded = unsafe extern "C" fn() -> bool;

// Everything SDK function pointers
#[cfg(windows)]
struct EverythingFunctions {
    set_search_w: EverythingSetSearchW,
    set_request_flags: EverythingSetRequestFlags,
    set_max: EverythingSetMax,
    set_sort: EverythingSetSort,
    query_w: EverythingQueryW,
    get_num_results: EverythingGetNumResults,
    get_result_file_name_w: EverythingGetResultFileNameW,
    get_result_path_w: EverythingGetResultPathW,
    get_result_full_path_name_w: EverythingGetResultFullPathNameW,
    get_result_size: EverythingGetResultSize,
    get_result_date_modified: EverythingGetResultDateModified,
    get_last_error: EverythingGetLastError,
    is_db_loaded: EverythingIsDBLoaded,
}

/// File information returned from Everything SDK
#[derive(Debug, Clone)]
pub struct EverythingFile {
    pub name: String,
    pub path: String,
    pub full_path: PathBuf,
    pub size: u64,
    pub modified: i64,
}

/// Everything SDK client wrapper
pub struct EverythingClient {
    is_available: bool,
    #[cfg(windows)]
    functions: Option<EverythingFunctions>,
}

impl EverythingClient {
    /// Creates a new Everything client and checks if the SDK is available
    pub fn new() -> Result<Self> {
        #[cfg(windows)]
        {
            // Try to load the Everything DLL dynamically
            let functions = unsafe { Self::load_everything_dll()? };
            
            // Check if Everything database is loaded
            let is_available = unsafe { (functions.is_db_loaded)() };
            
            if !is_available {
                warn!("Everything SDK database is not loaded");
                return Err(LauncherError::EverythingNotAvailable);
            }

            info!("Everything SDK is available and database is loaded");
            Ok(Self {
                is_available: true,
                functions: Some(functions),
            })
        }

        #[cfg(not(windows))]
        {
            Err(LauncherError::EverythingNotAvailable)
        }
    }

    #[cfg(windows)]
    unsafe fn load_everything_dll() -> Result<EverythingFunctions> {
        // Try to load Everything64.dll
        let dll_name = Self::to_wide_string("Everything64.dll");
        let dll_handle = LoadLibraryW(PCWSTR(dll_name.as_ptr()))
            .map_err(|_| {
                error!("Failed to load Everything64.dll - make sure Everything is installed");
                LauncherError::EverythingNotAvailable
            })?;

        // Load all function pointers
        macro_rules! get_proc {
            ($name:expr) => {{
                let proc_name = format!("{}\0", $name);
                GetProcAddress(dll_handle, windows::core::PCSTR(proc_name.as_ptr()))
                    .ok_or_else(|| {
                        error!("Failed to load function: {}", $name);
                        LauncherError::EverythingNotAvailable
                    })?
            }};
        }

        Ok(EverythingFunctions {
            set_search_w: std::mem::transmute(get_proc!("Everything_SetSearchW")),
            set_request_flags: std::mem::transmute(get_proc!("Everything_SetRequestFlags")),
            set_max: std::mem::transmute(get_proc!("Everything_SetMax")),
            set_sort: std::mem::transmute(get_proc!("Everything_SetSort")),
            query_w: std::mem::transmute(get_proc!("Everything_QueryW")),
            get_num_results: std::mem::transmute(get_proc!("Everything_GetNumResults")),
            get_result_file_name_w: std::mem::transmute(get_proc!("Everything_GetResultFileNameW")),
            get_result_path_w: std::mem::transmute(get_proc!("Everything_GetResultPathW")),
            get_result_full_path_name_w: std::mem::transmute(get_proc!("Everything_GetResultFullPathNameW")),
            get_result_size: std::mem::transmute(get_proc!("Everything_GetResultSize")),
            get_result_date_modified: std::mem::transmute(get_proc!("Everything_GetResultDateModified")),
            get_last_error: std::mem::transmute(get_proc!("Everything_GetLastError")),
            is_db_loaded: std::mem::transmute(get_proc!("Everything_IsDBLoaded")),
        })
    }

    /// Checks if Everything SDK is available
    pub fn is_available(&self) -> bool {
        self.is_available
    }

    /// Searches for files matching the query
    pub fn search(&self, query: &str, max_results: u32) -> Result<Vec<EverythingFile>> {
        if !self.is_available {
            return Err(LauncherError::EverythingNotAvailable);
        }

        #[cfg(windows)]
        {
            let functions = self.functions.as_ref().ok_or(LauncherError::EverythingNotAvailable)?;
            
            unsafe {
                // Set search query
                let query_wide = Self::to_wide_string(query);
                (functions.set_search_w)(query_wide.as_ptr());

                // Set request flags
                (functions.set_request_flags)(
                    EVERYTHING_REQUEST_FILE_NAME
                        | EVERYTHING_REQUEST_PATH
                        | EVERYTHING_REQUEST_FULL_PATH_AND_FILE_NAME
                        | EVERYTHING_REQUEST_SIZE
                        | EVERYTHING_REQUEST_DATE_MODIFIED,
                );

                // Set max results
                (functions.set_max)(max_results);

                // Set sort order (by name)
                (functions.set_sort)(EVERYTHING_SORT_NAME_ASCENDING);

                // Execute query
                let success = (functions.query_w)(true);
                if !success {
                    let error_code = (functions.get_last_error)();
                    error!("Everything query failed with error code: {}", error_code);
                    return Err(LauncherError::SearchError(format!(
                        "Everything query failed: error code {}",
                        error_code
                    )));
                }

                // Get number of results
                let num_results = (functions.get_num_results)();
                debug!("Everything returned {} results", num_results);

                // Collect results
                let mut results = Vec::new();
                for i in 0..num_results.min(max_results) {
                    if let Some(file) = self.get_result_at_index(i) {
                        results.push(file);
                    }
                }

                Ok(results)
            }
        }

        #[cfg(not(windows))]
        {
            let _ = (query, max_results);
            Err(LauncherError::EverythingNotAvailable)
        }
    }

    #[cfg(windows)]
    unsafe fn get_result_at_index(&self, index: u32) -> Option<EverythingFile> {
        let functions = self.functions.as_ref()?;
        
        // Get file name
        let name_ptr = (functions.get_result_file_name_w)(index);
        let name = Self::from_wide_ptr(name_ptr)?;

        // Get path
        let path_ptr = (functions.get_result_path_w)(index);
        let path = Self::from_wide_ptr(path_ptr)?;

        // Get full path
        let mut full_path_buf = vec![0u16; 260]; // MAX_PATH
        let full_path_len = (functions.get_result_full_path_name_w)(
            index,
            full_path_buf.as_mut_ptr(),
            full_path_buf.len() as u32,
        );

        let full_path = if full_path_len > 0 {
            let full_path_str = Self::from_wide_slice(&full_path_buf[..full_path_len as usize])?;
            PathBuf::from(full_path_str)
        } else {
            PathBuf::from(&path).join(&name)
        };

        // Get size
        let mut size_high: u32 = 0;
        let size_low = (functions.get_result_size)(index, &mut size_high);
        let size = ((size_high as u64) << 32) | (size_low as u64);

        // Get modified date
        let mut modified_high: u32 = 0;
        let modified_low = (functions.get_result_date_modified)(index, &mut modified_high);
        let modified = ((modified_high as i64) << 32) | (modified_low as i64);

        Some(EverythingFile {
            name,
            path,
            full_path,
            size,
            modified,
        })
    }

    // Helper functions for string conversion
    #[cfg(windows)]
    fn to_wide_string(s: &str) -> Vec<u16> {
        OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    #[cfg(windows)]
    unsafe fn from_wide_ptr(ptr: *const u16) -> Option<String> {
        if ptr.is_null() {
            return None;
        }

        let len = (0..).take_while(|&i| *ptr.offset(i) != 0).count();
        let slice = std::slice::from_raw_parts(ptr, len);
        Self::from_wide_slice(slice)
    }

    #[cfg(windows)]
    fn from_wide_slice(slice: &[u16]) -> Option<String> {
        String::from_utf16(slice).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_everything_client_creation() {
        // This test will only pass if Everything is installed and running
        match EverythingClient::new() {
            Ok(client) => {
                assert!(client.is_available());
            }
            Err(LauncherError::EverythingNotAvailable) => {
                // Expected if Everything is not installed
                println!("Everything SDK not available - test skipped");
            }
            Err(e) => {
                panic!("Unexpected error: {}", e);
            }
        }
    }

    #[test]
    #[cfg(windows)]
    fn test_everything_search() {
        match EverythingClient::new() {
            Ok(client) => {
                // Search for .txt files
                match client.search("*.txt", 10) {
                    Ok(results) => {
                        println!("Found {} .txt files", results.len());
                        for file in results.iter().take(5) {
                            println!("  - {} ({})", file.name, file.path);
                        }
                    }
                    Err(e) => {
                        println!("Search failed: {}", e);
                    }
                }
            }
            Err(_) => {
                println!("Everything SDK not available - test skipped");
            }
        }
    }
}
