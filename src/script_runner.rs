use std::process::Command;
use std::path::PathBuf;

/// A function to run javascript code from a file,
/// 
/// # Example
/// helloworld.js:
/// ```js
/// console.log("Hello world")
/// ```
/// 
/// main.rs
/// ```
/// let filepath = "helloworld.js";
/// let output = run_js(filepath, "");
/// println!("{}", output);
/// ```
pub fn run_js(program_file: &str, args: &str) -> Result<String, String> {
    let path = PathBuf::from(program_file);
    let output = match Command::new("node")
    .arg(path)
    .arg(args)
    .output()
    {
        Ok(v) => v,
        Err(e) => {
            return Err(String::from(format!("Failed to run script \"{:?}\": {}", program_file, e)));
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

/// A function to run python code from a file,
/// 
/// # Example
/// helloworld.py:
/// ```python
/// print("Hello world")
/// ```
/// 
/// main.rs
/// ```
/// let filepath = "helloworld.py";
/// let output = run_python(filepath, "");
/// println!("{}", output);
/// ```
pub fn run_python(program_file: &str, args: &str) -> Result<String, String> {
    let path = PathBuf::from(program_file);
    let output = match Command::new("python3")
    .arg(path)
    .arg(args)
    .output()
    {
        Ok(v) => v,
        Err(e) => {
            return Err(String::from(format!("Failed to run script \"{:?}\": {}", program_file, e)));
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


