#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
#[cfg(target_os = "windows")]
use windows::Win32::System::Threading::CREATE_NO_WINDOW;

use std::{env, fs};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use regex::Regex;
use crate::{tools, APP_NAME, APP_PATH};
static JAVA_CHILD: Lazy<Arc<Mutex<Option<Child>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));

const CFG_FILE:&str="TG-FF-BUNDLE.cfg";

#[cfg(all(target_os = "macos"))]
pub fn tg_start(jar_path:&str, res_path:&str, data_path:&str, java_cmd:&str, start_port:u16) -> bool {
    println!("find bundel info");
    let log_dir = env::temp_dir().to_string_lossy().to_string();
    let app_secret = tools::auth_app();
    #[cfg(debug_assertions)]
    {
        println!("app_secret-->{}", &app_secret);
        println!("start_port-->{}", start_port);
        println!("res_path-->{}", res_path);
        println!("data_path-->{}", data_path);
    }
    let app_update=PathBuf::from(data_path).join(APP_PATH).join(APP_NAME);
    if app_update.exists() {
        if let Some(cfg_file_dir)= PathBuf::from(jar_path).parent(){
            let cfg_file=cfg_file_dir.join(CFG_FILE);
            #[cfg(debug_assertions)]
            println!("cfg_file is : {:?}", &cfg_file);
            #[cfg(debug_assertions)]
            println!("app_update is : {:?}", &app_update);
            let _ = replace_app_classpath(cfg_file.to_str().unwrap(),app_update.to_str().unwrap());
        }
    }
    let child = Command::new(java_cmd)
        .current_dir(res_path)
        .env("BUNDLE_TEMP", &log_dir)
        .env("app.home", tools::hex_encode(data_path))
        .env("app.secret", app_secret)
        .env("app.port", start_port.to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    match child {
        Ok(mut child) => {
            println!("命令调用成功：");
            let mut java = JAVA_CHILD.lock().unwrap();
            *java = Some(child);


            // let stdout = child.stdout.take();
            // if let Some(out) = stdout {
            //     let reader = BufReader::new(out);
            //     tauri::async_runtime::spawn(async move {
            //         let mut lines = reader.lines();
            //         while let Some(Ok(line)) = lines.next() {
            //             let clean_output = String::from_utf8_lossy(line.as_bytes()).to_string();
            //             println!("java---{}", clean_output);
            //         }
            //     });
            // }
            // let stderr = child.stderr.take();
            // if let Some(err) = stderr {
            //     let reader = BufReader::new(err);
            //     tauri::async_runtime::spawn(async move {
            //         let mut lines = reader.lines();
            //
            //         while let Some(Ok(line)) = lines.next() {
            //             let clean_output = String::from_utf8_lossy(line.as_bytes()).to_string();
            //             println!("java--error---{}", clean_output);
            //         }
            //     });
            // }

            return true;
        }
        Err(err) => {
            println!("start error {:?}", err);
            return false;
        }
    }
}
fn replace_app_classpath(file_path: &str, new_value: &str) -> std::io::Result<()> {
    #[cfg(debug_assertions)]
    println!("replace_app_classpath file_path: {:?}", file_path);
    let content = fs::read_to_string(file_path)?;
    #[cfg(debug_assertions)]
    println!("replace_app_classpath content: {:?}", &content);
    if content.contains(new_value) { return Ok(()) }

    let re = Regex::new(r"(?m)^app\.classpath=.*$").unwrap();
    let new_line = format!("app.classpath={}", new_value);
    let updated = re.replace_all(&content, new_line);
    #[cfg(debug_assertions)]
    println!("replace_app_classpath updated: {:?}", &updated);
    fs::write(file_path, updated.as_bytes())?;
    Ok(())
}

#[cfg(all(target_os = "windows"))]
pub fn tg_start(jar_path:&str, res_path:&str, data_path:&str, java_cmd:&str, start_port:u16) -> bool {
    println!("find bundel info");
    let log_dir = env::temp_dir().to_string_lossy().to_string();
    let app_secret = tools::auth_app();
    #[cfg(debug_assertions)]
    {
        println!("app_secret-->{}", &app_secret);
        println!("start_port-->{}", start_port);
        println!("res_path-->{}", res_path);
        println!("data_path-->{}", data_path);
    }
    let child = Command::new(java_cmd)
        .current_dir(&res_path)
        .env("BUNDLE_TEMP", &log_dir)
        .env("app.home", tools::hex_encode(data_path))
        .env("app.secret", app_secret)
        .env("app.port", start_port.to_string())
        // .arg("-server")
        // .arg("-Xms512m")
        // .arg("-Xmx2g")
        // .arg("-XX:InitialCodeCacheSize=8m")//mac下独有，允许应用在 Hardened Runtime 模式下申请未签名的可执行内存区
        // .arg("-XX:ReservedCodeCacheSize=64m")
        // .arg("-XX:+UseG1GC")
        // .arg("-XX:+TieredCompilation")
        // .arg("-XX:+AlwaysPreTouch")
        // .arg("-XX:+UseStringDeduplication")
        // .arg("-XX:MaxInlineLevel=20")
        // .arg(format!("-XX:ErrorFile={}/hs_err_pid%p.log", log_dir))
        // .arg("-Djava.awt.headless=true")
        // .arg("-Duser.timezone=Asia/Shanghai")
        // .arg("-Dfile.encoding=UTF-8")
        // .arg(format!("-Dapp.home={}", tools::hex_encode(data_path))) // 进程名称
        //  // 文件编码
        // .arg("-Dmode=prod")
        // .arg(format!("-Dapp.secret={}", app_secret))
        // .arg(format!("-Dapp.port={}", start_port))
        // .arg("-jar")
        .arg(jar_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(CREATE_NO_WINDOW.0)
        .spawn();
    match child {
        Ok(mut child) => {
            println!("命令调用成功：");
            let mut java = JAVA_CHILD.lock().unwrap();
            *java = Some(child);

            // let stdout = child.stdout.take();
            // if let Some(out) = stdout {
            //     let reader = BufReader::new(out);
            //     tauri::async_runtime::spawn(async move {
            //         let mut lines = reader.lines();
            //         while let Some(Ok(line)) = lines.next() {
            //             let clean_output = String::from_utf8_lossy(line.as_bytes()).to_string();
            //             println!("java---{}", clean_output);
            //         }
            //     });
            // }
            // let stderr = child.stderr.take();
            // if let Some(err) = stderr {
            //     let reader = BufReader::new(err);
            //     tauri::async_runtime::spawn(async move {
            //         let mut lines = reader.lines();
            //
            //         while let Some(Ok(line)) = lines.next() {
            //             let clean_output = String::from_utf8_lossy(line.as_bytes()).to_string();
            //             println!("java--error---{}", clean_output);
            //         }
            //     });
            // }

            return true;
        }
        Err(err) => {
            println!("start error {:?}", err);
            return false;
        }
    }


    // match child {
    //     Ok(mut child) => {
    //         println!("命令调用成功：");
    //         // 处理标准输出流
    //         let stdout = child.stdout.take();
    //         if let Some(out) = stdout {
    //             let reader = BufReader::new(out);
    //             let app_handle_clone = app_handle_clone.clone();
    //
    //             tauri::async_runtime::spawn(async move {
    //                 let mut lines = reader.lines();
    //                 while let Ok(Some(line)) = lines.next_line().await {
    //                     let clean_output = String::from_utf8_lossy(line.as_bytes()).to_string();
    //                     // 发送输出到前端
    //                     app_handle_clone.emit("js_event", clean_output).unwrap();
    //                 }
    //             });
    //         }
    //         // 处理标准错误流
    //         let stderr = child.stderr.take();
    //         if let Some(err) = stderr {
    //             let reader = BufReader::new(err);
    //             let app_handle_clone = app_handle_clone.clone();
    //             tauri::async_runtime::spawn(async move {
    //                 let mut lines = reader.lines();
    //                 while let Ok(Some(line)) = lines.next_line().await {
    //                     let clean_output = String::from_utf8_lossy(line.as_bytes()).to_string();
    //                     // 发送输出到前端
    //                     app_handle_clone.emit("js_event", clean_output).unwrap();
    //                 }
    //             });
    //         }
    //         //检测启动
    //         tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    //         let pid = child.id().unwrap_or(0);
    //         if tools::find_pid_running(pid as usize) {
    //             println!("启动成功");
    //             let url = get_url().await;
    //             app_handle_clone
    //                 .emit("js_status", json!({"status":1,"url":&url}))
    //                 .unwrap();
    //             let stdin = child.stdin.take();
    //             if let Some(stdin) = stdin {
    //                 let mut guard = JAVA_STDIN.lock().await;
    //                 *guard = Some(stdin);
    //             }
    //             let mut java_process_info = JAVA_PROCESS.lock().await;
    //             *java_process_info = Some(JavaProcessInfo {
    //                 pid: child.id().unwrap(),
    //             });
    //             return;
    //         }
    //         app_handle_clone
    //             .emit("js_status", json!({"status":0}))
    //             .unwrap();
    //     }
    //     Err(err) => {
    //         app_handle_clone
    //             .emit("js_status", json!({"status":0}))
    //             .unwrap();
    //         println!("start error {:?}", err);
    //     }
    // }
}
/// 同步杀掉子进程（panic hook中调用）
pub fn kill_java_child() {
    let mut guard = JAVA_CHILD.lock().unwrap();
    if let Some(mut child) = guard.take() {
        if let Err(e) = child.kill() {
            eprintln!("❌ 杀子进程失败: {}", e);
        } else {
            println!("✅ 子进程已被杀死");
        }
        let _ = child.wait(); // 防止留下僵尸进程
    }
}

/// 检查全局Java子进程是否还活着
/// 检查tokio子进程是否活着
pub fn is_java_child_alive() -> bool {
    let mut child_guard = JAVA_CHILD.lock().unwrap();
    if let Some(child) = child_guard.as_mut() {
        match child.try_wait() {
            Ok(Some(_status)) => {
                // 子进程已经退出
                false
            }
            Ok(None) => {
                // 子进程还活着
                true
            }
            Err(e) => {
                // 出错，比如子进程句柄崩了
                eprintln!("⚠️ try_wait失败: {}", e);
                false
            }
        }
    } else {
        // 没有子进程对象
        false
    }
}

