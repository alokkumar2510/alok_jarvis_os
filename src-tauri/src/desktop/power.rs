use std::process::Command;
use std::io::{Result, Error, ErrorKind};

pub fn lock_workstation() -> Result<()> {
    println!("DesktopPower: Locking workstation...");
    
    // Attempt Win32 Native API call using windows crate
    unsafe {
        if windows::Win32::System::Shutdown::LockWorkStation().is_ok() {
            return Ok(());
        }
    }

    // Fallback to rundll32 if native API fails
    let status = Command::new("rundll32.exe")
        .args(["user32.dll,LockWorkStation"])
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(Error::new(ErrorKind::Other, "Failed to lock workstation via command fallback"))
    }
}

pub fn suspend_system() -> Result<()> {
    println!("DesktopPower: Suspending system to Sleep state...");

    // Call powrprof DLL via rundll32
    // Arguments: 0 (Hibernate = false, meaning Suspend/Sleep), 1 (Force = true), 0 (DisableWake = false)
    let status = Command::new("rundll32.exe")
        .args(["powrprof.dll,SetSuspendState", "0,1,0"])
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(Error::new(ErrorKind::Other, "Failed to suspend system"))
    }
}
