#![deny(warnings)]

use crate::formula;
use crate::git;
use crate::util;
use crate::util::iso8601;

use eyre::Report;
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn install(package_name: &str) -> Result<(), Report> {
    let url = format!("https://{}-cask.git", package_name);

    let cwd = env::current_dir()?;

    let now = {
        let start = SystemTime::now();

        start.duration_since(UNIX_EPOCH)?
    };

    let dest_dir = env::temp_dir().join(format!("cask_{}", now.as_secs()));

    let option_target = match git::clone(&url, &dest_dir, vec![]) {
        Ok(()) => {
            let config_file_path = dest_dir.join("Cask.toml");

            if !config_file_path.exists() {
                return Err(eyre::format_err!(
                    "{} is not a valid formula!",
                    package_name
                ));
            }

            let config = formula::new(&config_file_path)?;

            let target = if cfg!(target_os = "macos") {
                config.darwin
            } else if cfg!(target_os = "windows") {
                config.windows
            } else if cfg!(target_os = "linux") {
                config.linux
            } else {
                fs::remove_dir_all(dest_dir)?;
                return Err(eyre::format_err!("not support your system"));
            };

            let hash_of_package = {
                let mut hasher = Sha256::new();

                hasher.update(package_name);
                format!("{:X}", hasher.finalize())
            };

            let mut package_dir = match dirs::home_dir() {
                Some(d) => Ok(d),
                None => Err(eyre::format_err!("can not found home dir")),
            }?;

            println!("{}", hash_of_package);

            package_dir = package_dir
                .join(".cask")
                .join("formula")
                .join(hash_of_package);

            println!("{:?}", &package_dir);

            if !&package_dir.exists() {
                fs::create_dir_all(&package_dir)?;
                fs::create_dir_all(&package_dir.join("bin"))?;
            }

            let contents = {
                let config_file = File::open(&config_file_path)?;
                let mut buf_reader = BufReader::new(&config_file);
                let mut file_content = String::new();
                buf_reader.read_to_string(&mut file_content)?;

                file_content
            };

            // write to a formula file
            {
                let file_path = package_dir.join("Cask.toml");

                let mut formula_file = {
                    if file_path.exists() {
                        File::open(&file_path)?
                    } else {
                        File::create(&file_path)?
                    }
                };

                formula_file.write_all(
                    format!(
                        r#"[cask]
package_name = "{}"
created_at = "{}"

"#,
                        package_name,
                        iso8601(&SystemTime::now())
                    )
                    .as_str()
                    .as_bytes(),
                )?;
                formula_file.write_all(contents.as_bytes())?;
            }

            target
        }
        Err(_) => {
            if dest_dir.exists() {
                fs::remove_dir_all(dest_dir)?;
            }
            process::exit(0x1);
        }
    };

    // remove cloned repo
    fs::remove_dir_all(dest_dir)?;

    let target = match option_target {
        Some(p) => Ok(p),
        None => Err(eyre::format_err!(
            "{} not support your system",
            package_name
        )),
    }?;

    let option_arch = if cfg!(target_arch = "x86") {
        target.x86
    } else if cfg!(target_arch = "x86_64") {
        target.x86_64
    } else if cfg!(target_arch = "arm") {
        target.arm
    } else if cfg!(target_arch = "aarch64") {
        target.aarch64
    } else if cfg!(target_arch = "mips") {
        target.mips
    } else if cfg!(target_arch = "mips64") {
        target.mips64
    } else if cfg!(target_arch = "mips64el") {
        target.mips64el
    } else {
        None
    };

    let arch = match option_arch {
        Some(a) => Ok(a),
        None => Err(eyre::format_err!("{} not support your arch", package_name)),
    }?;

    let dest = cwd.join("gpm.tar.gz");

    util::download(&arch.url, &dest).await?;

    Ok(())
}
