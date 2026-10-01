//! Permission handling for files MCP Manager creates or replaces.
//!
//! On Unix, `rename` keeps the temporary file's mode and owner, not the
//! destination's. A temp file created under a typical umask (`022`) would turn
//! an existing `0600` config into `0644`. Replacement files are therefore
//! created with the destination's mode (or `0600` when the file is new) and
//! the previous owner/group is copied when the platform allows it.
//!
//! Windows has no mode bits. New files receive an owner-only DACL. Replacing
//! an existing file copies its DACL onto the temporary file before the rename
//! so the write does not widen access.
//!
//! Directory modes are only tightened for directories this process creates
//! under the MCP Manager application-data directory. Client config directories
//! the user already has are left alone.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

pub const SECRET_FILE_MODE: u32 = 0o600;
pub const SECRET_DIR_MODE: u32 = 0o700;

/// Writes `content` to a new file at `destination`, then renames it over `path`.
/// The caller deletes `destination` if this returns an error.
pub fn atomic_replace_file(destination: &Path, path: &Path, content: &[u8]) -> Result<(), String> {
    let mode = existing_mode(path).unwrap_or(SECRET_FILE_MODE);
    write_new_file(destination, content, mode)?;
    if path.is_file() {
        // Best effort. Failure leaves the file owned by the current user,
        // which is the owner of `path` in normal use, and never widens mode.
        copy_owner_and_acl(path, destination);
    }
    Ok(())
}

/// `0600` on Unix, owner-only DACL on Windows.
pub fn restrict_new_file(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        set_unix_mode(path, SECRET_FILE_MODE)
    }
    #[cfg(windows)]
    {
        windows::owner_only(path)
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = path;
        Ok(())
    }
}

/// `0700` on Unix, owner-only DACL on Windows.
///
/// On Unix this only removes group and other bits. It does not grant the
/// owner a bit they do not already have, so a directory made read-only on
/// purpose stays read-only and a failed save can still be detected.
pub fn restrict_new_dir(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let current = fs::metadata(path)
            .map_err(|e| e.to_string())?
            .permissions()
            .mode()
            & 0o777;
        let tightened = current & SECRET_DIR_MODE;
        if tightened != current {
            set_unix_mode(path, tightened)?;
        }
        Ok(())
    }
    #[cfg(windows)]
    {
        windows::owner_only(path)
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = path;
        Ok(())
    }
}

/// Tightens directories from the application-data root down to `file`'s parent
/// when `file` lives there. Other directories are not modified.
pub fn tighten_app_data_dirs(file: &Path) {
    let root = crate::platform::PlatformContext::current().app_data_dir();
    let Ok(relative) = file.strip_prefix(&root) else {
        return;
    };
    let _ = restrict_new_dir(&root);
    let mut acc = root;
    let Some(parent) = relative.parent() else {
        return;
    };
    for component in parent.components() {
        acc.push(component);
        if acc.is_dir() {
            let _ = restrict_new_dir(&acc);
        }
    }
}

fn existing_mode(path: &Path) -> Option<u32> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(path).ok()?.permissions().mode() & 0o777;
        Some(mode)
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        None
    }
}

fn write_new_file(path: &Path, content: &[u8], mode: u32) -> Result<(), String> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        // umask can only clear bits. `set_permissions` below restores the
        // exact mode of an existing file, which may be wider than the umask
        // but never wider than that file already was.
        options.mode(mode & 0o777);
    }
    let mut file = options.open(path).map_err(|e| e.to_string())?;
    file.write_all(content).map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        set_unix_mode(path, mode)?;
    }
    #[cfg(windows)]
    {
        // New files start owner-only. `copy_owner_and_acl` replaces that with
        // the destination DACL when the destination already exists.
        windows::owner_only(path)?;
    }
    let _ = file;
    Ok(())
}

#[cfg(unix)]
fn set_unix_mode(path: &Path, mode: u32) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(path).map_err(|e| e.to_string())?.permissions();
    permissions.set_mode(mode & 0o777);
    fs::set_permissions(path, permissions).map_err(|e| e.to_string())
}

#[cfg(unix)]
fn copy_owner_and_acl(from: &Path, to: &Path) {
    use std::os::unix::fs::MetadataExt;
    use std::os::unix::io::AsRawFd;
    let Ok(meta) = fs::metadata(from) else {
        return;
    };
    let Ok(file) = fs::File::open(to) else {
        return;
    };
    // `fchown` is not in stable `std`. It copies the existing owner and group
    // onto the replacement inode so a `0640` file keeps its group instead of
    // becoming a private file of only the writer. Failure is ignored: the
    // mode set above is already no wider than the original.
    let _ = unsafe { libc::fchown(file.as_raw_fd(), meta.uid(), meta.gid()) };
}

#[cfg(not(unix))]
fn copy_owner_and_acl(from: &Path, to: &Path) {
    #[cfg(windows)]
    windows::copy_dacl(from, to);
    #[cfg(not(windows))]
    {
        let _ = (from, to);
    }
}

#[cfg(windows)]
mod windows {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::path::Path;
    use std::ptr;

    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::Security::Authorization::ConvertStringSecurityDescriptorToSecurityDescriptorW;
    use windows_sys::Win32::Security::{
        DACL_SECURITY_INFORMATION, GROUP_SECURITY_INFORMATION, OWNER_SECURITY_INFORMATION,
        PSECURITY_DESCRIPTOR,
    };
    use windows_sys::Win32::Storage::FileSystem::{GetFileSecurityW, SetFileSecurityW};

    const SDDL_REVISION_1: u32 = 1;

    pub fn owner_only(path: &Path) -> Result<(), String> {
        // Protected DACL, file-all to the owner, no inherited ACEs.
        apply_sddl(path, "D:P(A;;FA;;;OW)")
    }

    pub fn copy_dacl(from: &Path, to: &Path) {
        let info =
            DACL_SECURITY_INFORMATION | OWNER_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION;
        let Some(from) = wide(from) else {
            return;
        };
        let Some(to) = wide(to) else {
            return;
        };
        let mut needed = 0u32;
        // The first call only reports the buffer size.
        unsafe {
            GetFileSecurityW(from.as_ptr(), info, ptr::null_mut(), 0, &mut needed);
        }
        if needed == 0 {
            return;
        }
        let mut buffer = vec![0u8; needed as usize];
        let ok = unsafe {
            GetFileSecurityW(
                from.as_ptr(),
                info,
                buffer.as_mut_ptr().cast(),
                needed,
                &mut needed,
            )
        };
        if ok == 0 {
            return;
        }
        unsafe {
            SetFileSecurityW(to.as_ptr(), info, buffer.as_ptr().cast());
        }
    }

    fn apply_sddl(path: &Path, sddl: &str) -> Result<(), String> {
        let Some(path) = wide(path) else {
            return Err("path is not valid Unicode".to_string());
        };
        let descriptor_text: Vec<u16> = OsStr::new(sddl).encode_wide().chain(Some(0)).collect();
        let mut sd: PSECURITY_DESCRIPTOR = ptr::null_mut();
        let ok = unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                descriptor_text.as_ptr(),
                SDDL_REVISION_1,
                &mut sd,
                ptr::null_mut(),
            )
        };
        if ok == 0 || sd.is_null() {
            return Err("could not build a restrictive Windows DACL".to_string());
        }
        let applied = unsafe { SetFileSecurityW(path.as_ptr(), DACL_SECURITY_INFORMATION, sd) };
        unsafe {
            LocalFree(sd.cast());
        }
        if applied == 0 {
            return Err("could not apply a restrictive Windows DACL".to_string());
        }
        Ok(())
    }

    fn wide(path: &Path) -> Option<Vec<u16>> {
        Some(path.as_os_str().encode_wide().chain(Some(0)).collect())
    }
}
