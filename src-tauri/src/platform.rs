use std::path::PathBuf;

pub fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .or_else(|| {
            let drive = std::env::var_os("HOMEDRIVE")?;
            let path = std::env::var_os("HOMEPATH")?;
            let mut home = PathBuf::from(drive);
            home.push(path);
            Some(home)
        })
}

pub fn home_dir_result() -> Result<PathBuf, String> {
    home_dir().ok_or_else(|| "Could not resolve the user home directory".to_string())
}

pub fn runtime_dir() -> Result<PathBuf, String> {
    Ok(home_dir_result()?.join(".copiwaifu"))
}

pub fn primary_port_file() -> Result<PathBuf, String> {
    Ok(runtime_dir()?.join("port"))
}

pub fn fallback_port_file() -> PathBuf {
    std::env::temp_dir().join("copiwaifu-port")
}
