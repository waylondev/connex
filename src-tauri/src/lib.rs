// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// 公共工具模块
mod utils;

// 负载测试特有工具模块
mod load_test_utils;

// 负载测试核心模块
mod load_test;

// 监控模块
mod monitoring;

/// 执行负载测试
#[tauri::command]
async fn run_load_test(config: load_test::Config) -> load_test::LoadTestResult {
    load_test::run(config).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, run_load_test])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}