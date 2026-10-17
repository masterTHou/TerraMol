#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
use std::{io, path::PathBuf, time::Duration};
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut port = 0;
    let mut browser = true;
    let mut ready: Option<PathBuf> = None;
    let mut idle = 180;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--no-browser" => browser = false,
            "--port" => port = args.next().ok_or("missing port")?.parse()?,
            "--ready-file" => ready = Some(args.next().ok_or("missing ready file")?.into()),
            "--idle-timeout-seconds" => {
                idle = args.next().ok_or("missing idle timeout")?.parse::<u64>()?;
                if idle == 0 {
                    return Err("idle timeout must be positive".into());
                }
            }
            "--help" => {
                println!("TerraMol viewer [--no-browser] [--port N] [--ready-file PATH] [--idle-timeout-seconds N]");
                return Ok(());
            }
            _ => return Err(format!("unknown option: {arg}").into()),
        }
    }
    let server = mol_engine_core::viewer::Server::bind(port, Duration::from_secs(idle))?;
    let url = server.url();
    if let Some(path) = ready {
        std::fs::write(path, serde_json::to_vec(&serde_json::json!({"url":url}))?)?;
    }
    println!("TerraMol: {url}");
    if browser {
        open(&url)?;
    }
    server.serve()?;
    Ok(())
}
fn open(url: &str) -> io::Result<()> {
    #[cfg(windows)]
    let status = std::process::Command::new("rundll32.exe")
        .args(["url.dll,FileProtocolHandler", url])
        .status()?;
    #[cfg(target_os = "macos")]
    let status = std::process::Command::new("open").arg(url).status()?;
    #[cfg(all(unix, not(target_os = "macos")))]
    let status = std::process::Command::new("xdg-open").arg(url).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "Cannot open browser. Open this address manually: {url}"
        )))
    }
}
fn main() {
    if let Err(e) = run() {
        eprintln!("TerraMol could not start: {e}");
        #[cfg(windows)]
        {
            #[link(name = "user32")]
            unsafe extern "system" {
                fn MessageBoxW(
                    window: *mut std::ffi::c_void,
                    text: *const u16,
                    caption: *const u16,
                    kind: u32,
                ) -> i32;
            }
            let text: Vec<u16> = format!(
                "TerraMol 无法启动：\n{e}\n\n请重新解压完整的发行包，或将此信息反馈给项目。\0"
            )
            .encode_utf16()
            .collect();
            let title: Vec<u16> = "TerraMol\0".encode_utf16().collect();
            unsafe {
                MessageBoxW(std::ptr::null_mut(), text.as_ptr(), title.as_ptr(), 0x10);
            }
        }
        std::process::exit(1);
    }
}
