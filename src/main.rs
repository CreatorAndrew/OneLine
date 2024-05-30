use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(target_family = "windows")]
mod windows {
    pub fn hide_console_window() {
        use std::ptr;
        use winapi::um::wincon::GetConsoleWindow;
        use winapi::um::winuser::{ShowWindow, SW_HIDE};
        let window = unsafe {GetConsoleWindow()};
        if window != ptr::null_mut() {
            unsafe {
                ShowWindow(window, SW_HIDE);
            }
        }
    }

    pub fn show_console_window() {
        use std::ptr;
        use winapi::um::wincon::GetConsoleWindow;
        use winapi::um::winuser::{ShowWindow, SW_SHOW};
        let window = unsafe {GetConsoleWindow()};
        if window != ptr::null_mut() {
            unsafe {
                ShowWindow(window, SW_SHOW);
            }
        }
    }
}

fn drive(path: String) -> String {
    let letters = [
        "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z"
    ];
    let mut drive = "".to_string();
    for i in 0..letters.len() {
        drive = " ".to_string() + letters[i] + ":";
        if path.contains(&drive) {
            break;
        }
    }
    return drive;
}

fn exists(line: String, segment: String, os_drive: &str, root_dir: &str) -> bool {
    if line.contains(":/") {
        return Path::new(
            &(
                line[line.rfind(":/").unwrap() - 1..].replace("\"", "").replacen(os_drive, root_dir, 1).to_string()
                + &segment.replacen(&drive(segment.to_string()), "", 1)
            )
        ).exists();
    }
    false
}

fn command_found(program: &str) -> bool {
    if let Ok(path_env) = env::var("PATH") {
        let delimiter;
        if env::consts::OS == "windows" {
            delimiter = ";";
        } else {
            delimiter = ":";
        }
        for path_dir in path_env.split(delimiter) {
            let path = format!("{}/{}", path_dir, program);
            if Path::new(&path).exists() {
                return true;
            }
        }
    }
    false
}

fn working_dir() -> io::Result<PathBuf> {
    let working_dir = env::current_dir()?;
    Ok(working_dir)
}

fn apply_working_dir(path: &str) -> String {
    let mut temp_path = path.replace("\\", "/");
    let mut exe_path = working_dir().expect("").display().to_string().replace("\\", "/");
    if env::consts::OS != "windows" {
        exe_path = "Z:".to_string() + &exe_path;
    }
    if temp_path == ".".to_string() {
        temp_path = exe_path;
    } else if temp_path == "..".to_string() {
        temp_path = exe_path[..exe_path.rfind("/").unwrap()].to_string();
    } else if temp_path.starts_with("./") {
        temp_path = temp_path.replacen("./", &(exe_path.to_string() + "/").as_str(), 1)
    } else if temp_path.starts_with("../") {
        temp_path = temp_path.replacen("../", &(exe_path[..exe_path.rfind("/").unwrap()].to_string() + "/").as_str(), 1)
    }
    return temp_path;
}

fn main() {
    let os_drive;
    let root_dir;
    if env::consts::OS == "windows" {
        #[cfg(target_family = "windows")]
        windows::hide_console_window();
        os_drive = "C:/";
        root_dir = "C:/";
    } else {
        os_drive = "Z:/";
        root_dir = "/";
    }
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);
    let mut concat = false;
    let mut end = false;
    let mut begin = -1;
    let mut index = 0;
    while index < args.len() {
        if args[index].contains("<concat>") {
            let mut count = 0;
            while args[index].contains("<concat>") {
                args[index] = args[index].replace("<concat>", "");
                concat = !concat;
                count += 1;
            }
            if !concat && count % 2 != 0 {
                end = true;
            } else {
                begin = index as i32;
            }
        }
        if (concat || end) && index as i32 != begin {
            args[begin as usize] = args.clone()[begin as usize].to_string() + &args.clone()[index].to_string();
            args.remove(index);
            end = false;
            index -= 1;
        }
        if concat {
            args[begin as usize] += " ";
        }
        index += 1;
    }
    for i in 0..args.len() {
        let mut line = "".to_string() + &args[i].replace("\\", "/");
        let temp_line = line.clone();
        let sublines:Vec<&str> = temp_line.split(" ").collect();
        line = "".to_string();
        for subline in sublines {
            line += &(apply_working_dir(subline) + " ").as_str();
        }
        let mut segments: Vec<String> = Vec::new();
        while line.contains("/") {
            segments.push(line[..line.find("/").unwrap()].to_string());
            line = line[line.find("/").unwrap() + 1..].to_string();
            if !line.contains("/") {
                segments.push(line[..line.len() - 1].to_string());
                line = "".to_string();
            }
        }
        for j in 0..segments.len() {
            let mut quote = false;
            if segments[j].contains(" ") {
                quote = exists(line.clone(), "".to_string() + segments[j].as_str(), os_drive, root_dir) || command_found(segments[j].as_str());
            }
            if quote {
                let temp_segment = "".to_string() + &segments[j];
                segments[j] = "\"".to_string() + &segments[j].replace(&drive("".to_string() + &segments[j]), "") + "\"";
                if temp_segment.contains(&drive("".to_string() + &temp_segment)) {
                    segments[j] += &drive("".to_string() + &temp_segment);
                }
            }
            line += &segments[j];
            if j < segments.len() - 1 {
                line += "/";
            }
        }
        line = line.replace(os_drive, root_dir);
        if env::consts::OS == "windows" {
            line = line.replace("/", "\\").replace(" \\", " /");
            if line.starts_with("start") {
                Command::new("cmd").arg("/c").arg(line.replacen("start", "", 1)).spawn().expect("");
            } else {
                let output = Command::new("cmd").arg("/c").arg(line).output().expect("");
                io::stdout().write_all(&output.stdout).unwrap();
                io::stdout().write_all(&output.stderr).unwrap();
            }
        } else {
            if line.ends_with("&") {
                Command::new("sh").arg("-c").arg(line.replacen("&", "", 1)).spawn().expect("");
            } else {
                let output = Command::new("sh").arg("-c").arg(line).output().expect("");
                io::stdout().write_all(&output.stdout).unwrap();
                io::stdout().write_all(&output.stderr).unwrap();
            }
        }
    }
    #[cfg(target_family = "windows")]
    windows::show_console_window();
}
