use std::{
    env, fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

const WINDOWS_TOOLS: [&str; 5] = [
    "CLINIC_OP",
    "cpuburn",
    "FurMark_win64",
    "LibreHardwareMonitorWrapper",
    "win-active",
];

const LINUX_TOOLS: [&str; 1] = [
    "linux_tools",
];

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let src_root = Path::new(&manifest_dir).join("externals");

    println!("cargo:rerun-if-changed={}", src_root.display());

    let out_dir = env::var("OUT_DIR").unwrap();
    let mut target_root = PathBuf::from(out_dir);
    // OUT_DIR is target/<profile>/build/<pkg>/out — pop 3 times to reach target/<profile>
    target_root.pop();
    target_root.pop();
    target_root.pop();
    let target_root = target_root.join("externals");

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let is_windows = target_os == "windows";

    if !src_root.is_dir() {
        println!("cargo:warning=externals directory not found at {}, skipping copy", src_root.display());
        return;
    }

    let walker = WalkDir::new(&src_root).into_iter().filter_entry(|e| {
        let file_name = e.file_name().to_str().unwrap_or("");

        if e.file_type().is_dir() {
            if is_windows && LINUX_TOOLS.contains(&file_name) {
                return false;
            }
            if !is_windows && WINDOWS_TOOLS.contains(&file_name) {
                return false;
            }
        }
        true
    });

    for entry in walker.filter_map(|e| e.ok()) {
        let path = entry.path();
        let relative_path = path.strip_prefix(&src_root).unwrap();
        let target_path = target_root.join(relative_path);

        if path.is_dir() {
            fs::create_dir_all(&target_path).ok();
        } else {
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent).ok();
            }
            fs::copy(path, &target_path).expect("Failed to copy file");
        }
    }
}

