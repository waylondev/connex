// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod logging;

fn main() {
    // 使用智能日志配置
    if let Err(e) = logging::init_smart() {
        eprintln!("Failed to initialize logging: {}", e);
        return;
    }
    
    connex_lib::run()
}
