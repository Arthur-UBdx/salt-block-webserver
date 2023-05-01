use std::process::Command;

pub fn run_js(program_file: &str) -> Result<String, String> {
    let output = match Command::new("node")
    .arg(program_file) // replace with your Python script file name
    .output()
    {
        Ok(v) => v,
        _ => {
            return Err(String::from("failed to run script"));
        }
    };

    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout);
        return Ok(result.to_string());
    } else {
        let result = String::from_utf8_lossy(&output.stderr);
        return Err(result.to_string());
    }
}


pub fn run_python(program_file: &str) -> Result<String, String> {
    let output = match Command::new("python")
    .arg(program_file) // replace with your Python script file name
    .output()
    {
        Ok(v) => v,
        _ => {
            return Err(String::from("failed to run script"));
        }
    };

    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout);
        return Ok(result.to_string());
    } else {
        let result = String::from_utf8_lossy(&output.stderr);
        return Err(result.to_string());
    }
}


