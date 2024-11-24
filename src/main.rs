extern crate winreg;
extern crate winapi;

use std::env;
use std::fs;
use std::io::{self};
use winreg::enums::*;
use winreg::RegKey;

/// Function to install the app as a service
fn install_service() -> io::Result<()> {
    let exe_path = env::current_exe()?.to_string_lossy().to_string();
    let install_path = "C:\\Program Files\\RustService";
    let service_path = format!("{install_path}\\rust_service.exe");

    // Copy the executable to the installation directory
    fs::create_dir_all(install_path)?;
    fs::copy(&exe_path, &service_path)?;

    println!("Installed application to: {}", service_path);

    // Add the app to startup
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu.create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run")?;
    key.set_value("RustService", &service_path)?;

    println!("Service added to startup.");
    Ok(())
}

/// Function to start the service logic
fn start_service_logic() -> io::Result<()> {
    println!("Service is running...");
    loop {
        // Simulate some background task
        std::thread::sleep(std::time::Duration::from_secs(10));
        println!("Service logic executed.");
    }
}

/// Uninstall the service
fn uninstall_service() -> io::Result<()> {
    let install_path = "C:\\Program Files\\RustService";
    let service_path = format!("{install_path}\\rust_service.exe");

    // Remove the registry key
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu.open_subkey_with_flags(
        "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
        KEY_SET_VALUE,
    )?;
    key.delete_value("RustService")?;

    // Remove the service files
    if fs::remove_file(&service_path).is_ok() {
        fs::remove_dir_all(install_path)?;
    }

    println!("Service uninstalled successfully.");
    Ok(())
}

fn main() -> io::Result<()> {

    let args: Vec<String> = env::args().collect();

    if args.contains(&"--install".to_string()) {
        install_service()?;
        println!("Service installed successfully.");
    } else if args.contains(&"--uninstall".to_string()) {
        uninstall_service()?;
        println!("Service uninstalled successfully.");
    } else {
        start_service_logic()?;
    }

    Ok(())
}
