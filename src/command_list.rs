#![deny(warnings)]

use crate::cask;

use chrono::prelude::*;
use eyre::Report;
use tabled::{Style, Table, Tabled};

#[derive(Tabled)]
struct PackageInfo {
    name: String,
    bin: String,
    version: String,
    install_at: String,
}

pub async fn list(cask: &cask::Cask) -> Result<(), Report> {
    cask.init()?;

    let mut packages: Vec<PackageInfo> = vec![];

    for package in cask.list_formula()? {
        let cask_info = package.cask.ok_or_else(|| {
            eyre::format_err!(
                "can not parse cask property of package '{}'",
                package.package.name
            )
        })?;

        let create_at = DateTime::parse_from_str(&cask_info.created_at, "%+")
            .unwrap()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

        packages.push(PackageInfo {
            name: cask_info.name,
            bin: package.package.bin,
            version: cask_info.version,
            install_at: create_at,
        });
    }

    let table = Table::new(packages).with(Style::psql()).to_string();

    println!("{}", table);

    Ok(())
}
