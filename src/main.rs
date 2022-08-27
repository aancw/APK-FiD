// Copyright (c) 2022 Petruknisme
// 
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

extern crate clap;
extern crate zip;
extern crate colored;

use std::{io::Read};
use clap::Parser;
use colored::Colorize;
use std::io::prelude::*;


#[derive(Parser)]
#[clap(name = "APK-FiD")]
#[clap(author = "Petruknisme <me@petruknisme.com>")]
#[clap(version = "1.0")]
#[clap(about = "Give me your APK, I will give you framework name", long_about = None)]

struct Cli {
    /// Android APK file location
    #[clap(short, long)]
    file: String,
}

fn detect_framework(reader: impl Read + Seek) {
    let mut zip = zip::ZipArchive::new(reader).unwrap();
    let mut found = false;

    for i in 0..zip.len() {
        let file = zip.by_index(i).unwrap();
        
        if file.name() == "assets/index.android.bundle" {
            println!("{}", "[*] React Native Framework Detected!".green());
            found = true;
            break;
        } else if file.name().contains("res/raw/xml/config.xml") {
            println!("{}", "[*] Ionic Framework + Cordova Detected!".green());
            found = true;
            break;
        } else if file.name().contains("capacitor.config.json") {
            println!("{}", "[*] Ionic Framework + Capacitor Detected!".green());
            found = true;
            break;
        } else if file.name().contains("libflutter.so") || file.name().contains("libapp.so") || file.name().contains("flutter_assets")  {
            println!("{}", "[*] Flutter Framework Detected!".green());
            found = true;
            break;
        } else if file.name().contains("framework7.js") || file.name().contains("framework7.css") || file.name().contains("framework7-bundle.js")  {
            println!("{}", "[*] Framework7 Detected!".green());
            found = true;
            break;
        } else if file.name().contains("tsconfig.json") {
            println!("{}", "[*] NativeScript Framework Detected!".green());
            found = true;
            break;
        }
    }
    
    if !found {
        println!("{}", "[*] Framework is unknown or using Native Android Platform".green());
    }
}

fn main() {
    let cli = Cli::parse();
    let file_loc = cli.file;
    
    println!("{}", "
     /$$$$$$  /$$$$$$$  /$$   /$$       /$$$$$$$$ /$$ /$$$$$$$ 
    /$$__  $$| $$__  $$| $$  /$$/      | $$_____/|__/| $$__  $$
   | $$  \\ $$| $$  \\ $$| $$ /$$/       | $$       /$$| $$  \\ $$
   | $$$$$$$$| $$$$$$$/| $$$$$/ /$$$$$$| $$$$$   | $$| $$  | $$
   | $$__  $$| $$____/ | $$  $$|______/| $$__/   | $$| $$  | $$
   | $$  | $$| $$      | $$\\  $$       | $$      | $$| $$  | $$
   | $$  | $$| $$      | $$ \\  $$      | $$      | $$| $$$$$$$/
   |__/  |__/|__/      |__/  \\__/      |__/      |__/|_______/ 

   Give me your APK
   I will give you framework name
   by Petruknisme
    ".yellow());
    println!("{} {} {}",
                "[*] Using".blue(),
                file_loc.red(),
                "as input file ".blue() 
            );
    
    let zipfile = std::fs::File::open(&file_loc).unwrap();     
    println!("{}", "[*] Detecting the framework...".blue());
    detect_framework(&zipfile);
}