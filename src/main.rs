use clap::Parser;
use ini::Ini;
use rand::Rng;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use walkdir::WalkDir;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Storage::FileSystem::*;
use windows::Win32::System::Registry::*;
use windows::Win32::System::Diagnostics::ToolHelp::*;
use windows::Win32::System::Com::*;
use windows::Win32::UI::Shell::*;

// 嵌入资源文件
const PECMD_EXE: &[u8] = include_bytes!("../resources/PECMD.EXE");
const DENSE_CONTENT: &str = include_str!("../resources/Dense");

/// WinPE插件加载器
#[derive(Parser)]
#[command(name = "WinPE Plugin Loader")]
#[command(about = "WinPE插件加载器 - 支持CE、Edgeless、HPM插件", long_about = None)]
struct Cli {
    /// 操作命令或插件文件路径 (main / *.ce / *.7z / *.hpm)
    #[arg(value_name = "COMMAND_OR_FILE")]
    input: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    
    if let Some(input) = cli.input {
        let cmd = input.to_lowercase();
        
        if cmd == "main" {
            // 检查是否已经加载过
            if get_registry_value("LoadPlugins") == Some(1) {
                println!("插件已加载过，跳过重复加载");
                return;
            }
            
            println!("开始搜索并加载插件...");
            load_ce_plugins();
            load_edgeless_plugins();
            load_hpm_modules();
            
            // 标记已加载
            set_registry_value("LoadPlugins", 1);
            println!("所有插件加载完成");
        } else if cmd.ends_with(".ce") {
            // 加载指定CE插件
            let path = input.trim_matches('"');
            println!("加载CE插件: {}", path);
            load_specific_ce_plugin(path);
        } else if cmd.ends_with(".7z") {
            // 加载指定Edgeless插件
            let path = input.trim_matches('"');
            println!("加载Edgeless插件: {}", path);
            load_specific_edgeless_plugin(path, false);
        } else if cmd.ends_with(".hpm") {
            // 加载指定HPM模块
            let path = input.trim_matches('"');
            println!("加载HPM模块: {}", path);
            load_specific_hpm_module(path);
        } else {
            println!("错误：不支持的文件格式");
            println!("支持的格式: .ce .7z .hpm");
        }
    } else {
        println!("WinPE插件加载器");
        println!("用法:");
        println!("  winpe_plugin_loader.exe main              - 搜索并加载所有插件");
        println!("  winpe_plugin_loader.exe <plugin.ce>       - 加载指定CE插件");
        println!("  winpe_plugin_loader.exe <plugin.7z>       - 加载指定Edgeless插件");
        println!("  winpe_plugin_loader.exe <module.hpm>      - 加载指定HPM模块");
    }
}

// ==================== 注册表操作 ====================

fn get_registry_value(name: &str) -> Option<u32> {
    unsafe {
        let hkey = HKEY_CURRENT_USER;
        let subkey = w!("PEConfig");
        
        let mut result = HKEY::default();
        if RegOpenKeyExW(hkey, subkey, 0, KEY_READ, &mut result).is_ok() {
            let value_name = HSTRING::from(name);
            let mut data: u32 = 0;
            let mut data_size = std::mem::size_of::<u32>() as u32;
            
            let ret = RegQueryValueExW(
                result,
                &value_name,
                None,
                None,
                Some(&mut data as *mut u32 as *mut u8),
                Some(&mut data_size),
            );
            
            let _ = RegCloseKey(result);
            
            if ret.is_ok() {
                return Some(data);
            }
        }
    }
    None
}

fn set_registry_value(name: &str, value: u32) {
    unsafe {
        let hkey = HKEY_CURRENT_USER;
        let subkey = w!("PEConfig");
        
        let mut result = HKEY::default();
        let _ = RegCreateKeyW(hkey, subkey, &mut result);
        
        let value_name = HSTRING::from(name);
        let _ = RegSetValueExW(
            result,
            &value_name,
            0,
            REG_DWORD,
            Some(&value.to_le_bytes()),
        );
        
        let _ = RegCloseKey(result);
    }
}

fn get_ce_registry_value(name: &str) -> Option<u32> {
    unsafe {
        let hkey = HKEY_CURRENT_USER;
        let subkey = w!("CE-Helper");
        
        let mut result = HKEY::default();
        if RegOpenKeyExW(hkey, subkey, 0, KEY_READ, &mut result).is_ok() {
            let value_name = HSTRING::from(name);
            let mut data: u32 = 0;
            let mut data_size = std::mem::size_of::<u32>() as u32;
            
            let ret = RegQueryValueExW(
                result,
                &value_name,
                None,
                None,
                Some(&mut data as *mut u32 as *mut u8),
                Some(&mut data_size),
            );
            
            let _ = RegCloseKey(result);
            
            if ret.is_ok() {
                return Some(data);
            }
        }
    }
    None
}

fn set_ce_registry_value(name: &str, value: u32) {
    unsafe {
        let hkey = HKEY_CURRENT_USER;
        let subkey = w!("CE-Helper");
        
        let mut result = HKEY::default();
        let _ = RegCreateKeyW(hkey, subkey, &mut result);
        
        let value_name = HSTRING::from(name);
        let _ = RegSetValueExW(
            result,
            &value_name,
            0,
            REG_DWORD,
            Some(&value.to_le_bytes()),
        );
        
        let _ = RegCloseKey(result);
    }
}

// ==================== 磁盘扫描 ====================

fn get_all_drives() -> Vec<String> {
    let mut drives = Vec::new();
    unsafe {
        let mask = GetLogicalDrives();
        for i in 0..26 {
            if mask & (1 << i) != 0 {
                let drive = format!("{}:", (b'A' + i) as char);
                drives.push(drive);
            }
        }
    }
    drives
}

// ==================== 文件操作 ====================

fn ensure_pecmd_exists() {
    let pecmd_path = "X:\\Windows\\PECMD.EXE";
    if !Path::new(pecmd_path).exists() {
        if let Err(e) = fs::write(pecmd_path, PECMD_EXE) {
            println!("警告：无法写入PECMD.EXE: {}", e);
        }
    }
}

fn copy_directory(src: &str, dst: &str) -> std::io::Result<()> {
    let options = fs_extra::dir::CopyOptions::new().overwrite(true);
    fs_extra::dir::copy(src, dst, &options)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    Ok(())
}

fn copy_file(src: &str, dst: &str) -> std::io::Result<()> {
    if let Some(parent) = Path::new(dst).parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(src, dst)?;
    Ok(())
}

fn generate_random_string(length: usize) -> String {
    let mut rng = rand::thread_rng();
    let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
        .chars()
        .collect();
    (0..length)
        .map(|_| chars[rng.gen_range(0..chars.len())])
        .collect()
}

fn normalize_path(path: &str) -> String {
    // 规范化路径：将多个连续的反斜杠替换为单个
    let mut result = String::new();
    let mut prev_was_backslash = false;
    
    for ch in path.chars() {
        if ch == '\\' || ch == '/' {
            if !prev_was_backslash {
                result.push('\\');
                prev_was_backslash = true;
            }
        } else {
            result.push(ch);
            prev_was_backslash = false;
        }
    }
    
    result
}

// ==================== 进程检查 ====================

fn is_process_running(process_name: &str) -> bool {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot.is_err() {
            return false;
        }
        let snapshot = snapshot.unwrap();
        
        let mut pe32 = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        
        if Process32FirstW(snapshot, &mut pe32).is_ok() {
            loop {
                let exe_name = String::from_utf16_lossy(&pe32.szExeFile)
                    .trim_end_matches('\0')
                    .to_lowercase();
                if exe_name == process_name.to_lowercase() {
                    let _ = CloseHandle(snapshot);
                    return true;
                }
                if Process32NextW(snapshot, &mut pe32).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snapshot);
    }
    false
}

// ==================== 解压功能 ====================

fn extract_7z(archive_path: &str, output_dir: &str, password: Option<&str>) -> std::result::Result<(), String> {
    let archive_path = Path::new(archive_path);
    if !archive_path.exists() {
        return Err(format!("文件不存在: {}", archive_path.display()));
    }
    
    // 创建输出目录
    if let Err(e) = fs::create_dir_all(output_dir) {
        return Err(format!("无法创建输出目录: {}", e));
    }
    
    // 使用sevenz-rust库解压
    let result = if password.is_some() {
        // sevenz-rust 0.6 不直接支持密码，尝试使用基本解压
        sevenz_rust::decompress_file(archive_path, output_dir)
    } else {
        sevenz_rust::decompress_file(archive_path, output_dir)
    };
    
    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("解压失败: {:?}", e)),
    }
}

// ==================== 运行程序 ====================

fn run_program(program: &str, args: &str, hide: bool) {
    let mut cmd = Command::new(program);
    
    if !args.is_empty() {
        cmd.args(args.split_whitespace());
    }
    
    if hide {
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }
    }
    
    match cmd.spawn() {
        Ok(_) => println!("  ✓ 启动程序: {}", program),
        Err(e) => println!("  ✗ 启动程序失败 {}: {}", program, e),
    }
}

fn run_pecmd(script_path: &str) {
    ensure_pecmd_exists();
    
    let is_running = is_process_running("PECMD.EXE");
    let cmd = if is_running { "LOAD" } else { "MAIN" };
    
    let full_cmd = format!("PECMD.EXE {} \"{}\"", cmd, script_path);
    
    let mut command = Command::new("cmd");
    command.args(&["/C", &full_cmd]);
    
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    
    match command.spawn() {
        Ok(_) => println!("  ✓ 执行PECMD脚本: {}", script_path),
        Err(e) => println!("  ✗ 执行PECMD失败: {}", e),
    }
}

// ==================== 快捷方式创建 ====================

fn create_desktop_shortcut(
    name: &str,
    target: &str,
    args: &str,
    icon: &str,
    description: &str,
) {
    unsafe {
        // 初始化COM
        if CoInitializeEx(None, COINIT_APARTMENTTHREADED).is_err() {
            println!("  ✗ COM初始化失败");
            return;
        }
        
        // 获取桌面路径
        let desktop = SHGetKnownFolderPath(&FOLDERID_Desktop, KF_FLAG_DEFAULT, HANDLE::default());
        if desktop.is_err() {
            println!("  ✗ 获取桌面路径失败");
            CoUninitialize();
            return;
        }
        
        let desktop_path = desktop.unwrap();
        let desktop_str = desktop_path.to_string().unwrap_or_default();
        let shortcut_path = format!("{}\\{}.lnk", desktop_str, name);
        
        // 创建IShellLink对象
        let shell_link: Result<IShellLinkW> = CoCreateInstance(
            &ShellLink,
            None,
            CLSCTX_INPROC_SERVER,
        );
        
        if shell_link.is_err() {
            println!("  ✗ 创建IShellLink失败");
            CoUninitialize();
            return;
        }
        
        let shell_link = shell_link.unwrap();
        
        // 设置目标路径
        let target_wide = HSTRING::from(target);
        if shell_link.SetPath(&target_wide).is_err() {
            println!("  ✗ 设置目标路径失败");
            CoUninitialize();
            return;
        }
        
        // 设置参数
        if !args.is_empty() {
            let args_wide = HSTRING::from(args);
            let _ = shell_link.SetArguments(&args_wide);
        }
        
        // 设置工作目录（使用目标文件的目录）
        if let Some(parent) = Path::new(target).parent() {
            if let Some(parent_str) = parent.to_str() {
                let work_dir = HSTRING::from(parent_str);
                let _ = shell_link.SetWorkingDirectory(&work_dir);
            }
        }
        
        // 设置描述
        if !description.is_empty() {
            let desc_wide = HSTRING::from(description);
            let _ = shell_link.SetDescription(&desc_wide);
        }
        
        // 设置图标
        if !icon.is_empty() && Path::new(icon).exists() {
            let icon_wide = HSTRING::from(icon);
            let _ = shell_link.SetIconLocation(&icon_wide, 0);
        } else if Path::new(target).exists() {
            // 如果没有指定图标，使用目标程序的图标
            let target_wide = HSTRING::from(target);
            let _ = shell_link.SetIconLocation(&target_wide, 0);
        }
        
        // 获取IPersistFile接口
        let persist_file: Result<IPersistFile> = shell_link.cast();
        if persist_file.is_err() {
            println!("  ✗ 获取IPersistFile接口失败");
            CoUninitialize();
            return;
        }
        
        let persist_file = persist_file.unwrap();
        
        // 保存快捷方式
        let shortcut_path_wide = HSTRING::from(shortcut_path.clone());
        if persist_file.Save(&shortcut_path_wide, TRUE).is_err() {
            println!("  ✗ 保存快捷方式失败: {}", shortcut_path);
        } else {
            println!("  ✓ 创建快捷方式: {} -> {}", name, target);
        }
        
        CoUninitialize();
    }
}

// ==================== 壁纸设置 ====================

fn set_wallpaper(wallpaper_path: &str) {
    use windows::Win32::UI::WindowsAndMessaging::*;
    
    // 检查文件是否存在
    if !Path::new(wallpaper_path).exists() {
        println!("  ✗ 壁纸文件不存在: {}", wallpaper_path);
        return;
    }
    
    // 获取文件绝对路径
    let abs_path = if let Ok(canonical) = fs::canonicalize(wallpaper_path) {
        canonical.to_string_lossy().to_string()
    } else {
        wallpaper_path.to_string()
    };
    
    unsafe {
        // 将路径转换为UTF-16
        let wallpaper_wide = HSTRING::from(abs_path.clone());
        
        // 使用SystemParametersInfoW设置壁纸
        let result = SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            Some(wallpaper_wide.as_ptr() as *const u16 as *mut _),
            SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
        );
        
        if result.is_ok() {
            println!("  ✓ 设置壁纸: {}", abs_path);
        } else {
            println!("  ✗ 设置壁纸失败: {}", abs_path);
        }
    }
}

// ==================== INI配置解析 ====================

fn parse_ini_config(ini_path: &str) -> HashMap<String, HashMap<String, String>> {
    let mut result = HashMap::new();
    
    if let Ok(conf) = Ini::load_from_file(ini_path) {
        for (section, properties) in conf.iter() {
            if let Some(section_name) = section {
                let mut section_map = HashMap::new();
                for (key, value) in properties.iter() {
                    section_map.insert(key.to_string(), value.to_string());
                }
                result.insert(section_name.to_string(), section_map);
            }
        }
    }
    
    result
}

// ==================== CE插件加载 ====================

fn load_ce_plugins() {
    println!("\n=== 搜索CE插件 ===");
    
    if get_ce_registry_value("CELoad") == Some(1) {
        println!("CE插件已加载过，跳过");
        set_ce_registry_value("CELoad", 1);
        return;
    }
    
    let drives = get_all_drives();
    let mut ce_drive = String::new();
    
    for drive in &drives {
        if let Ok(entries) = fs::read_dir(format!("{}:\\ce-apps", drive)) {
            if entries.count() > 0 {
                ce_drive = drive.clone();  // 不包含冒号和反斜杠
                break;
            }
        }
    }
    
    if ce_drive.is_empty() {
        println!("未找到CE插件目录");
        set_ce_registry_value("CELoad", 1);
        return;
    }
    
    println!("找到CE插件目录: {}:\\", ce_drive);
    
    let ce_apps_dir = format!("{}:\\ce-apps", ce_drive);
    if let Ok(entries) = fs::read_dir(&ce_apps_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "ce" {
                    println!("  处理插件: {}", path.display());
                    process_ce_plugin(&path);
                }
            }
        }
    }
    
    set_ce_registry_value("CELoad", 1);
    println!("CE插件加载完成\n");
}

fn load_specific_ce_plugin(plugin_path: &str) {
    let path = Path::new(plugin_path.trim_matches('"'));
    if !path.exists() {
        println!("错误：插件文件不存在");
        return;
    }
    
    println!("加载CE插件...");
    process_ce_plugin(path);
    println!("CE插件加载完成");
}

fn process_ce_plugin(plugin_path: &Path) {
    let random_name = generate_random_string(15);
    let extract_dir = format!("X:\\Program Files\\CE-RAMOS\\{}", random_name);
    
    fs::create_dir_all(&extract_dir).ok();
    
    // 解压CE插件（带密码）
    match extract_7z(
        plugin_path.to_str().unwrap(),
        &extract_dir,
        Some("ZQFa08hkGv2@F"),
    ) {
        Ok(_) => {
            println!("  ✓ 解压成功");
            process_ce_plugin_content(&extract_dir);
        }
        Err(e) => {
            println!("  ✗ 解压失败: {}", e);
            fs::remove_dir_all(&extract_dir).ok();
        }
    }
}

fn process_ce_plugin_content(plugin_dir: &str) {
    // 处理lnk.cfg配置文件
    let cfg_path = format!("{}\\lnk.cfg", plugin_dir);
    if Path::new(&cfg_path).exists() {
        println!("  ✓ 找到配置文件");
        
        // 读取并替换%FN%
        if let Ok(content) = fs::read_to_string(&cfg_path) {
            // 确保 plugin_dir 以反斜杠结尾，避免路径拼接问题
            let plugin_dir_normalized = if plugin_dir.ends_with('\\') {
                plugin_dir.to_string()
            } else {
                format!("{}\\", plugin_dir)
            };
            
            // 替换 %FN%\ 和 %FN% （按这个顺序）
            let content = content
                .replace("%FN%\\", &plugin_dir_normalized)  // 先替换带反斜杠的
                .replace("%FN%", &plugin_dir_normalized);    // 再替换不带的
            
            // ⚠️ 关键修复：INI库会把反斜杠当作转义字符！
            // 必须将所有单反斜杠转义为双反斜杠，才能正确解析
            let content_escaped = content.replace("\\", "\\\\");
            
            // 解析配置
            let temp_cfg = format!("{}lnk_temp.cfg", plugin_dir_normalized);
            fs::write(&temp_cfg, content_escaped).ok();
            
            let config = parse_ini_config(&temp_cfg);
            
            for (section, props) in config.iter() {
                // 使用 starts_with 匹配 section 前缀，支持 [快捷方式], [快捷方式-1] 等格式
                if section.starts_with("复制目录") {
                    if let (Some(src), Some(dst)) = (props.get("源目录名称"), props.get("复制到目录名称")) {
                        println!("  • 复制目录: {} -> {}", src, dst);
                        copy_directory(src, dst).ok();
                    }
                } else if section.starts_with("复制文件") {
                    if let (Some(src), Some(dst)) = (props.get("源文件名称"), props.get("复制到文件名称")) {
                        println!("  • 复制文件: {} -> {}", src, dst);
                        copy_file(src, dst).ok();
                    }
                } else if section.starts_with("创建目录") {
                    if let Some(dir) = props.get("目录名称") {
                        println!("  • 创建目录: {}", dir);
                        fs::create_dir_all(dir).ok();
                    }
                } else if section.starts_with("运行程序") {
                    if let Some(program) = props.get("程序名称") {
                        let args = props.get("启动参数").map(|s| s.as_str()).unwrap_or("");
                        let hide = props.get("是否隐藏").map(|s| s == "0").unwrap_or(false);
                        println!("  • 运行程序: {}", program);
                        run_program(program, args, hide);
                    }
                } else if section.starts_with("快捷方式") {
                    if let Some(name) = props.get("快捷方式名称") {
                        let target = props.get("目标").map(|s| s.as_str()).unwrap_or("");
                        let args = props.get("参数文本").map(|s| s.as_str()).unwrap_or("");
                        let icon = props.get("图标文件").map(|s| s.as_str()).unwrap_or("");
                        let desc = props.get("备注").map(|s| s.as_str()).unwrap_or("");
                        
                        // 规范化路径（去除多余的反斜杠）
                        let target = normalize_path(target);
                        let icon = if !icon.is_empty() { normalize_path(icon) } else { icon.to_string() };
                        
                        create_desktop_shortcut(name, &target, args, &icon, desc);
                    }
                } else if section.starts_with("设置壁纸") {
                    if let Some(wallpaper) = props.get("壁纸文件名称") {
                        let wallpaper = normalize_path(wallpaper);
                        set_wallpaper(&wallpaper);
                    }
                }
            }
            
            fs::remove_file(&temp_cfg).ok();
        }
    }
    
    // 处理.reg文件
    process_reg_files(plugin_dir);
    
    // 处理.exe文件
    process_exe_files(plugin_dir);
    
    // 处理.bat文件
    process_bat_files(plugin_dir);
    
    // 处理.cmd文件
    process_cmd_files(plugin_dir);
    
    // 处理.ini文件（PECMD脚本）
    process_ini_files(plugin_dir);
    
    // 处理.wcs文件
    process_wcs_files(plugin_dir);
}

fn process_reg_files(dir: &str) {
    for entry in WalkDir::new(dir)
        .max_depth(1)
        .into_iter()
        .flatten()
    {
        if let Some(ext) = entry.path().extension() {
            if ext == "reg" {
                let path = entry.path().to_str().unwrap();
                println!("  • 导入注册表: {}", entry.file_name().to_str().unwrap());
                
                let mut cmd = Command::new("regedit");
                cmd.args(&["/s", path]);
                
                #[cfg(windows)]
                {
                    use std::os::windows::process::CommandExt;
                    const CREATE_NO_WINDOW: u32 = 0x08000000;
                    cmd.creation_flags(CREATE_NO_WINDOW);
                }
                
                cmd.spawn().ok();
            }
        }
    }
}

fn process_exe_files(dir: &str) {
    for entry in WalkDir::new(dir)
        .max_depth(1)
        .into_iter()
        .flatten()
    {
        if let Some(ext) = entry.path().extension() {
            if ext == "exe" && !entry.file_name().to_str().unwrap().eq_ignore_ascii_case("pecmd.exe") {
                let path = entry.path().to_str().unwrap();
                println!("  • 运行程序: {}", entry.file_name().to_str().unwrap());
                run_program(path, "", true);
            }
        }
    }
}

fn process_bat_files(dir: &str) {
    for entry in WalkDir::new(dir)
        .max_depth(1)
        .into_iter()
        .flatten()
    {
        if let Some(ext) = entry.path().extension() {
            if ext == "bat" {
                let path = entry.path().to_str().unwrap();
                println!("  • 运行批处理: {}", entry.file_name().to_str().unwrap());
                run_program(path, "", true);
            }
        }
    }
}

fn process_cmd_files(dir: &str) {
    for entry in WalkDir::new(dir)
        .max_depth(1)
        .into_iter()
        .flatten()
    {
        if let Some(ext) = entry.path().extension() {
            if ext == "cmd" {
                let path = entry.path().to_str().unwrap();
                println!("  • 运行命令: {}", entry.file_name().to_str().unwrap());
                run_program(path, "", true);
            }
        }
    }
}

fn process_ini_files(dir: &str) {
    ensure_pecmd_exists();
    
    for entry in WalkDir::new(dir)
        .max_depth(1)
        .into_iter()
        .flatten()
    {
        if let Some(ext) = entry.path().extension() {
            if ext == "ini" {
                let path = entry.path().to_str().unwrap();
                println!("  • 执行PECMD脚本: {}", entry.file_name().to_str().unwrap());
                run_pecmd(path);
            }
        }
    }
}

fn process_wcs_files(dir: &str) {
    ensure_pecmd_exists();
    
    for entry in WalkDir::new(dir)
        .max_depth(1)
        .into_iter()
        .flatten()
    {
        if let Some(ext) = entry.path().extension() {
            if ext.to_str().unwrap().eq_ignore_ascii_case("wcs") {
                let path = entry.path().to_str().unwrap();
                println!("  • 执行WCS脚本: {}", entry.file_name().to_str().unwrap());
                run_pecmd(path);
            }
        }
    }
}

// ==================== Edgeless插件加载 ====================

fn load_edgeless_plugins() {
    println!("\n=== 搜索Edgeless插件 ===");
    
    if get_ce_registry_value("EdgelessLoad") == Some(1) {
        println!("Edgeless插件已加载过，跳过");
        return;
    }
    
    let drives = get_all_drives();
    let mut edgeless_drive = String::new();
    
    for drive in &drives {
        let search_path = format!("{}:\\Edgeless\\Resource", drive);
        if Path::new(&search_path).exists() {
            edgeless_drive = drive.clone();  // 不包含冒号和反斜杠
            break;
        }
    }
    
    if edgeless_drive.is_empty() {
        println!("未找到Edgeless插件目录");
        set_ce_registry_value("EdgelessLoad", 1);
        return;
    }
    
    println!("找到Edgeless插件目录: {}:\\", edgeless_drive);
    
    let edgeless_dir = format!("{}:\\Edgeless\\Resource", edgeless_drive);
    let target_dir = "X:\\Program Files\\Edgeless";
    fs::create_dir_all(target_dir).ok();
    
    if let Ok(entries) = fs::read_dir(&edgeless_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "7z" {
                    println!("  处理插件: {}", path.display());
                    load_specific_edgeless_plugin(path.to_str().unwrap(), true);
                }
            }
        }
    }
    
    // 复制桌面
    copy_directory("X:\\Users\\Default\\Desktop", "X:\\Users\\Administrator\\Desktop").ok();
    
    set_ce_registry_value("EdgelessLoad", 1);
    println!("Edgeless插件加载完成\n");
}

fn load_specific_edgeless_plugin(plugin_path: &str, batch_mode: bool) {
    let path = Path::new(plugin_path.trim_matches('"'));
    if !path.exists() {
        println!("错误：插件文件不存在");
        return;
    }
    
    if !batch_mode {
        println!("加载Edgeless插件...");
    }
    
    let target_dir = "X:\\Program Files\\Edgeless";
    fs::create_dir_all(target_dir).ok();
    
    match extract_7z(plugin_path, target_dir, None) {
        Ok(_) => {
            println!("  ✓ 解压成功");
            process_edgeless_scripts(target_dir);
            
            if !batch_mode {
                copy_directory("X:\\Users\\Default\\Desktop", "X:\\Users\\Administrator\\Desktop").ok();
                println!("Edgeless插件加载完成");
            }
        }
        Err(e) => {
            println!("  ✗ 解压失败: {}", e);
        }
    }
}

fn process_edgeless_scripts(dir: &str) {
    ensure_pecmd_exists();
    
    // 处理WCS文件
    process_wcs_files(dir);
    
    // 处理BAT文件
    process_bat_files(dir);
    
    // 处理CMD文件
    process_cmd_files(dir);
    
    // 处理INI文件
    process_ini_files(dir);
}

// ==================== HPM模块加载 ====================

fn load_hpm_modules() {
    println!("\n=== 搜索HPM模块 ===");
    
    if get_ce_registry_value("HPMLoad") == Some(1) {
        println!("HPM模块已加载过，跳过");
        return;
    }
    
    let drives = get_all_drives();
    let mut hpm_drive = String::new();
    
    for drive in &drives {
        let search_path = format!("{}:\\HotPEModule", drive);
        if Path::new(&search_path).exists() {
            hpm_drive = drive.clone();  // 不包含冒号和反斜杠
            break;
        }
    }
    
    if hpm_drive.is_empty() {
        println!("未找到HPM模块目录");
        set_ce_registry_value("HPMLoad", 1);
        return;
    }
    
    println!("找到HPM模块目录: {}:\\", hpm_drive);
    
    let hpm_dir = format!("{}:\\HotPEModule", hpm_drive);
    let target_base = "X:\\Program Files\\HotPEModules";
    fs::create_dir_all(target_base).ok();
    
    if let Ok(entries) = fs::read_dir(&hpm_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext.to_str().unwrap().eq_ignore_ascii_case("hpm") {
                    println!("  处理模块: {}", path.display());
                    process_hpm_module(path.to_str().unwrap());
                }
            }
        }
    }
    
    set_ce_registry_value("HPMLoad", 1);
    println!("HPM模块加载完成\n");
}

fn load_specific_hpm_module(module_path: &str) {
    let path = Path::new(module_path.trim_matches('"'));
    if !path.exists() {
        println!("错误：模块文件不存在");
        return;
    }
    
    println!("加载HPM模块...");
    
    let target_base = "X:\\Program Files\\HotPEModules";
    fs::create_dir_all(target_base).ok();
    
    process_hpm_module(module_path);
    println!("HPM模块加载完成");
}

fn process_hpm_module(module_path: &str) {
    let tmp_dir = "X:\\Program Files\\HotPEModules\\tmp";
    fs::create_dir_all(tmp_dir).ok();
    
    match extract_7z(module_path, tmp_dir, None) {
        Ok(_) => {
            println!("  ✓ 解压成功");
            
            // 读取模块名称
            let config_path = format!("{}\\HPM.config", tmp_dir);
            let mut mod_name = "unknown".to_string();
            
            if let Ok(conf) = Ini::load_from_file(&config_path) {
                if let Some(section) = conf.section(Some("HPM_config")) {
                    if let Some(name) = section.get("mod_name") {
                        mod_name = name.to_string();
                    }
                }
            }
            
            // 重命名目录
            let random_suffix = generate_random_string(5);
            let final_dir = format!(
                "X:\\Program Files\\HotPEModules\\{}_{}",
                mod_name, random_suffix
            );
            
            if fs::rename(tmp_dir, &final_dir).is_ok() {
                // 删除Dense文件并写入新的
                let dense_file = format!("{}\\HPM.WCE.Dense", final_dir);
                fs::remove_file(&dense_file).ok();
                
                let wce_file = format!("{}\\HPM.WCE", final_dir);
                fs::write(&wce_file, DENSE_CONTENT).ok();
                
                // 确保PECMD.EXE存在
                let pecmd_path = format!("{}\\Pecmd.exe", final_dir);
                if !Path::new(&pecmd_path).exists() {
                    fs::write(&pecmd_path, PECMD_EXE).ok();
                }
                
                // 运行PECMD
                println!("  • 执行HPM模块");
                run_pecmd(&wce_file);
            }
        }
        Err(e) => {
            println!("  ✗ 解压失败: {}", e);
            fs::remove_dir_all(tmp_dir).ok();
        }
    }
}
