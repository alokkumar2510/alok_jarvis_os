use std::process::Command;

pub struct TaskExecutor;

impl TaskExecutor {
    pub fn execute_cmd(cmd: &str) -> Result<String, String> {
        println!("TaskExecutor: Executing shell command: '{}'", cmd);
        
        let output = if cfg!(target_os = "windows") {
            Command::new("powershell")
                .args(["-NoProfile", "-Command", cmd])
                .output()
        } else {
            Command::new("sh")
                .args(["-c", cmd])
                .output()
        };

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                if out.status.success() {
                    Ok(stdout)
                } else {
                    Err(format!("Exit code {}. Stderr: {}", out.status.code().unwrap_or(-1), stderr))
                }
            }
            Err(e) => Err(format!("Failed to start command process: {:?}", e)),
        }
    }
}
