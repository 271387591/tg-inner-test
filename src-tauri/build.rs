use std::path::Path;
use std::{env, fs};
fn main() {
    // let config_path = Path::new("tauri.conf.json");
    // // 读取 `tauri.conf.json`
    // let tauri_conf_content =
    //     fs::read_to_string(config_path).expect("Failed to read tauri.conf.json");
    //
    // // 解析 JSON
    // let mut tauri_conf: serde_json::Value =
    //     serde_json::from_str(&tauri_conf_content).expect("Failed to parse tauri.conf.json");
    //
    // // 生成新的 `resources` 资源列表
    // #[cfg(target_os = "macos")]
    // let new_resources=vec!["app-macos", "runtime-macos"];
    // #[cfg(target_os = "windows")]
    // let new_resources=vec!["app-windows", "runtime-windows"];
    // // 修改 `resources` 字段
    // if let Some(bundle) = tauri_conf.get_mut("bundle") {
    //     bundle["resources"] = serde_json::json!(new_resources);
    // }
    //
    // // 序列化回字符串
    // let new_conf_content =
    //     serde_json::to_string_pretty(&tauri_conf).expect("Failed to serialize tauri.conf.json");
    //
    // // 只有内容变更时才写入，避免无意义写入
    // if new_conf_content != tauri_conf_content {
    //     fs::write(config_path, new_conf_content).expect("Failed to update tauri.conf.json");
    // }
    // 确保调用tauri_build::build()，这会设置必要的环境变量
    tauri_build::build();
}
