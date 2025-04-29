pub mod starter;

use aes::Aes128;
use block_modes::{BlockMode, Cbc};
use block_padding::Pkcs7;
use futures_util::StreamExt;
use hex::{decode, encode};
use regex::Regex;
use reqwest::header::CONTENT_LENGTH;
use reqwest::Client;
use serde_json::Value;
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::process::Command;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use sysinfo::{Pid, System, SystemExt};
use tokio::fs::{self, File};
use tokio::io::{self, AsyncWriteExt, BufWriter};

static APP_KV: [u8; 32] = [
    50, 75, 11, 12, 37, 69, 55, 37, 56, 56, 17, 65, 56, 7, 50, 54, 37, 69, 3, 37, 65, 69, 37, 66,
    54, 115, 37, 69, 27, 47, 66, 19,
];
const VERSION_SECRET:&str = "b237de17a1144f42a8daf58218d87179";

pub async fn get_version_info(local_url: &str) -> anyhow::Result<Value> {
    // 目标 URL
    let url = format!("{}/get_version", local_url);
    // 发送 GET 请求并解析 JSON
    let response: Value = reqwest::get(url).await?.json().await?;
    return Ok(response);
}
pub async fn get_health(start_port: u16) -> bool {
    // 目标 URL
    let url = format!("http://127.0.0.1:{}/health", start_port);
    let handle = tauri::async_runtime::spawn(async move {
        let client = Client::new();
        match client.get(url.as_str()).send().await {
            Ok(response) => {
                println!("response status: {}", response.status());
                response.status().is_success()
            },
            Err(e) => {
                println!("Failed to connect: {:?}", e);
                false
            }
        }
    });
    handle.await.unwrap_or_else(|e| {
        println!("Task failed: {:?}", e);
        false
    })
}


pub fn get_start_port(port: u16) -> u16 {
    if is_port_in_use(port) {
        return get_start_port(port + 1);
    }
    return port;
}

pub fn auth_app() -> String {
    let s = get_disk_serial().unwrap_or("".to_string());
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let k_i = key_iv(APP_KV);
    encrypt(&format!("{}|{}", s, millis), k_i.0, k_i.1).unwrap()
}
pub fn is_port_in_use(port: u16) -> bool {
    let address = format!("127.0.0.1:{}", port);
    if let Ok(addr) = address.parse() {
        let socket_addr: SocketAddr = addr;
        if let Ok(_) = TcpStream::connect_timeout(&socket_addr, Duration::from_secs(1)) {
            return true;
        }
    }
    false
}
pub fn encrypt(data: &str, key: [u8; 16], iv: [u8; 16]) -> anyhow::Result<String> {
    // 创建 AES-128 CBC 加密器
    let cipher = Cbc::<Aes128, Pkcs7>::new_from_slices(&key, &iv)?;

    // 转换数据为字节
    let data_bytes = data.as_bytes();

    // 加密
    let ciphertext = cipher.encrypt_vec(data_bytes);

    // 转换为 Base64 以便传输
    Ok(encode(&ciphertext))
}
pub fn decrypt(data: &str, key: [u8; 16], iv: [u8; 16]) -> anyhow::Result<String> {
    let cipher = Cbc::<Aes128, Pkcs7>::new_from_slices(&key, &iv)?;
    let ciphertext = decode(data)?;
    let decrypted_data = cipher.decrypt_vec(&ciphertext)?;
    Ok(String::from_utf8(decrypted_data)?)
}
pub fn key_iv(key_iv: [u8; 32]) -> ([u8; 16], [u8; 16]) {
    let mut key: [u8; 16] = [0; 16];
    let mut iv: [u8; 16] = [0; 16];
    let mut key_index = 0;
    let mut iv_index = 0;
    for i in 0..key_iv.len() {
        if i % 2 == 0 {
            key[key_index] = key_iv[i];
            key_index = key_index + 1;
        } else {
            iv[iv_index] = key_iv[i];
            iv_index = iv_index + 1;
        }
    }
    (key, iv)
}
pub fn hex_encode(data: &str) -> String {
    encode(data)
}

#[cfg(target_os = "windows")]
pub fn get_disk_serial() -> Option<String> {
    let output = Command::new("cmd")
        .arg("/C")
        .arg("wmic")
        .arg("diskdrive")
        .arg("get")
        .arg("SerialNumber")
        .output()
        .ok()?;

    let serial = String::from_utf8_lossy(&output.stdout)
        .lines()
        .nth(1)
        .map(|s| s.trim().to_string())?;

    Some(serial)
}
#[cfg(target_os = "macos")]
pub fn get_disk_serial() -> Option<String> {
    let output = Command::new("diskutil")
        .arg("info")
        .arg("-plist")
        .arg("/")
        .output()
        .ok()?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    let re = Regex::new(r#"<key>VolumeUUID</key>\s*<string>([^<]+)</string>"#).unwrap();
    if let Some(caps) = re.captures(output_str.trim()) {
        if let Some(volume_uuid) = caps.get(1) {
            return Some(volume_uuid.as_str().to_string());
        }
    }
    Some("serial_number".to_string())
}
pub fn ip_to_int(ip: Ipv4Addr) -> u32 {
    let octets = ip.octets();
    u32::from(octets[0]) << 24
        | u32::from(octets[1]) << 16
        | u32::from(octets[2]) << 8
        | u32::from(octets[3])
}

pub fn int_to_ip(n: u32) -> Ipv4Addr {
    Ipv4Addr::new(
        (n >> 24) as u8,
        (n >> 16 & 0xFF) as u8,
        (n >> 8 & 0xFF) as u8,
        (n & 0xFF) as u8,
    )
}

pub async fn download_with_progress<F>(
    url: &str,
    file_path: &str,
    mut progress_callback: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnMut(f64, u64, u64),
{

    let object_key=    url.rsplit('/').next().unwrap_or("").to_string();
    let exp_time= SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()+300;
    let token=generate_hmac_token(&object_key,exp_time,VERSION_SECRET);
    let url=format!("{}?token={}&ts={}",url,token,exp_time);

    let client = Client::new();
    let response = client.get(url.as_str()).send().await?;

    let total_size = response
        .headers()
        .get(CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);

    println!("文件大小: {} bytes", total_size);

    // 🔥 **检查文件是否存在，存在则删除**
    if fs::try_exists(file_path).await? {
        fs::remove_file(file_path).await?;
        println!("已删除旧文件: {}", file_path);
    }

    let file = File::create(file_path).await?;
    let mut writer = BufWriter::new(file);

    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();
    let start_time = Instant::now();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        writer.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;

        // 计算进度并调用回调
        let percentage = if total_size > 0 {
            (downloaded as f64 / total_size as f64) * 100.0
        } else {
            0.0
        };

        progress_callback(percentage, downloaded, total_size);
    }

    writer.flush().await?;

    let elapsed_time = start_time.elapsed();
    println!("\n下载完成！耗时: {:.2?}", elapsed_time);

    Ok(())
}

pub fn kill_process(pid: u32) -> io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        // Windows: 使用 `taskkill`
        Command::new("taskkill")
            .args(&["/F", "/PID", &pid.to_string()]) // /F 强制终止
            .status()?; // 执行命令
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: 使用 `kill -9`
        Command::new("kill")
            .args(&["-9", &pid.to_string()])
            .status()?;
    }

    #[cfg(target_os = "linux")]
    {
        // Linux: 使用 `kill -9`
        Command::new("kill")
            .args(&["-9", &pid.to_string()])
            .status()?;
    }

    Ok(())
}

pub fn find_pid_running(pid: usize) -> bool {
    if pid <= 0 {
        return false;
    }
    let pid = Pid::from(pid); // 替换为你要检查的进程ID
    let mut system = System::new();
    system.refresh_process(pid);
    return system.process(pid).is_some();
}

type HmacSha256 = Hmac<Sha256>;
pub fn generate_hmac_token(object_key: &str, expiry: u64, secret: &str) -> String {
    let message = format!("{}{}", object_key, expiry);

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(message.as_bytes());

    let result = mac.finalize();
    let signature_bytes = result.into_bytes();
    // 直接转换为 hex，小写输出
    hex::encode(signature_bytes)
}
