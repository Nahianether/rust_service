use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io::{self};
use std::ptr;
use winapi::um::processthreadsapi::{GetCurrentProcess, OpenProcessToken};
use winapi::um::securitybaseapi::GetTokenInformation;
// use winapi::um::shellapi::ShellExecuteW;
use std::io::Write;
use winapi::um::winnt::{TokenElevation, HANDLE, TOKEN_ELEVATION, TOKEN_QUERY};

/// Check if the application is running as an administrator
fn is_running_as_admin() -> bool {
    use std::mem;
    use winapi::shared::minwindef::FALSE;

    unsafe {
        let mut token: HANDLE = ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return false;
        }

        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut return_length = 0;

        let success = GetTokenInformation(
            token,
            TokenElevation,
            &mut elevation as *mut _ as *mut _,
            mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut return_length,
        );

        if success == FALSE {
            return false;
        }

        elevation.TokenIsElevated != 0
    }
}

/// Register the application as a scheduled task for elevated startup
fn register_task() -> io::Result<()> {
    let exe_path = env::current_exe()?.to_string_lossy().to_string();
    let task_name = "RustServiceTask";

    println!("Registering scheduled task with Task Scheduler...");

    // schtasks command to create a new task
    let output = std::process::Command::new("schtasks")
        .args(&[
            "/Create", // Create a new task
            "/SC", "ONLOGON", // Trigger: On user logon
            "/TN", task_name, // Task name
            "/TR", &exe_path, // Path to the executable
            "/RL", "HIGHEST", // Run with the highest privileges
        ])
        .output()?; // Capture the command output

    if !output.status.success() {
        eprintln!(
            "Failed to create scheduled task: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to register scheduled task. Please ensure the application is run as an administrator.",
        ));
    }

    println!("Scheduled task created successfully for elevated startup.");
    Ok(())
}

/// Unregister the scheduled task
fn unregister_task() -> io::Result<()> {
    let task_name = "RustServiceTask";

    let output = std::process::Command::new("schtasks")
        .args(&["/Delete", "/TN", task_name, "/F"])
        .output()?;

    if !output.status.success() {
        eprintln!(
            "Failed to delete scheduled task: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to unregister scheduled task",
        ));
    }

    println!("Task unregistered successfully.");
    Ok(())
}

/// Install the application as a service
fn install_service() -> io::Result<()> {
    let exe_path = env::current_exe()?.to_string_lossy().to_string();
    let install_path = "C:\\Program Files\\RustService";
    let service_path = format!("{install_path}\\rust_service.exe");

    fs::create_dir_all(install_path)?;
    fs::copy(&exe_path, &service_path)?;

    println!("Installed application to: {}", service_path);

    // Register the scheduled task for elevated startup
    register_task()?;

    println!("Service installed and configured for elevated startup.");
    Ok(())
}

/// Uninstall the service
fn uninstall_service() -> io::Result<()> {
    let install_path = "C:\\Program Files\\RustService";
    let service_path = format!("{install_path}\\rust_service.exe");

    // Unregister the scheduled task
    unregister_task()?;

    if fs::remove_file(&service_path).is_ok() {
        fs::remove_dir_all(install_path)?;
    }

    println!("Service uninstalled successfully.");
    Ok(())
}

fn log_message(message: &str) {
    let log_file = "C:\\Program Files\\RustService\\service_log.txt";
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_file) {
        let _ = writeln!(file, "{}", message);
    }
}

/// Main function
fn main() -> io::Result<()> {
    log_message("Application started.");

    if !is_running_as_admin() {
        println!("WARNING: The application is not running as an administrator.");
        println!("Please restart the application with administrative privileges.");
        std::thread::sleep(std::time::Duration::from_secs(120));
        std::process::exit(1); // Exit if not running as admin
    }

    log_message("The application is running as an administrator.");
    println!("The application is running as an administrator.");

    let args: Vec<String> = env::args().collect();
    if args.contains(&"--install".to_string()) {
        install_service()?; // Install and register the task
        println!("Service installed successfully.");
        log_message("Service installed successfully.");
    } else if args.contains(&"--uninstall".to_string()) {
        uninstall_service()?; // Uninstall and unregister the task
        println!("Service uninstalled successfully.");
        log_message("Service uninstalled successfully.");
    } else {
        println!("No valid arguments provided.");
        log_message("No valid arguments provided.");
    }

    Ok(())
}

//-----------------------------------------------------------------------------------

// use std::env;
// use windows_service::{
//     define_windows_service,
//     service::{
//         ServiceAccess, ServiceErrorControl, ServiceInfo,
//         ServiceStartType, ServiceState, ServiceType,
//     },
//     service_manager::{ServiceManager, ServiceManagerAccess},
//     Result,
// };

// const SERVICE_NAME: &str = "rust_service";
// const SERVICE_DISPLAY_NAME: &str = "Rust Background Service";
// const SERVICE_DESCRIPTION: &str = "A Rust-based Windows background service example.";

// fn main() -> Result<()> {
//     // Request administrative privileges
//     if !is_elevated() {
//         println!("This application requires administrative privileges.");
//         return Ok(());
//     }

//     // Detect if the app was launched to install the service or run the service
//     let args: Vec<String> = env::args().collect();
//     if args.len() > 1 && args[1] == "--service" {
//         // run_service()?;
//     } else {
//         install_or_start_service()?;
//     }

//     Ok(())
// }

// fn install_or_start_service() -> Result<()> {
//     // Check if the service is already installed
//     let service_manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)?;

//     match service_manager.open_service(SERVICE_NAME, ServiceAccess::QUERY_STATUS) {
//         Ok(service) => {
//             let status = service.query_status()?;
//             if status.current_state == ServiceState::Stopped {
//                 println!("Starting existing service...");
//                 service.start(&[""])?;
//             } else {
//                 println!("Service is already running.");
//             }
//         }
//         Err(_) => {
//             println!("Service not found. Installing...");

//             // Install the service
//             let exe_path = env::current_exe().map_err(|e| windows_service::Error::Winapi(e.into()))?.to_string_lossy().to_string();
//             let service_info = ServiceInfo {
//                 name: SERVICE_NAME.into(),
//                 display_name: SERVICE_DISPLAY_NAME.into(),
//                 service_type: ServiceType::OWN_PROCESS,
//                 start_type: ServiceStartType::AutoStart,
//                 error_control: ServiceErrorControl::Normal,
//                 executable_path: exe_path.into(),
//                 launch_arguments: vec!["--service".into()],
//                 dependencies: vec![],
//                 account_name: None, // Use the LocalSystem account
//                 account_password: None,
//             };

//             service_manager.create_service(&service_info, ServiceAccess::START)?;
//             service_manager.create_service(&service_info, ServiceAccess::START)?;
//             println!("Service installed successfully. Starting service...");

//             // Start the service
//             let service = service_manager.open_service(SERVICE_NAME, ServiceAccess::START)?;
//             let service = service_manager.open_service(SERVICE_NAME, ServiceAccess::START)?;
//         }
//     }

//     Ok(())
// }

// // fn run_service() -> windows_service::Result<()> {
// //     // Define the service logic
// //     define_windows_service!();

// //     ffi_service_main()
// // }

// fn my_service_main() {
//     // Your service logic goes here
//     println!("Service is running...");
//     loop {
//         std::thread::sleep(std::time::Duration::from_secs(10));
//     }
// }

// fn is_elevated() -> bool {
//     use std::ptr::null_mut;
//     use winapi::um::processthreadsapi::OpenProcessToken;
//     use winapi::um::securitybaseapi::GetTokenInformation;
//     use winapi::um::winnt::{TokenElevation, HANDLE, TOKEN_ELEVATION, TOKEN_QUERY};
//     use winapi::um::errhandlingapi::GetLastError;

//     unsafe {
//         let mut token_handle: HANDLE = null_mut();
//         if OpenProcessToken(winapi::um::processthreadsapi::GetCurrentProcess(), TOKEN_QUERY, &mut token_handle) == 0 {
//             return false;
//         }

//         let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
//         let mut return_length = 0;
//         let result = GetTokenInformation(
//             token_handle,
//             TokenElevation,
//             &mut elevation as *mut _ as *mut _,
//             std::mem::size_of::<TOKEN_ELEVATION>() as u32,
//             &mut return_length,
//         );

//         if result == 0 {
//             return false;
//         }

//         elevation.TokenIsElevated != 0
//     }
// }
