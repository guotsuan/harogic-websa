//! Loading contract for the process-wide HAROGIC session shim.
use std::ffi::{c_char, c_void};

#[cfg(unix)]
use std::ffi::{c_int, CStr, CString};

#[cfg(unix)]
#[link(name = "dl")]
unsafe extern "C" {
    pub(crate) fn dlopen(path: *const c_char, flags: c_int) -> *mut c_void;
    pub(crate) fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    pub(crate) fn dlclose(handle: *mut c_void) -> c_int;
    fn dlerror() -> *const c_char;
}

/// Linux RTLD_NOW | RTLD_NODELETE: the session and its function pointers
/// outlive individual acquisition modes, including failed opens.
#[cfg(unix)]
pub(crate) const SDK_DLOPEN_FLAGS: c_int = 2 | 0x1000;

#[cfg(unix)]
pub(crate) fn open(path: &str) -> Result<*mut c_void, String> {
    let path = CString::new(path).map_err(|_| "invalid SDK shim path")?;
    let library = unsafe { dlopen(path.as_ptr(), SDK_DLOPEN_FLAGS) };
    if library.is_null() {
        let reason = unsafe { dlerror() };
        return if reason.is_null() {
            Err("dynamic loader returned no diagnostic".into())
        } else {
            Err(unsafe { CStr::from_ptr(reason) }
                .to_string_lossy()
                .into_owned())
        };
    }
    Ok(library)
}

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn LoadLibraryExW(path: *const u16, file: *mut c_void, flags: u32) -> *mut c_void;
    fn GetModuleHandleExW(flags: u32, address: *const u16, module: *mut *mut c_void) -> i32;
    fn GetProcAddress(module: *mut c_void, symbol: *const c_char) -> *mut c_void;
    fn FreeLibrary(module: *mut c_void) -> i32;
}

#[cfg(windows)]
pub(crate) fn open(path: &str) -> Result<*mut c_void, String> {
    use std::{io, os::windows::ffi::OsStrExt, path::Path, ptr};

    const LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR: u32 = 0x100;
    const LOAD_LIBRARY_SEARCH_DEFAULT_DIRS: u32 = 0x1000;
    const GET_MODULE_HANDLE_EX_FLAG_PIN: u32 = 1;
    const GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS: u32 = 4;

    let path = std::path::absolute(Path::new(path)).map_err(|e| e.to_string())?;
    let mut wide: Vec<u16> = path.as_os_str().encode_wide().collect();
    if wide.contains(&0) {
        return Err("invalid SDK shim path".into());
    }
    wide.push(0);
    // Resolve the vendor DLL and its private dependencies beside the shim.
    let library = unsafe {
        LoadLibraryExW(
            wide.as_ptr(),
            ptr::null_mut(),
            LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_DEFAULT_DIRS,
        )
    };
    if library.is_null() {
        return Err(io::Error::last_os_error().to_string());
    }
    let mut pinned = ptr::null_mut();
    let result = unsafe {
        GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FLAG_PIN | GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
            library.cast(),
            &mut pinned,
        )
    };
    if result == 0 {
        let error = io::Error::last_os_error();
        unsafe {
            FreeLibrary(library);
        }
        return Err(error.to_string());
    }
    Ok(library)
}

#[cfg(windows)]
pub(crate) unsafe fn dlsym(library: *mut c_void, symbol: *const c_char) -> *mut c_void {
    unsafe { GetProcAddress(library, symbol) }
}

#[cfg(windows)]
pub(crate) unsafe fn dlclose(library: *mut c_void) {
    unsafe {
        FreeLibrary(library);
    }
}
