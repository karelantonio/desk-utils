use core::{
    error::Error,
    fmt::{Debug, Display},
    ptr::write_bytes,
};
use std::{
    fs::{File, read_dir, read_to_string},
    path::PathBuf,
};

enum AppError {
    Args(String),
    UnexpectedArg(String),
    ListBacklightDir(std::io::Error),
    ListEntry(std::io::Error),
    NoBright,
    ReadingFile(PathBuf, std::io::Error),
    WritingFile(PathBuf, std::io::Error),
    InvalidIntegerInFile(PathBuf),
}

impl Display for AppError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            AppError::Args(msg) => write!(f, "error in args\nCaused by: {msg}"),
            AppError::UnexpectedArg(name) => {
                write!(f, "error in args\nCaused by: unexpected arg ({name})")
            }
            AppError::NoBright => write!(
                f,
                "error in args\nCaused by: No bright specified, try --help"
            ),
            AppError::ListBacklightDir(err) => {
                write!(f, "could not read /sys/class/backlight\nCaused by: {err}")
            }
            AppError::ListEntry(err) => {
                write!(f, "error listing /sys/class/backlight\nCaused by: {err}")
            }
            AppError::ReadingFile(p, err) => write!(
                f,
                "error reading file: {}\nCaused by: {err}",
                p.to_string_lossy()
            ),
            AppError::WritingFile(p, err) => write!(
                f,
                "error writing file: {}\nCaused by: {err}",
                p.to_string_lossy()
            ),
            AppError::InvalidIntegerInFile(p) => {
                write!(f, "invalid integer in file: {}", p.to_string_lossy())
            }
        }
    }
}

impl Debug for AppError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Error for AppError {}

#[derive(Clone)]
enum Bright {
    Increment(f64),
    Decrement(f64),
    Set(f64),
}

type AppResult = Result<(), AppError>;

fn main() -> Result<(), AppError> {
    // Parse args
    let mut args = std::env::args();
    let mut newbright = None;
    let mut name = None;
    let _binname = args.next();
    while let Some(arg) = args.next() {
        if arg == "-h" || arg == "--help" {
            println!(
                r#"
Usage: bright [OPTIONS] value [name]

Options:
  -h, --help
      Show this help message

Arguments:
  value:
      Percent showing the new value. If preceded
      by a + or -, the value is interpreted as a
      delta (which is added to the current value)
  name:
      The name of one of the directories under
      /sys/class/backlight, if not specified will
      appply this value to all of them.

Examples:
  $ bright 100
     ^ Set brightess to a maximum in all devs
  $ bright 1 intel_backlight
     ^ Set the brightness to 1% on
     /sys/class/backlight/intel_backlight
  $ bright +1
     ^ Increment brightness by 1%
  $ bright -1
     ^ Decrement brightness by 1%
"#
            );
            return Ok(());
        } else if newbright.is_none() {
            // Parse the bright
            newbright = Some(if arg.starts_with("+") {
                Bright::Increment(
                    arg[1..]
                        .parse()
                        .map_err(|e| AppError::Args(format!("invalid value ({e})")))?,
                )
            } else if arg.starts_with("-") {
                Bright::Decrement(
                    arg[1..]
                        .parse()
                        .map_err(|e| AppError::Args(format!("invalid value: ({e})")))?,
                )
            } else {
                Bright::Set(
                    arg.parse()
                        .map_err(|e| AppError::Args(format!("invalid value: ({e})")))?,
                )
            });
        } else if name.is_none() {
            name = Some(arg);
        } else {
            return Err(AppError::UnexpectedArg(arg));
        }
    }

    let newbright = newbright.ok_or(AppError::NoBright)?;

    // If directory not specified, iter over all
    if let Some(name) = name {
        let mut path = PathBuf::new();
        path.push("/sys/class/backlight");
        path.push(name);
        set_bright(path, newbright)?;
        // ^ this might actually be vulnerable
    } else {
        for dir in read_dir("/sys/class/backlight").map_err(AppError::ListBacklightDir)? {
            set_bright(dir.map_err(AppError::ListEntry)?.path(), newbright.clone())?;
        }
    }

    Ok(())
}

fn set_bright(name: PathBuf, newbright: Bright) -> Result<(), AppError> {
    let mut brightness = name.clone();
    brightness.push("brightness");
    let mut max_brightness = name.clone();
    max_brightness.push("max_brightness");

    let (actual, total) = (
        read_to_string(&brightness)
            .map_err(|e| AppError::ReadingFile(brightness.clone(), e))?
            .trim()
            .parse::<i64>()
            .map_err(|e| AppError::InvalidIntegerInFile(brightness.clone()))?,
        read_to_string(&max_brightness)
            .map_err(|e| AppError::ReadingFile(max_brightness.clone(), e))?
            .trim()
            .parse::<i64>()
            .map_err(|_| AppError::InvalidIntegerInFile(max_brightness))?,
    );

    let new = (match newbright {
        Bright::Set(val) => total as f64 * val / 100.0,
        Bright::Increment(val) => actual as f64 + total as f64 * val / 100.0,
        Bright::Decrement(val) => actual as f64 - total as f64 * val / 100.0,
    } as i64)
        .min(total)
        .max(0);

    // Save
    std::fs::write(&brightness, format!("{new}\n"))
        .map_err(|e| AppError::WritingFile(brightness, e))?;
    println!(
        "Brightness for {} updated: {:.1}%->{:.1}%, ({actual}->{new})",
        name.file_name()
            .map(|e| e.to_string_lossy())
            .unwrap_or("(unknown)".into()),
        actual as f64 * 100.0 / (total as f64),
        new as f64 * 100.0 / total as f64
    );

    Ok(())
}
