struct ShellReturn {
    err_code: u8,
    stdout: String,
    stderr: String
}

#[macro_export]
macro_rules! sh {
    ($($arg:tt)*) => {{
        let output = Command::new("sh")
            .arg("-c")
            .arg(format!($($arg)*))
            .output()
            .expect("failed to execute command");

        ShReturn {
            err_code: output.status.code().unwrap_or(1) as u8,
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        }
    }};
}
