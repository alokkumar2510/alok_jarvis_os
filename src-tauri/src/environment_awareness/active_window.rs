use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveWindowInfo {
    pub hwnd: isize,
    pub title: String,
    pub process_name: String,
}

pub fn capture() -> ActiveWindowInfo {
    unsafe {
        #[link(name = "user32")]
        extern "system" {
            fn GetForegroundWindow() -> isize;
            fn GetWindowTextW(hwnd: isize, lpString: *mut u16, nMaxCount: i32) -> i32;
            fn GetWindowThreadProcessId(hwnd: isize, lpdwprocessid: *mut u32) -> u32;
        }
        #[link(name = "kernel32")]
        extern "system" {
            fn OpenProcess(dwdesiredaccess: u32, binherithandle: i32, dwprocessid: u32) -> isize;
            fn CloseHandle(hobject: isize) -> i32;
            fn QueryFullProcessImageNameW(hprocess: isize, dwflags: u32, lpexeName: *mut u16, lpdwsize: *mut u32) -> i32;
        }

        let hwnd = GetForegroundWindow();
        if hwnd == 0 {
            return ActiveWindowInfo {
                hwnd: 0,
                title: "unknown".to_string(),
                process_name: "unknown".to_string(),
            };
        }

        let mut title_buf = [0u16; 512];
        let len = GetWindowTextW(hwnd, title_buf.as_mut_ptr(), 512);
        let title = if len > 0 {
            String::from_utf16_lossy(&title_buf[..len as usize])
        } else {
            "unknown".to_string()
        };

        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        let mut process_name = "unknown.exe".to_string();
        if pid != 0 {
            let process_handle = OpenProcess(0x1000, 0, pid); // PROCESS_QUERY_LIMITED_INFORMATION
            if process_handle != 0 {
                let mut size = 260;
                let mut buffer = vec![0u16; 260];
                if QueryFullProcessImageNameW(process_handle, 0, buffer.as_mut_ptr(), &mut size) != 0 {
                    let path = String::from_utf16_lossy(&buffer[..size as usize]);
                    if let Some(filename) = path.split('\\').last() {
                        process_name = filename.to_string();
                    }
                }
                CloseHandle(process_handle);
            }
        }

        ActiveWindowInfo {
            hwnd,
            title,
            process_name,
        }
    }
}
