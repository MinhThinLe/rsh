#[cfg(target_os = "linux")]
pub fn get_process_name(pid: u32) -> String {
    use std::fs;

    let string = fs::read_to_string(format!("/proc/{pid}/comm")).unwrap_or_default();
    string.trim().to_string()
}
