use crate::github;
use crate::global::*;

use semver::Version;
#[cfg(not(windows))]
use std::{thread, time};

pub fn self_update_available() -> bool {
    let current_version: Version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
    let latest_version = github::latest_version(GH_OWNER, GH_REPO);

    current_version < latest_version
}

#[cfg(not(windows))]
pub fn run(_update_only: bool, _args: Vec<String>) {
    if self_update_available() {
        println!("A new version of the AlterWare launcher is available.");
        println!(
            "Download it at {}",
            github::latest_release_url(GH_OWNER, GH_REPO)
        );
        println!("Launching in 10 seconds..");
        thread::sleep(time::Duration::from_secs(10));
    }
}

#[cfg(windows)]
pub fn restart(args: Vec<String>) -> std::io::Error {
    use std::os::windows::process::CommandExt;
    match std::process::Command::new(std::env::current_exe().unwrap())
        .args(args.into_iter().skip(1))
        .creation_flags(0x00000010) // CREATE_NEW_CONSOLE
        .spawn()
    {
        Ok(_) => std::process::exit(0),
        Err(err) => err,
    }
}

#[cfg(windows)]
pub fn run(update_only: bool, args: Vec<String>) {
    use std::{fs, path::PathBuf};

    use crate::http;
    use crate::misc;

    let working_dir = std::env::current_dir().unwrap();
    let files = fs::read_dir(&working_dir).unwrap();

    for file in files {
        let file = file.unwrap();
        let file_name = file.file_name().into_string().unwrap();

        if file_name.contains("alterware-launcher")
            && (file_name.contains(".__relocated__.exe")
                || file_name.contains(".__selfdelete__.exe"))
        {
            fs::remove_file(file.path()).unwrap_or_else(|_| {
                println!("Failed to remove old launcher file.");
            });
        }
    }

    if self_update_available() {
        println!("Performing launcher self-update.");
        println!(
            "If you run into any issues, please download the latest version at {}",
            github::latest_release_url(GH_OWNER, GH_REPO)
        );

        let update_binary = PathBuf::from("alterware-launcher-update.exe");
        let file_path = working_dir.join(&update_binary);

        if update_binary.exists() {
            fs::remove_file(&update_binary).unwrap();
        }

        let launcher_name = if cfg!(target_arch = "x86") {
            "alterware-launcher-x86.exe"
        } else {
            "alterware-launcher.exe"
        };
        println!("{}", launcher_name);
        http::download_file(
            &format!(
                "{}/download/{}",
                github::latest_release_url(GH_OWNER, GH_REPO),
                launcher_name
            ),
            &file_path,
        );

        if !file_path.exists() {
            println!("Failed to download launcher update.");
            return;
        }

        self_replace::self_replace("alterware-launcher-update.exe").unwrap();
        fs::remove_file(&file_path).unwrap();

        // restarting spawns a new console, automation should manually restart on exit code 201
        if !update_only {
            let restart_error = restart(args).to_string();
            println!("Failed to restart launcher: {}", restart_error);
            println!("Please restart the launcher manually.");
            misc::stdin();
        }
        std::process::exit(201);
    }
}
