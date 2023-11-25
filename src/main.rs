use std::env;
use std::fs;
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
    let letters = ["A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M",
                               "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z"];
    let mut drive = "".to_string();
    for i in 0..letters.len() {
        drive = " ".to_string() + letters[i] + ":";
        if path.contains(&drive) {
            break;
        }
    }
    return drive;
}

fn exists(line: String, seg: String, os_drive: &str, root_dir: &str) -> bool {
    if line.contains(":/") {
        return Path::new(&(line[line.rfind(":/").unwrap() - 1..].replace("\"", "").replacen(os_drive, root_dir, 1).to_string() +
                                &seg.replacen(&drive(seg.to_string()), "", 1))).exists();
    }
    false
}

fn command_found(program: &str) -> bool {
    if let Ok(path) = env::var("PATH") {
        let delimiter;
        if env::consts::OS == "windows" {
            delimiter = ";";
        } else {
            delimiter = ":";
        }
        for p in path.split(delimiter) {
            let p_str = format!("{}/{}", p, program);
            if fs::metadata(p_str).is_ok() {
                return true;
            }
        }
    }
    false
}

fn exe_path() -> io::Result<PathBuf> {
    let exe_path = env::current_exe()?;
    Ok(exe_path)
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
    let mut here = exe_path().expect("").display().to_string().replace("\\", "/");
    here = here[..here.rfind("/").unwrap()].to_string();
    if env::consts::OS != "windows" {
        here = "Z:".to_string() + &here;
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
        let mut line = "".to_string() + &args[i];
        let mut path = "".to_string() + &here;
        let mut prev = "".to_string();
        while line.contains("../") {
            line = line.replacen("../", "", 1);
            path = path[..path.rfind("/").unwrap()].to_string();
            prev += "../";
        }
        path += "/";
        if prev == "".to_string() {
            prev = "../".to_string();
        }
        line = "".to_string() + &args[i];
        line = line.replace("\\", "/").replacen(&prev, &path, 1).replace("./", &(here.to_string() + "/")) + "/";
        let mut segs: Vec<String> = Vec::new();
        while line.contains("/") {
            segs.push(line[..line.find("/").unwrap()].to_string());
            line = line[line.find("/").unwrap() + 1..].to_string();
            if !line.contains("/") {
                line = "".to_string();
            }
        }
        for j in 0..segs.len() {
            let mut quote = false;
            if segs[j].contains(" ") {
                quote = exists(line.clone(), "".to_string() + segs[j].as_str(), os_drive, root_dir) || command_found(segs[j].as_str());
            }
            if quote {
                let temp = "".to_string() + &segs[j];
                segs[j] = "\"".to_string() + &segs[j].replace(&drive("".to_string() + &segs[j]), "") + "\"";
                if temp.contains(&drive("".to_string() + &temp)) {
                    segs[j] += &drive("".to_string() + &temp);
                }
            }
            line += &segs[j];
            if j < segs.len() - 1 {
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
