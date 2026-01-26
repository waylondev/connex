// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod logging;

fn main() {
    // 使用专门的日志模块初始化
    if let Err(e) = logging::init() {
        eprintln!("Failed to initialize logging: {}", e);
        return;
    }
    
    connex_lib::run()
}
