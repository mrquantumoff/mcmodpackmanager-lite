#[allow(unused_imports)]
use directories::BaseDirs;

use slint::{ModelNotify, ModelRc, SharedString};
use std::{cell::RefCell, fs::create_dir_all, path::PathBuf};
mod vecmodel;
use vecmodel::VecModel;
slint::include_modules!();

static mut MODPACK: String = String::new();

fn set_modpack(value: String) {
    // unsafe {
    unsafe { MODPACK = value };
}
#[tokio::main]
async fn main() {
    let ui = AppWindow::new().expect("Failed to create a window");

    ui.set_modpacks(ModelRc::new(get_modpack_options().unwrap()));

    let ui_handle = ui.as_weak();
    ui.set_scale(ui.window().scale_factor());

    ui.on_select_modpack(move |value| set_modpack(value.to_string()));

    ui.on_apply(move || apply_modpack());
    ui.on_clear(move || clear_modpack());
    ui.on_reload(move || {
        ui_handle
            .unwrap()
            .set_modpacks(ModelRc::new(get_modpack_options().unwrap()))
    });

    ui.run().expect("Failed to run the app")
}

fn get_modpack_options() -> Result<VecModel<SharedString>, String> {
    let minecraftfolder: String;

    #[cfg(target_os = "linux")]
    {
        minecraftfolder = "~/.minecraft".to_string();
    }
    #[cfg(target_os = "windows")]
    {
        minecraftfolder = format!(
            "{}",
            BaseDirs::new()
                .expect("No base directories")
                .data_dir()
                .join(".minecraft")
                .to_str()
                .unwrap()
        );
    }
    #[cfg(target_os = "macos")]
    {
        minecraftfolder = "/Library/Application Support/minecraft".to_string();
    }
    let _mdpckpath = PathBuf::from(minecraftfolder).join("modpacks");
    // Check if the folder exists
    if !_mdpckpath.exists() {
        // Create a new folder
        let res = create_dir_all(&_mdpckpath);
        match res {
            Ok(_) => {}
            Err(_) => return Err("Failed to create modpack folder".to_string()),
        }
    }
    let readres = _mdpckpath.read_dir();
    let mut out: Vec<SharedString> = Vec::new();
    match readres {
        Ok(res) => {
            for entry in res {
                match entry {
                    Ok(entry) => {
                        if entry.path().is_dir() {
                            #[cfg(unix)]
                            {
                                let path = entry.path().to_str().unwrap().to_string();
                                let split: Vec<&str> = path.split('/').collect();
                                #[allow(clippy::cmp_owned)]
                                if split[split.len() - 1].to_string() != "free" {
                                    out.push(SharedString::from(
                                        split[split.len() - 1].to_string(),
                                    ));
                                }
                            }
                            #[cfg(windows)]
                            {
                                let path = entry.path().to_str().unwrap().to_string();
                                let split: Vec<&str> = path.split("\\").collect();
                                if split[split.len() - 1].to_string() != "free" {
                                    out.push(SharedString::from(
                                        split[split.len() - 1].to_string(),
                                    ));
                                }
                            }
                        }
                    }
                    Err(_) => return Err("Failed to read entry".to_string()),
                }
            }
        }
        Err(_) => return Err("Failed to read modpack folder".to_string()),
    }
    Ok(VecModel {
        array: RefCell::new(out),
        notify: ModelNotify::default(),
    })
}

fn clear_modpack() {
    set_modpack("free".to_string());
    apply_modpack();
}

fn apply_modpack() {
    let minecraftfolder: String;

    let modpack: String;
    unsafe { modpack = MODPACK.clone() }

    if modpack == String::new() {
        return;
    }

    #[cfg(target_os = "linux")]
    {
        minecraftfolder = "~/.minecraft".to_string();
    }
    #[cfg(target_os = "windows")]
    {
        minecraftfolder = format!(
            "{}",
            BaseDirs::new()
                .expect("No base directories")
                .data_dir()
                .join(".minecraft")
                .to_str()
                .unwrap()
        );
    }
    #[cfg(target_os = "macos")]
    {
        minecraftfolder = "/Library/Application Support/minecraft".to_string();
    }
    println!(
        "{} \n{}",
        &minecraftfolder,
        PathBuf::from(&minecraftfolder)
            .join("mods")
            .to_str()
            .unwrap()
    );
    let _mdpckpath = PathBuf::from(&minecraftfolder).join("modpacks");

    if PathBuf::from(&minecraftfolder).join("mods").exists() {
        let res = std::fs::remove_dir_all(PathBuf::from(&minecraftfolder).join("mods"));
        match res {
            Ok(_) => {}
            Err(_) => return,
        }
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(
            &_mdpckpath.join(modpack),
            PathBuf::from(minecraftfolder).join("mods"),
        )
        .unwrap();
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_dir(
            &_mdpckpath.join(modpack),
            PathBuf::from(minecraftfolder).join("mods"),
        )
        .unwrap();
    }
}
