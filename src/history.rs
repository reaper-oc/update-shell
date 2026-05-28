use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

pub struct History {
    entries: Vec<String>,
    file_path: PathBuf,
    max_entries: usize,
}

impl History {
    pub fn new() -> Self {
        let file_path = get_history_path();
        let entries = load_history(&file_path);
        History {
            entries,
            file_path,
            max_entries: 10000,
        }
    }

    pub fn add(&mut self, cmd: &str) {
        let trimmed = cmd.trim();
        if trimmed.is_empty() {
            return;
        }
        if self.entries.last().map_or(false, |last| last == trimmed) {
            return;
        }
        self.entries.push(trimmed.to_string());
        if self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }
        self.append_to_file(trimmed);
    }

    pub fn all(&self) -> &[String] {
        &self.entries
    }

    pub fn search(&self, term: &str) -> Vec<String> {
        self.entries
            .iter()
            .rev()
            .filter(|e| e.contains(term))
            .take(10)
            .cloned()
            .collect()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    fn append_to_file(&self, cmd: &str) {
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)
        {
            let _ = writeln!(file, "{}", cmd);
        }
    }
}

fn get_history_path() -> PathBuf {
    let base = if let Ok(dir) = std::env::var("XDG_DATA_HOME") {
        PathBuf::from(dir)
    } else if let Some(home) = dirs::home_dir() {
        home.join(".local").join("share")
    } else {
        PathBuf::from("/tmp")
    };
    let hist_dir = base.join("updsh");
    let _ = fs::create_dir_all(&hist_dir);
    hist_dir.join("history.txt")
}

fn load_history(path: &PathBuf) -> Vec<String> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return vec![],
    };
    BufReader::new(file)
        .lines()
        .filter_map(|l| l.ok())
        .filter(|l| !l.trim().is_empty())
        .collect()
}

#[allow(dead_code)]
pub struct HistoryHelper {
    pub entries: Vec<String>,
    pub idx: usize,
}

impl HistoryHelper {
    pub fn new(entries: &[String]) -> Self {
        HistoryHelper {
            entries: entries.to_vec(),
            idx: entries.len(),
        }
    }

    pub fn prev(&mut self) -> Option<&str> {
        if self.idx == 0 {
            return None;
        }
        self.idx -= 1;
        self.entries.get(self.idx).map(|s| s.as_str())
    }

    pub fn next(&mut self) -> Option<&str> {
        if self.idx >= self.entries.len() {
            return None;
        }
        self.idx += 1;
        self.entries.get(self.idx).map(|s| s.as_str())
    }
}
