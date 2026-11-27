use std::path::Path;

#[cfg(target_os = "linux")]
pub fn get_process_name(pid: u32) -> String {
    use std::fs;

    let string = fs::read_to_string(format!("/proc/{pid}/comm")).unwrap_or_default();
    string.trim().to_string()
}

pub fn change_directory(dest_dir: impl AsRef<Path>) {
    if let Err(e) = std::env::set_current_dir(dest_dir) {
        println!("{e}");
    }
}
