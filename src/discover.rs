use core::{convert::From, ffi::CStr};
use std::{
    ffi::OsStr,
    fs::{DirEntry, File},
    io::Read,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};

use anyhow::bail;
use home_dir::HomeDirExt;

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
        let fullpath = fullpath.expand_home().unwrap_or(fullpath);
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

fn parse_desktop_file(path: PathBuf) -> anyhow::Result<Option<DesktopEntry>> {
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
