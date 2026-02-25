use anyhow::Result;

#[cfg(target_os = "macos")]
pub fn register() -> Result<()> {
    use std::process::Command;

    cleanup_legacy_workflows();
    
    // 强制 macOS 服务进程刷新
    Command::new("/System/Library/CoreServices/pbs")
        .arg("-flush")
        .output()
        .ok();

    Ok(())
}

#[cfg(target_os = "macos")]
pub fn unregister() -> Result<()> {
    cleanup_legacy_workflows();
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn cleanup_legacy_workflows() {
    use std::fs;
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return,
    };
    for name in &["用TinyImage压缩", "TinyImage"] {
        let dir = home.join(format!("Library/Services/{}.workflow", name));
        if dir.exists() {
            fs::remove_dir_all(&dir).ok();
        }
    }
}

#[cfg(target_os = "windows")]
pub fn register() -> Result<()> {
    use winreg::enums::*;
    use winreg::RegKey;

    let exe = std::env::current_exe()?;
    let exe_path = exe.to_string_lossy().into_owned();
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    for ext in &["png", "jpg", "jpeg", "webp"] {
        let key_path = format!(
            r"Software\Classes\SystemFileAssociations\.{}\shell\TinyImage",
            ext
        );
        let (key, _) = hkcu.create_subkey(&key_path)?;
        key.set_value("", &"用TinyImage压缩")?;
        key.set_value("Icon", &format!("{},0", exe_path))?;

        let (cmd_key, _) = hkcu.create_subkey(format!(r"{}\command", key_path))?;
        cmd_key.set_value("", &format!("\"{}\" \"%1\"", exe_path))?;
    }

    Ok(())
}

#[cfg(target_os = "windows")]
pub fn unregister() -> Result<()> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    for ext in &["png", "jpg", "jpeg", "webp"] {
        let key_path = format!(
            r"Software\Classes\SystemFileAssociations\.{}\shell\TinyImage",
            ext
        );
        hkcu.delete_subkey_all(&key_path).ok();
    }

    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn register() -> Result<()> {
    anyhow::bail!("当前平台不支持右键菜单集成");
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn unregister() -> Result<()> {
    Ok(())
}
