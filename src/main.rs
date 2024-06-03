use std::{
    env::{self, current_exe}, error::Error, fs, path::Path, process::Command, thread, time::Duration
};
use winreg::{
    RegKey,
    enums::*,
};
use toml::value::Array;
use serde::Deserialize;
use fs_extra::dir::CopyOptions;
use serde_json::Value;

#[derive(Deserialize)]
struct Config {
    mods: Mods,
    settings: Settings,
}
#[derive(Deserialize)]
struct Settings {
    channel: String,
}
#[derive(Deserialize)]
struct Mods {
    directories: Array,
}

struct SoftwareKey {}
impl SoftwareKey {
    fn getvalue(path: &str, key: &str) -> String {
        let hkcu: RegKey = RegKey::predef(HKEY_CURRENT_USER);
        hkcu.open_subkey_with_flags(Path::new("Software").join(path), KEY_READ)
            .unwrap()
            .get_value(key)
            .unwrap()
    }
    fn setvalue(path: &str, key: &str, value: String) -> () {
        let hkcu: RegKey = RegKey::predef(HKEY_CURRENT_USER);
        hkcu.open_subkey_with_flags(Path::new("Software").join(path), KEY_WRITE)
            .unwrap()
            .set_value(key, &value)
            .unwrap()
    }
}

fn install() -> Result<(), Box<dyn Error>> {
    SoftwareKey::setvalue("Classes\\roblox-player\\shell\\open\\command", "", format!("\"{}\" %1",current_exe()?.to_str().unwrap()));
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Welcome to \x1b[1m\x1b[35mKurublox\x1b[0m!");

    let executable_path = Path::new(env::current_exe()?.as_os_str().to_str().unwrap()).parent().unwrap().to_str().unwrap().to_string();
    let config: Config = toml::from_str(fs::read_to_string(format!("{}\\Config.toml",executable_path)).expect("Failed to read config.").as_str()).expect("Failed to parse config.");
    let args: Vec<_> = env::args().collect();
    if args.len() > 1 && args[1].to_string().starts_with("roblox-player") {
        let roblox_path = SoftwareKey::getvalue("Classes\\roblox-player\\DefaultIcon", "");
        let roblox_directory = Path::new(&roblox_path).parent().unwrap().to_str().unwrap().to_string();

        println!("{}", "Checking for roblox updates...");
        let roblox_latest_version_data = minreq::get(format!("https://clientsettingscdn.roblox.com/v2/client-version/windowsplayer/channel/{}",config.settings.channel))
            .send()?
            .json::<Value>()?;
        let latest_roblox_version = roblox_latest_version_data["clientVersionUpload"].as_str().unwrap();
        let current_roblox_version = Path::new(roblox_directory.as_str()).file_name().unwrap().to_str().unwrap().to_string();
        if latest_roblox_version != current_roblox_version.as_str() {
            println!("Your version of Roblox is out of date. Downloading installer...");
            let roblox_installation_binary_content = minreq::get(format!("https://setup.rbxcdn.com/{latest_roblox_version}-Roblox.exe"))
                .send()?;
            let roblox_installation_binary = roblox_installation_binary_content
                .as_bytes(); 
            fs::write(format!("{executable_path}\\roblox-install.exe"), roblox_installation_binary)?;
            println!("Executing installer...");
            Command::new(format!("{executable_path}\\roblox-install.exe"))
                .spawn()?
                .wait()?;
            println!("Cleaning up...");
            fs::remove_file(format!("{executable_path}\\roblox-install.exe"))?;
            println!("Installing \x1b[1m\x1b[35mKurublox\x1b[0m...");
            install()?;
        } else {
            println!("Roblox is up to date.");
        }

        println!("{}","Applying modifications...");
        let mut options = CopyOptions::new();
        options.overwrite = true;
        for directory in config.mods.directories.iter() {
            if fs::metadata(format!("{executable_path}\\{}",directory.as_str().unwrap())).is_ok() && !fs::metadata(format!("{roblox_path}\\{}",directory.as_str().unwrap())).is_ok()  {
                println!("Copying {} to Roblox directory",directory.as_str().unwrap());
                fs_extra::dir::copy(format!("{executable_path}\\{}",directory.as_str().unwrap()),&roblox_directory,&options)?;
        } else {
                eprintln!("{executable_path}\\{} does not exist or already applied",directory.as_str().unwrap());
            }
        }

        println!("Starting Roblox...");
        Command::new(roblox_path)
            .arg(args[1].to_string())
            .spawn()?;
    } else {
        println!("Installing..."); 
        install()?;
        println!("Kurublox has been installed \x1b[1m\x1b[32msuccessfully\x1b[0m!")
    }
    thread::sleep(Duration::from_secs(5));
    Ok(())
}
