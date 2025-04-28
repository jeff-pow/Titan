use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

const REPO_URL: &str = "https://raw.githubusercontent.com/jeff-pow/FileHydrant/main/";
const NET_NAME: &str = "net01.bin";

fn main() {
    let path = match env::var("EVALFILE") {
        Ok(evalfile) => {
            if PathBuf::from(&evalfile).is_relative() {
                Path::new(env!("CARGO_MANIFEST_DIR")).join(evalfile)
            } else {
                PathBuf::from(&evalfile)
            }
        }
        Err(_) => {
            if !Path::new(NET_NAME).exists() {
                println!("cargo:warning=Network not found. Downloading...");
                let status = Command::new("wget").arg(format!("{REPO_URL}/{NET_NAME}")).output();
                if status.is_err() {
                    panic!("Failed to download network");
                }
            }

            Path::new(env!("CARGO_MANIFEST_DIR")).join(NET_NAME)
        }
    };

    println!("cargo:rerun-if-env-changed=EVALFILE");
    println!("cargo:rerun-if-changed={NET_NAME}");
    println!("cargo:rustc-env=NETWORK={}", path.display());
}
