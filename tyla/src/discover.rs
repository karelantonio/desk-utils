use core::convert::From;
use std::{
    ffi::{CStr, CString, OsStr, OsString},
    fs::{DirEntry, File},
    io::Read,
    os::unix::ffi::OsStrExt,
    path::{Component, Path, PathBuf},
};

use libc::getuid;

fn user_home() -> Option<PathBuf> {
    // first try the environment variable
    if let Some(var) = std::env::var_os("HOME") {
        return Some(PathBuf::from(var));
    }

    // Else the passwd way
    unsafe {
        let res = libc::getpwuid(getuid());

        if res.is_null() || (*res).pw_dir.is_null() {
            None
        } else {
            let s: &[u8] = CStr::from_ptr((*res).pw_dir).to_bytes();
            Some(PathBuf::from(OsStr::from_bytes(s)).into())
        }
    }
}

fn user_home_of(name: &[u8]) -> Option<PathBuf> {
    // same as above
    unsafe {
        let name = CString::new(name).ok()?;
        let res = libc::getpwnam(name.as_ptr());

        if res.is_null() || (*res).pw_dir.is_null() {
            None
        } else {
            let s: &[u8] = CStr::from_ptr((*res).pw_dir).to_bytes();
            Some(PathBuf::from(OsStr::from_bytes(s)).into())
        }
    }
}

fn expand_home(path: PathBuf) -> PathBuf {
    let mut res = PathBuf::new();

    let mut comps = path.as_path().components();
    match comps.next() {
        Some(Component::Normal(s)) => {
            let s_bytes = s.as_bytes();
            if s_bytes == b"~"
                && let Some(homedir) = user_home()
            {
                res.push(homedir);
            } else if s_bytes.starts_with(b"~")
                && let Some(homedir) = user_home_of(&s_bytes[1..])
            {
                res.push(homedir);
            } else {
                res.push(Component::Normal(s));
            }
        }
        Some(p) => res.push(p),
        Option::None => (),
    }

    for com in comps {
        res.push(com);
    }

    res
}

/// Get the XDG data directories from the environment or return the default ones
fn xdg_data_dirs() -> Vec<String> {
    match std::env::var("XDG_DATA_DIRS") {
        Ok(val) => val.split(':').map(Into::into).collect(),
        Err(err) => {
            log::debug!(
                "Environment variable: XDG_DATA_DIRS not found or invalid ({err}), using default (/usr/local/share, /usr/share and ~/.local/share)"
            );
            vec![
                "/usr/local/share".into(),
                "/usr/share".into(),
                "~/.local/share".into(),
            ]
        }
    }
}

#[derive(Debug, Clone)]
pub struct DesktopEntry {
    pub name: String,
    pub icon: Option<String>,
    pub cmd: String,
    pub terminal: bool,
}

pub fn desktop_apps() -> Vec<DesktopEntry> {
    let mut apps = Vec::new();
    for dir in xdg_data_dirs() {
        let fullpath = PathBuf::from(format!("{dir}/applications"));
        let fullpath = expand_home(fullpath);
        log::debug!("Checking directory '{fullpath:?}' for desktop entries");
        let files = match std::fs::read_dir(fullpath) {
            Ok(dirs) => dirs,
            Err(err) => {
                log::warn!("Could not read directory: {err}, skipping");
                continue;
            }
        };

        for entry in files {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    log::warn!("Could not read entry: {err}, ignoring...");
                    continue;
                }
            };

            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            if Some(OsStr::from_bytes(b"desktop")) != path.extension() {
                continue;
            }

            // Is a desktop file, parse
            let entry = match parse_desktop_file(path) {
                Ok(e) => e,
                Err(err) => {
                    log::warn!("Error (ignored): {err}");
                    continue;
                }
            };

            let Some(app) = entry else { continue };

            apps.push(app);
        }
    }
    apps
}

fn parse_desktop_file(path: PathBuf) -> Result<Option<DesktopEntry>, Box<dyn std::error::Error>> {
    log::debug!("Parsing file: {path:?}");
    // First read the file
    let mut content = Vec::new();
    let _ = File::open(path)?.read_to_end(&mut content)?;
    let mut in_desktop_entry = false;

    let mut name = None;
    let mut icon = None;
    let mut cmd = None;
    let mut terminal = false;
    let mut isapp = false;

    for mut line in content.split(|c| *c == b'\n') {
        while line.len() > 0 && (line[0] == b' ' || line[0] == b'\t') {
            line = &line[1..];
        }

        if line.len() == 0 {
            // Empty
            continue;
        }

        if line.starts_with(b"[") {
            // Section name
            in_desktop_entry = line.starts_with(b"[Desktop Entry]");
            continue;
        }

        if line.starts_with(b"#") {
            // Comment
            continue;
        }

        if !line.contains(&b'=') {
            // Not important or invalid i guess
            continue;
        }
        let mut sli = line.splitn(2, |c| *c == b'=');
        let (Some(bname), Some(bvalue), Option::None) = (sli.next(), sli.next(), sli.next()) else {
            continue; // Else invalid
        };

        if !in_desktop_entry {
            continue;
        }

        if bname == b"Name" {
            name = Some(String::from_utf8(bvalue.into())?);
        } else if bname == b"Icon" {
            icon = Some(String::from_utf8(bvalue.into())?);
        } else if bname == b"Type" {
            isapp = bvalue == b"Application";
        } else if bname == b"Terminal" {
            terminal = bvalue == b"true";
        } else if bname == b"Exec" {
            if bvalue.contains(&b'%') {
                // Not supported rn :(, implement later
                continue;
            }
            cmd = Some(String::from_utf8(bvalue.into())?);
        }
    }

    let Some(name) = name else {
        return Ok(None);
    };

    let Some(cmd) = cmd else {
        return Ok(None);
    };

    if !isapp {
        return Ok(None);
    }
    Ok(Some(DesktopEntry {
        name,
        cmd,
        icon,
        terminal,
    }))
}
