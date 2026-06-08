use std::process::Command;

pub struct VerificationEngine;

impl VerificationEngine {
    pub fn verify_step(verify_cmd: Option<&str>) -> bool {
        let cmd = match verify_cmd {
            Some(c) if !c.is_empty() => c,
            _ => return true, // Auto-verify success if no verify command exists
        };

        println!("VerificationEngine: Verifying step via cmd: '{}'", cmd);

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
                let success = out.status.success();
                let code = out.status.code().unwrap_or(-1);
                println!("VerificationEngine: Verification command finished with status: success={}, code={}", success, code);
                success
            }
            Err(e) => {
                eprintln!("VerificationEngine: Error running verification command: {:?}", e);
                false
            }
        }
    }
}
