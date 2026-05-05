// Copyright (c) 2022 Petruknisme
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;
use std::fs::File;
use std::io::{Read, Seek};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "APK-FiD")]
#[command(author = "Petruknisme <me@petruknisme.com>")]
#[command(version)]
#[command(about = "Give me your APK, I will give you framework name")]
struct Cli {
    /// Android APK file location
    #[arg(short, long)]
    file: PathBuf,

    /// Output format: text or json
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    output: OutputFormat,

    /// Optional JSON file containing extra framework signatures
    #[arg(long)]
    rules: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Clone, Copy)]
struct Signal {
    needle: &'static str,
    score: u8,
}

#[derive(Clone, Copy)]
struct Rule {
    framework: &'static str,
    min_score: u8,
    signals: &'static [Signal],
}

#[derive(Clone, Debug)]
struct RuntimeSignal {
    needle: String,
    score: u8,
}

#[derive(Clone, Debug)]
struct RuntimeRule {
    framework: String,
    min_score: u8,
    signals: Vec<RuntimeSignal>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum ConfidenceTier {
    Low,
    Medium,
    High,
}

impl ConfidenceTier {
    fn from_percent(confidence_pct: u8) -> Self {
        if confidence_pct >= 80 {
            Self::High
        } else if confidence_pct >= 50 {
            Self::Medium
        } else {
            Self::Low
        }
    }
}

impl fmt::Display for ConfidenceTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
        }
    }
}

#[derive(Debug, Serialize)]
struct Detection {
    framework: String,
    score: u16,
    confidence_pct: u8,
    confidence_tier: ConfidenceTier,
    matched_signals: Vec<String>,
    matched_files: Vec<String>,
}

#[derive(Debug, Serialize)]
struct DetectionReport {
    detected: Vec<Detection>,
    unknown_or_native_android: bool,
    inspected_entries: usize,
    rules_loaded: usize,
    custom_rules_loaded: usize,
}

#[derive(Debug, Deserialize)]
struct CustomRulesFile {
    rules: Vec<CustomRule>,
}

#[derive(Debug, Deserialize)]
struct CustomRule {
    framework: String,
    min_score: u8,
    signals: Vec<CustomSignal>,
}

#[derive(Debug, Deserialize)]
struct CustomSignal {
    needle: String,
    score: u8,
}

const RULES: &[Rule] = &[
    Rule {
        framework: "React Native",
        min_score: 70,
        signals: &[
            Signal {
                needle: "assets/index.android.bundle",
                score: 70,
            },
            Signal {
                needle: "libreactnativejni.so",
                score: 20,
            },
            Signal {
                needle: "com/facebook/react",
                score: 20,
            },
        ],
    },
    Rule {
        framework: "Flutter",
        min_score: 80,
        signals: &[
            Signal {
                needle: "libflutter.so",
                score: 60,
            },
            Signal {
                needle: "flutter_assets",
                score: 50,
            },
            Signal {
                needle: "io/flutter/embedding",
                score: 20,
            },
        ],
    },
    Rule {
        framework: "Ionic + Cordova",
        min_score: 70,
        signals: &[
            Signal {
                needle: "res/xml/config.xml",
                score: 40,
            },
            Signal {
                needle: "www/cordova.js",
                score: 40,
            },
            Signal {
                needle: "www/cordova_plugins.js",
                score: 30,
            },
        ],
    },
    Rule {
        framework: "Ionic + Capacitor",
        min_score: 70,
        signals: &[
            Signal {
                needle: "assets/capacitor.config.json",
                score: 50,
            },
            Signal {
                needle: "capacitor.config.json",
                score: 40,
            },
            Signal {
                needle: "bridge.js",
                score: 20,
            },
        ],
    },
    Rule {
        framework: "Framework7",
        min_score: 50,
        signals: &[
            Signal {
                needle: "framework7.js",
                score: 50,
            },
            Signal {
                needle: "framework7.css",
                score: 40,
            },
            Signal {
                needle: "framework7-bundle.js",
                score: 50,
            },
        ],
    },
    Rule {
        framework: "NativeScript",
        min_score: 70,
        signals: &[
            Signal {
                needle: "tns_modules",
                score: 50,
            },
            Signal {
                needle: "com/tns/",
                score: 40,
            },
            Signal {
                needle: "nativescript.config",
                score: 30,
            },
        ],
    },
    Rule {
        framework: "Unity",
        min_score: 80,
        signals: &[
            Signal {
                needle: "libunity.so",
                score: 60,
            },
            Signal {
                needle: "assets/bin/Data/",
                score: 50,
            },
            Signal {
                needle: "globalgamemanagers",
                score: 40,
            },
        ],
    },
    Rule {
        framework: "Unreal Engine",
        min_score: 80,
        signals: &[
            Signal {
                needle: "libUE4.so",
                score: 60,
            },
            Signal {
                needle: "assets/UE4Game/",
                score: 50,
            },
            Signal {
                needle: ".pak",
                score: 30,
            },
        ],
    },
    Rule {
        framework: "Xamarin/.NET for Android",
        min_score: 80,
        signals: &[
            Signal {
                needle: "libmonodroid.so",
                score: 50,
            },
            Signal {
                needle: "assemblies/",
                score: 40,
            },
            Signal {
                needle: "Mono.Android.dll",
                score: 40,
            },
        ],
    },
    Rule {
        framework: "Cocos2d-x",
        min_score: 70,
        signals: &[
            Signal {
                needle: "libcocos2dcpp.so",
                score: 50,
            },
            Signal {
                needle: "assets/src/project.json",
                score: 40,
            },
            Signal {
                needle: "assets/main.js",
                score: 30,
            },
        ],
    },
    Rule {
        framework: "Apache Weex",
        min_score: 80,
        signals: &[
            Signal {
                needle: "libweexcore.so",
                score: 50,
            },
            Signal {
                needle: "assets/weex-main-jsfm.js",
                score: 40,
            },
            Signal {
                needle: "com/taobao/weex",
                score: 40,
            },
        ],
    },
    Rule {
        framework: "Qt for Android",
        min_score: 80,
        signals: &[
            Signal {
                needle: "libQt5Core.so",
                score: 50,
            },
            Signal {
                needle: "libQt6Core.so",
                score: 50,
            },
            Signal {
                needle: "assets/qt-reserved-files/",
                score: 40,
            },
            Signal {
                needle: "org/qtproject/qt",
                score: 40,
            },
        ],
    },
    Rule {
        framework: "Godot",
        min_score: 80,
        signals: &[
            Signal {
                needle: "libgodot_android.so",
                score: 50,
            },
            Signal {
                needle: "assets/data.pck",
                score: 40,
            },
            Signal {
                needle: "org/godotengine/godot",
                score: 40,
            },
        ],
    },
    Rule {
        framework: "Solar2D (Corona SDK)",
        min_score: 80,
        signals: &[
            Signal {
                needle: "libcorona.so",
                score: 50,
            },
            Signal {
                needle: "assets/resource.car",
                score: 40,
            },
            Signal {
                needle: "assets/main.lua",
                score: 40,
            },
        ],
    },
    Rule {
        framework: "Adobe AIR",
        min_score: 80,
        signals: &[
            Signal {
                needle: "libCore.so",
                score: 40,
            },
            Signal {
                needle: "assets/META-INF/AIR/",
                score: 50,
            },
            Signal {
                needle: "assets/air/",
                score: 40,
            },
        ],
    },
    Rule {
        framework: "Appcelerator Titanium",
        min_score: 80,
        signals: &[
            Signal {
                needle: "assets/Resources/",
                score: 40,
            },
            Signal {
                needle: "org/appcelerator/titanium",
                score: 50,
            },
            Signal {
                needle: "tiapp.xml",
                score: 40,
            },
        ],
    },
    Rule {
        framework: "Kivy / Python-for-Android",
        min_score: 80,
        signals: &[
            Signal {
                needle: "libpython",
                score: 40,
            },
            Signal {
                needle: "assets/private.mp3",
                score: 40,
            },
            Signal {
                needle: "org/kivy/android",
                score: 50,
            },
        ],
    },
    Rule {
        framework: "Defold",
        min_score: 80,
        signals: &[
            Signal {
                needle: "libdmengine.so",
                score: 50,
            },
            Signal {
                needle: "assets/game.arcd",
                score: 40,
            },
            Signal {
                needle: "assets/game.projectc",
                score: 40,
            },
        ],
    },
];

impl RuntimeRule {
    fn from_static(rule: &Rule) -> Self {
        Self {
            framework: rule.framework.to_string(),
            min_score: rule.min_score,
            signals: rule
                .signals
                .iter()
                .map(|signal| RuntimeSignal {
                    needle: signal.needle.to_string(),
                    score: signal.score,
                })
                .collect(),
        }
    }
}

fn builtin_rules() -> Vec<RuntimeRule> {
    RULES.iter().map(RuntimeRule::from_static).collect()
}

fn load_custom_rules(path: &Path) -> Result<Vec<RuntimeRule>> {
    let data = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read custom rule file: {}", path.display()))?;

    let custom: CustomRulesFile = serde_json::from_str(&data).with_context(|| {
        format!(
            "failed to parse custom rule file as JSON: {}",
            path.display()
        )
    })?;

    let mut rules = Vec::with_capacity(custom.rules.len());
    for rule in custom.rules {
        if rule.framework.trim().is_empty() {
            continue;
        }
        if rule.signals.is_empty() {
            continue;
        }

        let signals = rule
            .signals
            .into_iter()
            .filter(|signal| !signal.needle.trim().is_empty() && signal.score > 0)
            .map(|signal| RuntimeSignal {
                needle: signal.needle,
                score: signal.score,
            })
            .collect::<Vec<_>>();

        if signals.is_empty() {
            continue;
        }

        rules.push(RuntimeRule {
            framework: rule.framework,
            min_score: rule.min_score,
            signals,
        });
    }

    Ok(rules)
}

fn detect_frameworks(reader: impl Read + Seek, rules: &[RuntimeRule]) -> Result<DetectionReport> {
    let mut zip = zip::ZipArchive::new(reader).context("failed to read APK as ZIP archive")?;
    let mut names = Vec::with_capacity(zip.len());

    for i in 0..zip.len() {
        let file = zip
            .by_index(i)
            .with_context(|| format!("failed to read ZIP entry at index {i}"))?;
        names.push(file.name().to_string());
    }

    let detected = rules
        .iter()
        .filter_map(|rule| score_rule(rule, &names))
        .collect::<Vec<_>>();

    Ok(DetectionReport {
        unknown_or_native_android: detected.is_empty(),
        detected,
        inspected_entries: names.len(),
        rules_loaded: rules.len(),
        custom_rules_loaded: rules.len().saturating_sub(RULES.len()),
    })
}

fn score_rule(rule: &RuntimeRule, names: &[String]) -> Option<Detection> {
    let mut score = 0u16;
    let mut matched_signals = Vec::new();
    let mut matched_files = Vec::new();

    for signal in &rule.signals {
        let hit_files = names
            .iter()
            .filter(|name| name.contains(signal.needle.as_str()))
            .cloned()
            .collect::<Vec<_>>();

        if !hit_files.is_empty() {
            score += u16::from(signal.score);
            matched_signals.push(signal.needle.clone());
            matched_files.extend(hit_files);
        }
    }

    if score < u16::from(rule.min_score) {
        return None;
    }

    let max_score = rule
        .signals
        .iter()
        .map(|signal| u16::from(signal.score))
        .sum::<u16>();

    if max_score == 0 {
        return None;
    }

    let confidence_pct = ((score as f32 / max_score as f32) * 100.0).round() as u8;
    let matched_files = BTreeSet::<String>::from_iter(matched_files)
        .into_iter()
        .collect::<Vec<_>>();

    Some(Detection {
        framework: rule.framework.clone(),
        score,
        confidence_pct,
        confidence_tier: ConfidenceTier::from_percent(confidence_pct),
        matched_signals,
        matched_files,
    })
}

fn print_banner() {
    println!(
        "{}",
        "
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
    "
        .yellow()
    );
}

fn print_text_report(file_loc: &Path, report: &DetectionReport) {
    print_banner();
    println!(
        "{} {} {}",
        "[*] Using".blue(),
        file_loc.display().to_string().red(),
        "as input file".blue()
    );
    println!(
        "{} {} (custom: {})",
        "[*] Loaded rules:".blue(),
        report.rules_loaded,
        report.custom_rules_loaded
    );
    println!("{}", "[*] Detecting framework(s)...".blue());
    println!("{}", "[*] Possible Framework Detected:\n".blue());

    if report.unknown_or_native_android {
        println!(
            "{}",
            "[*] Framework is unknown or using Native Android Platform".red()
        );
        return;
    }

    let mut sorted = report.detected.iter().collect::<Vec<_>>();
    sorted.sort_unstable_by_key(|d| (std::cmp::Reverse(d.confidence_pct), d.framework.as_str()));

    for detection in sorted {
        println!(
            "{} (confidence: {}% [{}], score: {}, signals: {})",
            detection.framework.green(),
            detection.confidence_pct,
            detection.confidence_tier,
            detection.score,
            detection.matched_signals.join(", ")
        );

        if !detection.matched_files.is_empty() {
            println!("    evidence files: {}", detection.matched_files.join(", "));
        }
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    let zipfile = File::open(&cli.file)
        .with_context(|| format!("failed to open input file: {}", cli.file.display()))?;

    let mut rules = builtin_rules();
    if let Some(rule_path) = &cli.rules {
        let mut custom = load_custom_rules(rule_path)?;
        rules.append(&mut custom);
    }

    let report = detect_frameworks(zipfile, &rules)?;

    match cli.output {
        OutputFormat::Text => print_text_report(&cli.file, &report),
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&report).context("failed to encode json")?;
            println!("{json}");
        }
    }

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{} {}", "[!] Error:".red(), err);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write};
    use zip::write::FileOptions;

    fn build_apk(entries: &[&str]) -> Cursor<Vec<u8>> {
        let mut bytes = Cursor::new(Vec::new());
        {
            let mut writer = zip::ZipWriter::new(&mut bytes);
            let options = FileOptions::default();
            for entry in entries {
                writer
                    .start_file(*entry, options)
                    .expect("failed to start zip entry");
                writer
                    .write_all(b"x")
                    .expect("failed to write zip entry data");
            }
            writer.finish().expect("failed to finish zip writer");
        }
        bytes.set_position(0);
        bytes
    }

    fn test_rules() -> Vec<RuntimeRule> {
        builtin_rules()
    }

    #[test]
    fn detects_flutter_with_high_confidence() {
        let apk = build_apk(&[
            "lib/arm64-v8a/libflutter.so",
            "assets/flutter_assets/AssetManifest.json",
        ]);
        let report = detect_frameworks(apk, &test_rules()).expect("detection should succeed");

        let flutter = report
            .detected
            .iter()
            .find(|d| d.framework == "Flutter")
            .expect("flutter should be detected");

        assert!(flutter.confidence_pct >= 80);
        assert!(matches!(flutter.confidence_tier, ConfidenceTier::High));
    }

    #[test]
    fn avoids_false_positive_nativescript_from_tsconfig_only() {
        let apk = build_apk(&["assets/tsconfig.json"]);
        let report = detect_frameworks(apk, &test_rules()).expect("detection should succeed");

        assert!(report
            .detected
            .iter()
            .all(|d| d.framework != "NativeScript"));
    }

    #[test]
    fn detects_react_native() {
        let apk = build_apk(&["assets/index.android.bundle"]);
        let report = detect_frameworks(apk, &test_rules()).expect("detection should succeed");

        assert!(report
            .detected
            .iter()
            .any(|d| d.framework == "React Native"));
    }

    #[test]
    fn returns_unknown_when_no_signal_matches() {
        let apk = build_apk(&["classes.dex", "AndroidManifest.xml"]);
        let report = detect_frameworks(apk, &test_rules()).expect("detection should succeed");

        assert!(report.unknown_or_native_android);
        assert!(report.detected.is_empty());
    }

    #[test]
    fn detects_unity() {
        let apk = build_apk(&[
            "lib/arm64-v8a/libunity.so",
            "assets/bin/Data/globalgamemanagers",
        ]);
        let report = detect_frameworks(apk, &test_rules()).expect("detection should succeed");

        assert!(report.detected.iter().any(|d| d.framework == "Unity"));
    }

    #[test]
    fn detects_unreal_engine() {
        let apk = build_apk(&[
            "lib/arm64-v8a/libUE4.so",
            "assets/UE4Game/Content/Paks/game.pak",
        ]);
        let report = detect_frameworks(apk, &test_rules()).expect("detection should succeed");

        assert!(report
            .detected
            .iter()
            .any(|d| d.framework == "Unreal Engine"));
    }

    #[test]
    fn detects_xamarin_dotnet() {
        let apk = build_apk(&[
            "lib/arm64-v8a/libmonodroid.so",
            "assemblies/Mono.Android.dll",
        ]);
        let report = detect_frameworks(apk, &test_rules()).expect("detection should succeed");

        assert!(report
            .detected
            .iter()
            .any(|d| d.framework == "Xamarin/.NET for Android"));
    }

    #[test]
    fn detects_apache_weex() {
        let apk = build_apk(&["lib/arm64-v8a/libweexcore.so", "assets/weex-main-jsfm.js"]);
        let report = detect_frameworks(apk, &test_rules()).expect("detection should succeed");

        assert!(report.detected.iter().any(|d| d.framework == "Apache Weex"));
    }

    #[test]
    fn detects_qt_for_android() {
        let apk = build_apk(&[
            "lib/arm64-v8a/libQt6Core.so",
            "assets/qt-reserved-files/android_rcc_bundle.rcc",
        ]);
        let report = detect_frameworks(apk, &test_rules()).expect("detection should succeed");

        assert!(report
            .detected
            .iter()
            .any(|d| d.framework == "Qt for Android"));
    }

    #[test]
    fn detects_godot() {
        let apk = build_apk(&["lib/arm64-v8a/libgodot_android.so", "assets/data.pck"]);
        let report = detect_frameworks(apk, &test_rules()).expect("detection should succeed");

        assert!(report.detected.iter().any(|d| d.framework == "Godot"));
    }

    #[test]
    fn detects_solar2d() {
        let apk = build_apk(&["lib/arm64-v8a/libcorona.so", "assets/resource.car"]);
        let report = detect_frameworks(apk, &test_rules()).expect("detection should succeed");

        assert!(report
            .detected
            .iter()
            .any(|d| d.framework == "Solar2D (Corona SDK)"));
    }

    #[test]
    fn detects_adobe_air() {
        let apk = build_apk(&[
            "lib/arm64-v8a/libCore.so",
            "assets/META-INF/AIR/application.xml",
        ]);
        let report = detect_frameworks(apk, &test_rules()).expect("detection should succeed");

        assert!(report.detected.iter().any(|d| d.framework == "Adobe AIR"));
    }

    #[test]
    fn detects_appcelerator_titanium() {
        let apk = build_apk(&[
            "assets/Resources/app.js",
            "assets/tiapp.xml",
            "org/appcelerator/titanium/TiApplication.class",
        ]);
        let report = detect_frameworks(apk, &test_rules()).expect("detection should succeed");

        assert!(report
            .detected
            .iter()
            .any(|d| d.framework == "Appcelerator Titanium"));
    }

    #[test]
    fn detects_kivy_python_for_android() {
        let apk = build_apk(&["lib/arm64-v8a/libpython3.11.so", "assets/private.mp3"]);
        let report = detect_frameworks(apk, &test_rules()).expect("detection should succeed");

        assert!(report
            .detected
            .iter()
            .any(|d| d.framework == "Kivy / Python-for-Android"));
    }

    #[test]
    fn detects_defold() {
        let apk = build_apk(&["lib/arm64-v8a/libdmengine.so", "assets/game.projectc"]);
        let report = detect_frameworks(apk, &test_rules()).expect("detection should succeed");

        assert!(report.detected.iter().any(|d| d.framework == "Defold"));
    }

    #[test]
    fn includes_evidence_files() {
        let apk = build_apk(&[
            "lib/arm64-v8a/libflutter.so",
            "assets/flutter_assets/AssetManifest.json",
        ]);
        let report = detect_frameworks(apk, &test_rules()).expect("detection should succeed");

        let flutter = report
            .detected
            .iter()
            .find(|d| d.framework == "Flutter")
            .expect("flutter should be detected");

        assert!(flutter
            .matched_files
            .iter()
            .any(|f| f.contains("libflutter.so")));
    }

    #[test]
    fn loads_custom_rule_file() {
        let json = r#"{
            "rules": [
                {
                    "framework": "CustomFW",
                    "min_score": 60,
                    "signals": [
                        {"needle": "assets/customfw.json", "score": 60}
                    ]
                }
            ]
        }"#;

        let path =
            std::env::temp_dir().join(format!("apk_fid_custom_rules_{}.json", std::process::id()));

        std::fs::write(&path, json).expect("must write custom rule file");
        let rules = load_custom_rules(&path).expect("must parse custom rule file");
        std::fs::remove_file(&path).ok();

        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].framework, "CustomFW");
        assert_eq!(rules[0].signals[0].needle, "assets/customfw.json");
    }
}
