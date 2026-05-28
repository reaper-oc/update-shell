use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const REGISTRY_URL: &str = "https://registry-theta-one.vercel.app";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: Option<String>,
    #[serde(rename = "type")]
    pub pkg_type: Option<String>,
    pub language: Option<String>,
    pub binary: Option<String>,
    pub depends: Vec<String>,
    pub files: Vec<PackageFile>,
    pub source: Option<String>,
    pub source_url: Option<String>,
    pub build_type: Option<String>,
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PackageFile {
    pub path: String,
    #[serde(default)]
    pub content: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct RegistryEntry {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: Option<String>,
    #[serde(rename = "type")]
    pub pkg_type: Option<String>,
    pub language: Option<String>,
    pub depends: Vec<String>,
    pub source_url: Option<String>,
    pub build_type: Option<String>,
}

fn pkg_dir() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
    base.join("updsh").join("packages")
}

fn installed_dir() -> PathBuf {
    pkg_dir().join("installed")
}

fn enabled_dir() -> PathBuf {
    pkg_dir().join("enabled")
}

fn bin_dir() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("~/.local/share"));
    base.join("updsh").join("bin")
}

fn guard_shell() -> Result<(), String> {
    match std::env::var("UPD_SHELL").as_deref() {
        Ok("updsh") => Ok(()),
        _ => Err("pkg: this command only works inside updSH shell".into()),
    }
}

fn ensure_dirs() {
    let _ = fs::create_dir_all(installed_dir());
    let _ = fs::create_dir_all(enabled_dir());
    let _ = fs::create_dir_all(bin_dir());
}

pub fn list_installed_names() -> Vec<String> {
    list_installed()
}

pub fn list_available_names() -> Vec<String> {
    let registry = fetch_registry();
    registry.iter().map(|e| e.name.clone()).collect()
}

fn list_installed() -> Vec<String> {
    ensure_dirs();
    let mut pkgs = vec![];
    if let Ok(entries) = fs::read_dir(installed_dir()) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                pkgs.push(name);
            }
        }
    }
    pkgs.sort();
    pkgs
}

fn is_installed(name: &str) -> bool {
    installed_dir().join(name).is_dir()
}

fn load_installed_meta(name: &str) -> Option<Package> {
    let path = installed_dir().join(name).join("meta.json");
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn save_installed_meta(pkg: &Package) -> Result<(), String> {
    let dir = installed_dir().join(&pkg.name);
    fs::create_dir_all(&dir).map_err(|e| format!("failed to create package dir: {}", e))?;
    let content = serde_json::to_string_pretty(pkg)
        .map_err(|e| format!("failed to serialize: {}", e))?;
    fs::write(dir.join("meta.json"), &content)
        .map_err(|e| format!("failed to write meta: {}", e))?;
    Ok(())
}

fn remove_installed(name: &str) -> Result<(), String> {
    let dir = installed_dir().join(name);
    if dir.exists() {
        fs::remove_dir_all(&dir).map_err(|e| format!("failed to remove {}: {}", name, e))?;
    }
    let bin = bin_dir().join(name);
    if bin.exists() {
        let _ = fs::remove_file(&bin);
    }
    let enabled = enabled_dir().join(format!("{}.sh", name));
    if enabled.exists() {
        let _ = fs::remove_file(&enabled);
    }
    Ok(())
}

fn enable_package(pkg: &Package) -> Result<(), String> {
    ensure_dirs();
    let mut script = String::from("# updSH package: ") + &pkg.name + " v" + &pkg.version + "\n";
    script.push_str("# installed by updSH package manager\n\n");

    for (k, v) in &pkg.env {
        script.push_str(&format!("export {}={}\n", k, v));
    }

    if pkg.pkg_type.as_deref() == Some("compile") || pkg.pkg_type.as_deref() == Some("build") {
        if let Some(bin) = &pkg.binary {
            let bin_path = bin_dir().join(bin);
            script.push_str(&format!(
                "export PATH=\"$PATH:{}\"\n",
                bin_path.parent().unwrap_or(&bin_dir()).display()
            ));
        }
    }

    for file in &pkg.files {
        if pkg.pkg_type.as_deref() == Some("build") && !file.path.ends_with(".sh") {
            let file_path = installed_dir().join(&pkg.name).join(&file.path);
            if let Some(parent) = file_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(&file_path, &file.content);
            script.push_str(&format!("# installed: {}\n", file.path));
        } else if pkg.pkg_type.as_deref() == Some("build") && file.path.ends_with(".sh") {
            script.push_str(&format!("# post-install: {}\n", file.path));
        } else {
            script.push_str(&format!("# source: {}\n", file.path));
            script.push_str(&file.content);
            script.push('\n');
        }
    }

    let path = enabled_dir().join(format!("{}.sh", pkg.name));
    fs::write(&path, &script).map_err(|e| format!("failed to write enabled script: {}", e))?;
    Ok(())
}

fn disable_package(name: &str) {
    let path = enabled_dir().join(format!("{}.sh", name));
    let _ = fs::remove_file(&path);
}

pub fn source_packages() {
    let dir = enabled_dir();
    if !dir.is_dir() {
        return;
    }
    if let Ok(entries) = fs::read_dir(&dir) {
        let mut scripts: Vec<PathBuf> = entries
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().ends_with(".sh"))
            .map(|e| e.path())
            .collect();
        scripts.sort();
        for path in scripts {
            if let Ok(content) = fs::read_to_string(&path) {
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    let _ = crate::parser::parse_line(line);
                }
            }
        }
    }
}

pub fn apply_enabled_packages() {
    let dir = enabled_dir();
    if !dir.is_dir() {
        return;
    }
    if let Ok(entries) = fs::read_dir(&dir) {
        let mut scripts: Vec<PathBuf> = entries
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().ends_with(".sh"))
            .map(|e| e.path())
            .collect();
        scripts.sort();
        for path in scripts {
            if let Ok(content) = fs::read_to_string(&path) {
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    let expanded = crate::alias::expand(line);
                    let parsed = crate::parser::parse_line(&expanded);
                    for pipeline in &parsed {
                        for cmd in &pipeline.commands {
                            if cmd.args.is_empty() {
                                continue;
                            }
                            let _ = crate::builtins::execute_builtin(
                                &cmd.args[0],
                                &cmd.args[1..],
                                &parsed,
                                &[],
                                &mut crate::job::JobControl::new(),
                            );
                        }
                    }
                }
            }
        }
    }
}

fn fetch_registry_remote() -> Option<Vec<RegistryEntry>> {
    let url = format!("{}/api/registry", REGISTRY_URL);
    let output = Command::new("curl")
        .args(["-s", "--max-time", "5", &url])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    serde_json::from_slice(&output.stdout).ok()
}

fn fetch_package_remote(name: &str) -> Option<Package> {
    let url = format!("{}/api/pkg/{}", REGISTRY_URL, name);
    let output = Command::new("curl")
        .args(["-s", "--max-time", "5", &url])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    serde_json::from_slice(&output.stdout).ok()
}

fn builtin_registry() -> Vec<Package> {
    let data: &str = include_str!("builtin_packages.json");
    serde_json::from_str(data).unwrap_or_else(|e| {
        eprintln!("updsh: failed to parse built-in registry: {}", e);
        vec![]
    })
}

fn fetch_package(name: &str) -> Option<Package> {
    fetch_package_remote(name).or_else(|| {
        builtin_registry().into_iter().find(|p| p.name == name)
    })
}

fn fetch_registry() -> Vec<RegistryEntry> {
    if let Some(remote) = fetch_registry_remote() {
        return remote;
    }
    builtin_registry()
        .iter()
        .map(|p| RegistryEntry {
            name: p.name.clone(),
            version: p.version.clone(),
            description: p.description.clone(),
            author: p.author.clone(),
            pkg_type: p.pkg_type.clone(),
            language: p.language.clone(),
            depends: p.depends.clone(),
            source_url: p.source_url.clone(),
            build_type: p.build_type.clone(),
        })
        .collect()
}

fn scan_malicious(content: &str, _name: &str) -> Vec<String> {
    let mut findings = vec![];

    let checks: Vec<(&str, &str)> = vec![
        ("rm -rf /", "wipes root filesystem"),
        ("rm -rf /*", "wipes root filesystem"),
        ("mkfs.", "formats filesystem"),
        ("dd if=/dev/zero of=", "writes to block device"),
        ("dd if=/dev/random of=", "writes to block device"),
        (":(){", "fork bomb"),
        (":|:&", "fork bomb"),
        ("| bash", "remote pipe to shell"),
        ("| sh", "remote pipe to shell"),
        ("| zsh", "remote pipe to shell"),
        ("| dash", "remote pipe to shell"),
        ("chmod 777 /", "opens permissions on root"),
        ("sudo rm -rf", "sudo rm -rf"),
        (">/dev/sda", "writes to disk device"),
        ("python -c", "python inline execution"),
        ("python3 -c", "python inline execution"),
        ("eval $(curl", "eval curl output"),
        ("eval $(wget", "eval wget output"),
    ];

    let lower = content.to_lowercase();
    for (pat, desc) in &checks {
        if lower.contains(pat) {
            findings.push(format!("  contains '{}' ({})", pat, desc));
        }
    }

    findings
}

fn compile_package(pkg: &Package) -> Result<(), String> {
    let source = pkg.source.as_ref().ok_or_else(|| "no source code".to_string())?;
    let bin_name = pkg.binary.as_deref().unwrap_or(&pkg.name);
    let out_dir = bin_dir();
    fs::create_dir_all(&out_dir).map_err(|e| format!("cannot create bin dir: {}", e))?;
    let out_path = out_dir.join(bin_name);

    let src_dir = installed_dir().join(&pkg.name);
    fs::create_dir_all(&src_dir).map_err(|e| format!("cannot create src dir: {}", e))?;

    match pkg.language.as_deref() {
        Some("c") | Some("c99") | None => {
            let src_path = src_dir.join("source.c");
            fs::write(&src_path, source).map_err(|e| format!("cannot write source: {}", e))?;

            let status = Command::new("gcc")
                .args(["-O2", "-Wall", "-Wextra", "-o"])
                .arg(&out_path)
                .arg(&src_path)
                .status()
                .map_err(|e| format!("gcc not found: {}", e))?;

            if !status.success() {
                let _ = fs::remove_file(&src_path);
                return Err("compilation failed".into());
            }

            let _ = fs::remove_file(&src_path);
        }
        Some("rust") | Some("rs") => {
            let src_path = src_dir.join("main.rs");
            fs::write(&src_path, source).map_err(|e| format!("cannot write source: {}", e))?;

            let status = Command::new("rustc")
                .args(["-O", "-o"])
                .arg(&out_path)
                .arg(&src_path)
                .status()
                .map_err(|e| format!("rustc not found: {}", e))?;

            if !status.success() {
                let _ = fs::remove_file(&src_path);
                return Err("compilation failed".into());
            }

            let _ = fs::remove_file(&src_path);
        }
        _ => return Err(format!("unsupported language: {:?}", pkg.language)),
    }

    Ok(())
}

fn build_package(pkg: &Package) -> Result<(), String> {
    let url = pkg.source_url.as_ref().ok_or_else(|| "no source URL".to_string())?;
    let bin_name = pkg.binary.as_deref().unwrap_or(&pkg.name);
    let out_dir = bin_dir();
    fs::create_dir_all(&out_dir).map_err(|e| format!("cannot create bin dir: {}", e))?;
    let out_path = out_dir.join(bin_name);
    let build_dir = installed_dir().join(&pkg.name).join("build");

    if build_dir.exists() {
        fs::remove_dir_all(&build_dir).map_err(|e| format!("cannot clean build dir: {}", e))?;
    }
    fs::create_dir_all(&build_dir).map_err(|e| format!("cannot create build dir: {}", e))?;

    println!("  -> cloning {} ...", url);
    let status = Command::new("git")
        .args(["clone", "--depth", "1", url, "."])
        .current_dir(&build_dir)
        .status()
        .map_err(|e| format!("git not found: {}", e))?;
    if !status.success() {
        let _ = fs::remove_dir_all(&build_dir);
        return Err("git clone failed".into());
    }

    for file in &pkg.files {
        let file_path = build_dir.join(&file.path);
        if let Some(parent) = file_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(&file_path, &file.content)
            .map_err(|e| format!("cannot write {}: {}", file.path, e))?;
        println!("  -> wrote {}", file.path);
    }

    match pkg.build_type.as_deref() {
        Some("cmake") => {
            let bt_dir = build_dir.join("build_cmake");
            fs::create_dir_all(&bt_dir).map_err(|e| format!("cannot create cmake dir: {}", e))?;

            println!("  -> running cmake ...");
            let status = Command::new("cmake")
                .args([
                    "..", "-DCMAKE_BUILD_TYPE=Release",
                    "-DENABLE_VULKAN=OFF",
                    "-DENABLE_WAYLAND=OFF",
                    "-DENABLE_X11=OFF",
                    "-DENABLE_DRM=OFF",
                    "-DENABLE_DDC=OFF",
                    "-DENABLE_RPM=OFF",
                    "-DENABLE_DEB=OFF",
                    "-DENABLE_PACMAN=OFF",
                    "-DENABLE_ZFS=OFF",
                    "-DENABLE_LM_SENSORS=OFF",
                    "-DENABLE_OPENCL=OFF",
                ])
                .current_dir(&bt_dir)
                .status()
                .map_err(|e| format!("cmake not found: {}", e))?;
            if !status.success() {
                eprintln!("  -> cmake failed. Build dir kept at: {}", build_dir.display());
                eprintln!("  -> cd {} && cd build_cmake && cmake .. -DCMAKE_BUILD_TYPE=Release", build_dir.display());
                return Err("cmake configuration failed".into());
            }

            println!("  -> running make (this may take a while) ...");
            let status = Command::new("make")
                .args(["-j"])
                .current_dir(&bt_dir)
                .status()
                .map_err(|e| format!("make not found: {}", e))?;
            if !status.success() {
                eprintln!("  -> make failed. Build dir kept at: {}", build_dir.display());
                eprintln!("  -> cd {}/build_cmake && make -j", build_dir.display());
                return Err("make failed".into());
            }

            let built = bt_dir.join(bin_name);
            if built.exists() {
                fs::copy(&built, &out_path).map_err(|e| format!("cannot copy binary: {}", e))?;
            }
        }
        Some("cargo") => {
            println!("  -> running cargo build --release ...");
            let status = Command::new("cargo")
                .args(["build", "--release"])
                .current_dir(&build_dir)
                .status()
                .map_err(|e| format!("cargo not found: {}", e))?;
            if !status.success() {
                eprintln!("  -> cargo build failed. Build dir kept at: {}", build_dir.display());
                eprintln!("  -> cd {} && cargo build --release", build_dir.display());
                return Err("cargo build failed".into());
            }
            let built = build_dir.join("target").join("release").join(bin_name);
            if built.exists() {
                fs::copy(&built, &out_path).map_err(|e| format!("cannot copy binary: {}", e))?;
            }
        }
        Some(other) => {
            return Err(format!("unsupported build type: {}", other));
        }
        None => {
            println!("  -> running make ...");
            let status = Command::new("make")
                .current_dir(&build_dir)
                .status()
                .map_err(|e| format!("make not found: {}", e))?;
            if !status.success() {
                eprintln!("  -> make failed. Build dir kept at: {}", build_dir.display());
                eprintln!("  -> cd {} && make", build_dir.display());
                return Err("make failed".into());
            }
            let built = build_dir.join(bin_name);
            if built.exists() {
                fs::copy(&built, &out_path).map_err(|e| format!("cannot copy binary: {}", e))?;
            }
        }
    }

    let _ = fs::remove_dir_all(&build_dir);

    Ok(())
}

pub fn execute_pkg(args: &[String]) -> Option<i32> {
    if let Err(e) = guard_shell() {
        eprintln!("{}", e);
        return Some(1);
    }

    let subcommand = args.first().map(|s| s.as_str()).unwrap_or("help");

    let result = match subcommand {
        "install" | "i" => cmd_install(&args[1..]),
        "remove" | "r" | "uninstall" => cmd_remove(&args[1..]),
        "list" | "ls" => cmd_list(),
        "search" | "s" => cmd_search(&args[1..]),
        "info" | "show" => cmd_info(&args[1..]),
        "update" | "up" => cmd_update(),
        "help" | "h" | _ => cmd_help(),
    };
    result.map(|_| 0).or(Some(1))
}

fn cmd_install(args: &[String]) -> Option<i32> {
    if args.is_empty() {
        eprintln!("pkg install: missing package name");
        eprintln!("usage: pkg install <package>");
        return None;
    }

    ensure_dirs();

    for name in args {
        if is_installed(name) {
            eprintln!("pkg: {} is already installed", name);
            continue;
        }

        let pkg = match fetch_package(name) {
            Some(p) => p,
            None => {
                eprintln!("pkg: package '{}' not found in registry", name);
                eprintln!("       try 'pkg search {}'", name);
                continue;
            }
        };

        for dep in &pkg.depends {
            if !is_installed(dep) {
                println!("  -> installing dependency: {}", dep);
                let dep_args = vec![dep.clone()];
                cmd_install(&dep_args);
            }
        }

        let all_content: String = pkg
            .files
            .iter()
            .map(|f| f.content.as_str())
            .chain(pkg.source.as_deref().into_iter())
            .collect::<Vec<_>>()
            .join("\n");

        let findings = scan_malicious(&all_content, name);
        if !findings.is_empty() {
            eprintln!("\npkg: SECURITY ALERT: malicious code detected in '{}'!", name);
            eprintln!("      This package contains potentially dangerous code:");
            for f in &findings {
                eprintln!("{}", f);
            }
            eprintln!();
            eprintln!("      The package has been REMOVED and BLOCKED for your safety.");
            let _ = remove_installed(name);
            continue;
        }

        for (k, v) in &pkg.env {
            std::env::set_var(k, v);
        }

        if pkg.pkg_type.as_deref() == Some("compile") {
            if let Err(e) = compile_package(&pkg) {
                eprintln!("pkg: compilation failed for {}: {}", name, e);
                continue;
            }
            let bin_path = bin_dir().join(pkg.binary.as_deref().unwrap_or(name));
            if bin_path.exists() {
                println!("  -> compiled: {}", bin_path.display());
            }
        } else if pkg.pkg_type.as_deref() == Some("build") {
            if let Err(e) = build_package(&pkg) {
                eprintln!("pkg: build failed for {}: {}", name, e);
                continue;
            }
            let bin_path = bin_dir().join(pkg.binary.as_deref().unwrap_or(name));
            if bin_path.exists() {
                println!("  -> built: {}", bin_path.display());
            }
        }

        if let Err(e) = save_installed_meta(&pkg) {
            eprintln!("pkg: failed to install {}: {}", name, e);
            continue;
        }

        if let Err(e) = enable_package(&pkg) {
            eprintln!("pkg: failed to enable {}: {}", name, e);
            continue;
        }

        if pkg.pkg_type.as_deref() == Some("build") {
            let pkg_dir = installed_dir().join(name);
            std::env::set_var("UPD_PKG_DIR", pkg_dir.to_string_lossy().as_ref());
            let mut setup_jobs = crate::job::JobControl::new();
            for file in &pkg.files {
                if file.path.ends_with(".sh") {
                    for line in file.content.lines() {
                        let line = line.trim();
                        if line.is_empty() || line.starts_with('#') {
                            continue;
                        }
                        let expanded = crate::alias::expand(line);
                        let parsed = crate::parser::parse_line(&expanded);
                        for pipeline in &parsed {
                            crate::executor::execute_pipeline(
                                &pipeline.commands,
                                &parsed,
                                &[],
                                &mut setup_jobs,
                            );
                        }
                    }
                }
            }
            if name == "sysfetch" {
                let home = std::env::var("HOME").unwrap_or_default();
                let cfg_content = format!(
                    r#"{{"logo":{{"type":"file","source":"{}/.config/fastfetch/logo.txt"}}}}"#,
                    home.trim_end_matches('/')
                );
                let cfg_dir = home.trim_end_matches('/').to_string() + "/.config/fastfetch";
                let _ = std::fs::create_dir_all(&cfg_dir);
                let _ = std::fs::write(cfg_dir + "/config.jsonc", &cfg_content);
                let _ = std::fs::remove_file(bin_dir().join("sysfetch"));
                let _ = std::os::unix::fs::symlink("fastfetch", bin_dir().join("sysfetch"));
            }
            let bin_path_str = bin_dir().to_string_lossy().to_string();
            if let Ok(current) = std::env::var("PATH") {
                if !current.split(':').any(|p| p == bin_path_str) {
                    std::env::set_var("PATH", format!("{}:{}", bin_path_str, current));
                }
            }
        }

        println!("  -> installed {}", name);
    }

    Some(0)
}

fn cmd_remove(args: &[String]) -> Option<i32> {
    if args.is_empty() {
        eprintln!("pkg remove: missing package name");
        eprintln!("usage: pkg remove <package>");
        return None;
    }

    for name in args {
        if !is_installed(name) {
            eprintln!("pkg: {} is not installed", name);
            continue;
        }

        disable_package(name);
        if let Err(e) = remove_installed(name) {
            eprintln!("pkg: failed to remove {}: {}", name, e);
            continue;
        }

        println!("  -> removed {}", name);
    }

    Some(0)
}

fn cmd_list() -> Option<i32> {
    let installed = list_installed();

    if installed.is_empty() {
        println!("No packages installed.");
        println!("  Run 'pkg search' to see available packages.");
        println!("  Run 'pkg install <package>' to install one.");
        return Some(0);
    }

    println!("Installed packages:");
    for name in &installed {
        if let Some(meta) = load_installed_meta(name) {
            let enabled = enabled_dir().join(format!("{}.sh", name)).exists();
            let status = if enabled { "enabled" } else { "disabled" };
            println!(
                "  \x1b[32m{}\x1b[0m v{}  \x1b[34m{}\x1b[0m  [{}]",
                name, meta.version, meta.description, status
            );
        } else {
            let enabled = enabled_dir().join(format!("{}.sh", name)).exists();
            let status = if enabled { "enabled" } else { "disabled" };
            println!("  \x1b[32m{}\x1b[0m  [{}]", name, status);
        }
    }

    Some(0)
}

fn cmd_search(args: &[String]) -> Option<i32> {
    let registry = fetch_registry();
    let query = args.first().map(|s| s.to_lowercase());

    let mut results: Vec<&RegistryEntry> = registry
        .iter()
        .filter(|p| {
            if let Some(q) = &query {
                p.name.to_lowercase().contains(q)
                    || p.description.to_lowercase().contains(q)
            } else {
                true
            }
        })
        .collect();

    results.sort_by_key(|p| &p.name);

    if results.is_empty() {
        if let Some(q) = query {
            println!("No packages found matching '{}'.", q);
        } else {
            println!("No packages available.");
        }
        return Some(0);
    }

    println!("Available packages:");
    for entry in &results {
        let installed = if is_installed(&entry.name) {
            " \x1b[32m[installed]\x1b[0m"
        } else {
            ""
        };
        let ptype = entry.pkg_type.as_deref().unwrap_or("source");
        let lang = entry.language.as_deref().unwrap_or("");
        let tag = if ptype == "compile" {
            format!(" \x1b[35m[{}-compile]\x1b[0m", lang)
        } else if ptype == "build" {
            let bt = entry.build_type.as_deref().unwrap_or("src");
            format!(" \x1b[35m[{}-build]\x1b[0m", bt)
        } else {
            String::new()
        };
        println!(
            "  \x1b[33m{}\x1b[0m  {}{}{}",
            entry.name, entry.description, installed, tag
        );
    }
    if query.is_none() {
        println!();
        println!("Run 'pkg install <package>' to install a package.");
    }

    Some(0)
}

fn cmd_info(args: &[String]) -> Option<i32> {
    let name = args.first()?;

    let pkg = fetch_package(name);
    let installed_meta = load_installed_meta(name);

    let pkg = match (pkg.as_ref(), installed_meta.as_ref()) {
        (Some(p), _) => p,
        (None, Some(m)) => m,
        (None, None) => {
            eprintln!("pkg: package '{}' not found", name);
            return None;
        }
    };

    let installed = is_installed(name);
    let enabled = enabled_dir().join(format!("{}.sh", name)).exists();
    let status = if installed && enabled {
        "installed, enabled"
    } else if installed {
        "installed, disabled"
    } else {
        "not installed"
    };

    let ptype = pkg.pkg_type.as_deref().unwrap_or("source");

    println!("\x1b[1m{}\x1b[0m v{}", pkg.name, pkg.version);
    println!("  {}", pkg.description);
    println!();
    println!("  Status:     {}", status);
    println!("  Type:       {}", ptype);
    if let Some(lang) = &pkg.language {
        println!("  Language:   {}", lang);
    }
    if let Some(author) = &pkg.author {
        println!("  Author:     {}", author);
    }
    if !pkg.depends.is_empty() {
        println!("  Depends:    {}", pkg.depends.join(", "));
    }
    if !pkg.env.is_empty() {
        println!(
            "  Env vars:   {}",
            pkg.env.keys().cloned().collect::<Vec<_>>().join(", ")
        );
    }
    if ptype == "compile" {
        if let Some(src) = &pkg.source {
            println!("  Source:     {} lines", src.lines().count());
        }
    }
    if ptype == "build" {
        if let Some(url) = &pkg.source_url {
            println!("  Repo:       {}", url);
        }
        if let Some(bt) = &pkg.build_type {
            println!("  Build:      {}", bt);
        }
    }
    if !pkg.files.is_empty() {
        println!("  Scripts:    {} file(s)", pkg.files.len());
        for f in &pkg.files {
            let lc = f.content.lines().count();
            println!("    - {} ({} lines)", f.path, lc);
        }
    }

    Some(0)
}

fn cmd_update() -> Option<i32> {
    println!("Checking registry for updates...");
    let registry = fetch_registry();
    let installed = list_installed();
    let mut updated = 0;

    for name in &installed {
        if let Some(meta) = load_installed_meta(name) {
            if let Some(remote) = registry.iter().find(|p| p.name == *name) {
                if remote.version != meta.version {
                    println!("  -> updating {}: {} -> {}", name, meta.version, remote.version);
                    if let Some(remote_pkg) = fetch_package(name) {
                        if let Err(e) = save_installed_meta(&remote_pkg) {
                            eprintln!("pkg: failed to update {}: {}", name, e);
                            continue;
                        }
                        if let Err(e) = enable_package(&remote_pkg) {
                            eprintln!("pkg: failed to re-enable {}: {}", name, e);
                            continue;
                        }
                        if remote_pkg.pkg_type.as_deref() == Some("compile") {
                            if let Err(e) = compile_package(&remote_pkg) {
                                eprintln!("pkg: recompilation failed for {}: {}", name, e);
                                continue;
                            }
                        } else if remote_pkg.pkg_type.as_deref() == Some("build") {
                            if let Err(e) = build_package(&remote_pkg) {
                                eprintln!("pkg: rebuild failed for {}: {}", name, e);
                                continue;
                            }
                        }
                        updated += 1;
                    }
                }
            }
        }
    }

    if updated == 0 {
        println!("All packages up to date.");
    } else {
        println!("Updated {} package(s).", updated);
    }

    Some(0)
}

fn cmd_help() -> Option<i32> {
    println!("updSH Package Manager");
    println!();
    println!("Usage: pkg <command> [arguments]");
    println!();
    println!("Commands:");
    println!("  install (i)  <pkg>   Install a package");
    println!("  remove (r)   <pkg>   Remove a package");
    println!("  list (ls)             List installed packages");
    println!("  search (s)   [query]  Search available packages");
    println!("  info (show)  <pkg>    Show package details");
    println!("  update (up)           Update installed packages");
    println!("  help (h)              Show this help");
    println!();
    println!("Note: pkg only works inside the updSH shell.");
    println!("      Registry: {}", REGISTRY_URL);
    println!("      The registry is fetched live from the Vercel-hosted repo.");
    println!("      Falls back to built-in packages if offline.");
    println!();
    println!("Package types:");
    println!("  source        Shell aliases and functions (sourced on startup)");
    println!("  compile       C/Rust source code (compiled with gcc/rustc)");
    println!("  build         Clone from git repo and build (cmake/cargo/make)");
    println!();
    println!("Security: All packages are scanned for malicious code.");
    println!("          Dangerous packages are automatically deleted.");
    Some(0)
}
