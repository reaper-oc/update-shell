use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub struct ShellConfig {
    pub prompt_style: String,
    pub color_user: String,
    pub color_host: String,
    pub color_path: String,
    pub color_git: String,
    pub color_exit: String,
    pub show_git: bool,
    pub show_exit_code: bool,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            prompt_style: "multiline".into(),
            color_user: "32".into(),
            color_host: "34".into(),
            color_path: "33".into(),
            color_git: "31".into(),
            color_exit: "31".into(),
            show_git: true,
            show_exit_code: true,
        }
    }
}

fn config_path() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
    base.join("updsh").join("env")
}

pub fn load_config() -> ShellConfig {
    let mut cfg = ShellConfig::default();
    let path = config_path();
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return cfg,
    };
    let mut vars: HashMap<String, String> = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            let k = k.trim().to_string();
            let v = v.trim().trim_matches('"').trim_matches('\'').to_string();
            vars.insert(k, v);
        }
    }
    if let Some(v) = vars.get("UPD_PROMPT_STYLE") {
        cfg.prompt_style = v.to_lowercase();
    }
    if let Some(v) = vars.get("UPD_COLOR_USER") { cfg.color_user = color_code(v); }
    if let Some(v) = vars.get("UPD_COLOR_HOST") { cfg.color_host = color_code(v); }
    if let Some(v) = vars.get("UPD_COLOR_PATH") { cfg.color_path = color_code(v); }
    if let Some(v) = vars.get("UPD_COLOR_GIT") { cfg.color_git = color_code(v); }
    if let Some(v) = vars.get("UPD_COLOR_EXIT") { cfg.color_exit = color_code(v); }
    if let Some(v) = vars.get("UPD_SHOW_GIT") { cfg.show_git = v == "1" || v == "yes" || v == "true"; }
    if let Some(v) = vars.get("UPD_SHOW_EXIT_CODE") { cfg.show_exit_code = v == "1" || v == "yes" || v == "true"; }
    for k in ["UPD_COLOR_USER", "UPD_COLOR_HOST", "UPD_COLOR_PATH", "UPD_COLOR_GIT", "UPD_COLOR_EXIT"] {
        if let Ok(val) = std::env::var(k) {
            if !val.is_empty() {
                let code = color_code(&val);
                match k {
                    "UPD_COLOR_USER" => cfg.color_user = code,
                    "UPD_COLOR_HOST" => cfg.color_host = code,
                    "UPD_COLOR_PATH" => cfg.color_path = code,
                    "UPD_COLOR_GIT" => cfg.color_git = code,
                    "UPD_COLOR_EXIT" => cfg.color_exit = code,
                    _ => {}
                }
            }
        }
    }
    if let Ok(val) = std::env::var("UPD_PROMPT_STYLE") { if !val.is_empty() { cfg.prompt_style = val.to_lowercase(); } }
    if let Ok(val) = std::env::var("UPD_SHOW_GIT") { cfg.show_git = val == "1" || val == "yes" || val == "true"; }
    if let Ok(val) = std::env::var("UPD_SHOW_EXIT_CODE") { cfg.show_exit_code = val == "1" || val == "yes" || val == "true"; }
    cfg
}

fn color_code(name: &str) -> String {
    match name {
        "black" | "30" => "30".into(),
        "red" | "31" => "31".into(),
        "green" | "32" => "32".into(),
        "yellow" | "33" => "33".into(),
        "blue" | "34" => "34".into(),
        "magenta" | "35" => "35".into(),
        "cyan" | "36" => "36".into(),
        "white" | "37" => "37".into(),
        "bred" | "1;31" => "1;31".into(),
        "bgreen" | "1;32" => "1;32".into(),
        "byellow" | "1;33" => "1;33".into(),
        "bblue" | "1;34" => "1;34".into(),
        "bmagenta" | "1;35" => "1;35".into(),
        "bcyan" | "1;36" => "1;36".into(),
        "bwhite" | "1;37" => "1;37".into(),
        _ => name.to_string(),
    }
}
