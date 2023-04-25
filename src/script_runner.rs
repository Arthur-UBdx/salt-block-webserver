use std::process::Command;
use io;

pub fn run_js(program_file: &str) -> Result<String, io::Error> {
    let output = match Command::new("node")
        .arg(program_file)
        .output() {
            Ok(v) => v,
            _ => {
                return Err(&format!("Failed to run script {} ", program_file));
            }
        };

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn run_python(program_file: &str) -> Result<String, io::Error> {
    let output = match Command::new("python3")
        .arg(program_file)
        .output() {
            Ok(v) => v,
            _ => {
                return Err(&format!("Failed to run script {} ", program_file));
            }
        };

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
