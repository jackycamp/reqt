#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in releaseO

extern crate anyhow;

use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::PathBuf;

fn init_db() -> Result<PathBuf> {
    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE")) // Fallback for Windows
        .map(PathBuf::from)
        .context("failed to determine home dir")?;

    let mut db_path = home.clone();
    db_path.push(".reqt");
    db_path.push("reqtdb");

    fs::create_dir_all(&db_path)?;
    sled::open(&db_path)?; // just open the db and do nothing? lmao

    Ok(db_path)
}

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
                        // let native_options = eframe::NativeOptions {
                        //     viewport: egui::ViewportBuilder::default()
                        //         .with_inner_size([1000.0, 800.0])
                        //         .with_min_inner_size([1000.0, 800.0])
                        //         .with_icon(
                        //             // NOTE: Adding an icon is optional
                        //             eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                        //                 .expect("Failed to load icon"),
                        //         ),
                        //     ..Default::default()
                        // };

    match init_db() {
        Ok(db_path) => {
            println!("db initialized {:?}", db_path);
        }
        Err(e) => {
            eprintln!("error initializing db: {:?}", e);
        }
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 800.0])
            .with_app_id("ligma"),
        ..Default::default()
    };
    eframe::run_native(
        "reqt",
        native_options,
        Box::new(|cc| Ok(Box::new(reqt::ReqtApp::new(cc)))),
    )
}
