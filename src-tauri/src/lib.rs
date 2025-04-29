mod tools;

extern crate core;

use anyhow::anyhow;
use once_cell::sync::Lazy;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::{env, fs};
use sysinfo::{PidExt, System, SystemExt};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconEvent};
use tauri::{Emitter, Manager};
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_shell::ShellExt;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use std::process::{Child, Command};
use std::string::ToString;
#[cfg(target_os = "windows")]
use windows::Win32::System::Threading::CREATE_NO_WINDOW;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;



#[derive(Clone, Debug)]
struct JavaProcessInfo {
    pid: u32,
}
#[derive(Clone, Debug)]
struct JavaInfo {
    jar_path: String,
    jre_path: String,
    data_path: String,
    res_path: String,
}
const BUNDLE_PATH: &str = "tg-ff-inner";
pub const APP_PATH: &str = "app";

pub const  APP_NAME: &str = "tg-ff-inner.bundle";

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

static JAVA_INFO: Lazy<Arc<Mutex<Option<JavaInfo>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));
const BASE_PORT: u16 = 41918;
static START_PORT: Lazy<u16> = Lazy::new(|| tools::get_start_port(BASE_PORT));
#[tauri::command]
async fn tg_start(app_handle: tauri::AppHandle) -> Result<Value, String> {
    let url = get_url().await;
    if tools::starter::is_java_child_alive(){
        #[cfg(debug_assertions)]
        println!("进程已经在运行，无需重复启动");
        return Ok(json!({"status":1,"url":&url}));
    }
    #[cfg(debug_assertions)]
    println!("未找到进程已经开始运行");
    let java_info = JAVA_INFO.lock().unwrap().clone();
    let java_info = java_info.unwrap();

    println!("find bundel info");
    let jar_path = java_info.jar_path.clone();
    let res_path = java_info.res_path.clone();
    let data_path = java_info.data_path.clone();
    let java_cmd = java_info.jre_path.clone();
    let start_port = *START_PORT;
    #[cfg(debug_assertions)]
    {
        println!("start_port-->{}", start_port);
        println!("res_path-->{}", &res_path);
        println!("data_path-->{}", &data_path);
    }

    // 构建命令
    if tools::starter::tg_start(&jar_path,&res_path,&data_path,&java_cmd,start_port){
        Ok(json!({"status":1,"url":&url}))
    }else {
        Err("NOT_FOUND_BUNDLE_INFO".to_string())
    }
}
// 2. 注册panic钩子
pub fn register_panic_hook() {
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("⚡ Panic occurred: {:?}", panic_info);
        // Kill Java进程
        tools::starter::kill_java_child();
    }));
}

async fn get_url() -> String {
    let exist_start_port = *START_PORT;
    format!("http://localhost:{}/?fr=ta", exist_start_port.clone())
}
async fn do_update_move_file(update_file: &str) -> bool {
    let java_info = JAVA_INFO.lock().unwrap().clone();
    let java_info = java_info.unwrap();
    let data_path = java_info.data_path.clone();
    #[cfg(target_os = "windows")]
    let dest_file = PathBuf::from(&data_path).join(BUNDLE_PATH).join(APP_PATH).join(APP_NAME);
    #[cfg(target_os = "macos")]
    let dest_file = PathBuf::from(&data_path).join(APP_PATH).join(APP_NAME);

    let dest_file=dest_file.to_string_lossy().to_string();

    #[cfg(debug_assertions)]
    println!("do_update_move_file  update_file is : {:?}", update_file);
    let update_file_path = PathBuf::from(update_file);
    // 如果目标文件存在，先删除它
    if update_file_path.exists() {
        if tools::starter::is_java_child_alive() {
            tools::starter::kill_java_child();
            tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
        }
        if let Ok(e)=fs::exists(&dest_file) {
            if e {
                fs::remove_file(&dest_file).unwrap();
            }
        }
        let _ = fs::copy(update_file, &dest_file);
        let _ = fs::remove_file(update_file);
        #[cfg(debug_assertions)]
        println!("删除临时文件: {}", update_file);
    }
    return true;
}

#[tauri::command]
async fn tg_init(app: tauri::AppHandle) -> Result<Value, String> {
    let mut config = JAVA_INFO.lock().unwrap();
    if let Ok(java_info) = get_java_info(&app) {
        println!("bundle info: {:?}", java_info);
        *config = Some(java_info);
    } else {
        println!("not found bundle");
        return Err("NOT_FOUND_BUNDLE_INFO".to_string());
    }
    let os = std::env::consts::OS.to_lowercase(); // 获取操作系统名称
    let arch = std::env::consts::ARCH.to_lowercase(); // 获取 CPU 架构
    let os_detail = format!("{}-{}", os, arch);
    let start_port = *START_PORT;
    Ok(json!({"os_detail":os_detail,"core_current_version":APP_VERSION,"start_port":start_port}))
}
#[tauri::command]
async fn check_status(app: tauri::AppHandle) -> bool {
    let exist_start_port = *START_PORT;
    tools::get_health(exist_start_port).await
}
#[tauri::command]
async fn tg_stop(app: tauri::AppHandle) -> bool {
    if tools::starter::is_java_child_alive() {
        tools::starter::kill_java_child();
        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
    }
    app.exit(0);
    true
}

#[tauri::command]
async fn perform_update(app_handle: tauri::AppHandle, url: String) -> bool {
    let _ = app_handle.emit("js_update", json!({"step":0})).unwrap();
    let app_handle_clone = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        let java_info = JAVA_INFO.lock().unwrap().clone();
        let java_info = java_info.unwrap();
        let data_path = java_info.data_path.clone();
        let file_path = PathBuf::from(data_path).join("tmp.bin");
        let file_path = file_path.to_str().unwrap().to_string();
        let _ = tools::download_with_progress(&url,&file_path,|percentage, downloaded, total| {
            app_handle_clone.emit("js_update",json!({"step":1,"percentage":percentage,"downloaded":downloaded,"total":total,"file_path":&file_path})).unwrap();
        }).await;
    });
    true
}
#[tauri::command]
async fn do_perform_update(app_handle: tauri::AppHandle, update_file_path: String) -> bool {
    let file_path = PathBuf::from(&update_file_path);
    if !file_path.exists() {
        return false;
    }
    do_update_move_file(&update_file_path).await
}

async fn check_process_stopped(pid: u32) -> bool {
    loop {
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        let mut system = System::new_all();
        system.refresh_all();
        let sys_pid = sysinfo::Pid::from_u32(pid);
        if system.process(sys_pid).is_none() {
            println!("process id is :{}", pid);
            return true; // 进程已停止
        }
    }
}

fn get_java_info(app: &tauri::AppHandle) -> anyhow::Result<JavaInfo> {
    #[cfg(target_os = "windows")]
    {
        let resource_dir=env::current_dir()?;
        let res_path:String=resource_dir.to_str().unwrap().to_string();
        let jre_path = resource_dir.join(BUNDLE_PATH).join("TG-FF-BUNDLE.exe");
        let app_jar = resource_dir.join(BUNDLE_PATH).join(APP_PATH).join(APP_NAME);
        return Ok(JavaInfo {
            jre_path: jre_path.to_str().unwrap().to_string(),
            jar_path: app_jar.to_str().unwrap().to_string(),
            res_path: res_path.clone(),
            data_path: res_path.clone(),
        })
    }
    #[cfg(target_os = "macos")]{
        let resource_dir=app.path().resource_dir()?;
        let jre_path = resource_dir.parent().unwrap().join(BUNDLE_PATH).join("Contents").join("MacOS").join("TG-FF-BUNDLE");
        let app_jar = resource_dir.parent().unwrap().join(BUNDLE_PATH).join("Contents").join(APP_PATH).join(APP_NAME);

        let mac_data_path=app.path().data_dir()?;
        let data_dir=mac_data_path.join("TG-FF");
        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir)?;
        }
        let data_app_dir=data_dir.join(APP_PATH);
        if !data_app_dir.exists() {
            std::fs::create_dir_all(&data_app_dir)?;
        }
        let res_path:String=app.path().resource_dir()?.to_str().unwrap().to_string();

        return Ok(JavaInfo {
            jre_path: jre_path.to_str().unwrap().to_string(),
            jar_path: app_jar.to_str().unwrap().to_string(),
            res_path: res_path.clone(),
            data_path: data_dir.to_str().unwrap().to_string(),
        })
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    register_panic_hook();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            #[cfg(target_os = "macos")]{
                let quit_m = MenuItem::with_id(app, "show", "主界面", true, None::<&str>)?;
                let quit_i = MenuItem::with_id(app, "quit", "退出程序", true, None::<&str>)?;
                let menu = Menu::with_items(app, &[&quit_m, &quit_i])?;
                let tray_icon = tauri::tray::TrayIconBuilder::new()
                    .on_tray_icon_event(|tray, event| match event {
                        TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        } => {
                            // println!("left click pressed and released");
                            // in this example, let's show and focus the main window when the tray is clicked
                            let app = tray.app_handle();
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        _ => {
                            // println!("unhandled event {event:?}");
                        }
                    })
                    .menu(&menu)
                    .show_menu_on_left_click(true)
                    .on_menu_event(|app, event| match event.id.as_ref() {
                        "quit" => {
                            app.emit("js_exit", "1").unwrap();
                        }
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        _ => {
                            println!("menu item {:?} not handled", event.id);
                        }
                    })
                    .icon(app.default_window_icon().unwrap().clone())
                    .build(app)?;
            }




            // #[cfg(debug_assertions)] // 仅在调试(debug)版本中包含此代码
            // {
            //     let window = app.get_webview_window("main").unwrap();
            //     window.open_devtools();
            //     // window.close_devtools();
            //     // let url = Url::parse("http://localhost:19092/index-dev.html").unwrap();
            //     // window.navigate(url).unwrap(); // 加载开发页面
            // }

            Ok(())
        })
        .on_window_event(|window, event| {

            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                window.app_handle().emit("js_exit", "1").unwrap();

                // app.emit("js_exit", "1").unwrap();
                // let window = event.window();
                // window.hide().unwrap();  // <-- 改用 hide 隐藏窗口到托盘
                api.prevent_close(); // 阻止关闭事件
            }
        })
        .invoke_handler(tauri::generate_handler![
            perform_update,
            do_perform_update,
            tg_init,
            check_status,
            tg_start,
            tg_stop,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
