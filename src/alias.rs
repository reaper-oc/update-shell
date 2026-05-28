use std::collections::HashMap;
use std::sync::Mutex;

static ALIASES: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);

pub fn init() {
    let mut guard = ALIASES.lock().unwrap();
    if guard.is_none() {
        *guard = Some(HashMap::new());
    }
}

pub fn set(name: &str, value: &str) {
    if let Ok(mut guard) = ALIASES.lock() {
        if let Some(map) = guard.as_mut() {
            map.insert(name.to_string(), value.to_string());
        }
    }
}

pub fn get(name: &str) -> Option<String> {
    if let Ok(guard) = ALIASES.lock() {
        guard.as_ref().and_then(|map| map.get(name).cloned())
    } else {
        None
    }
}

pub fn remove(name: &str) -> bool {
    if let Ok(mut guard) = ALIASES.lock() {
        guard.as_mut().map_or(false, |map| map.remove(name).is_some())
    } else {
        false
    }
}

pub fn list() -> Vec<(String, String)> {
    if let Ok(guard) = ALIASES.lock() {
        guard
            .as_ref()
            .map(|map| {
                let mut pairs: Vec<_> = map.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                pairs.sort_by_key(|(k, _)| k.clone());
                pairs
            })
            .unwrap_or_default()
    } else {
        vec![]
    }
}

pub fn expand(line: &str) -> String {
    if let Some(first_word) = line.split_whitespace().next() {
        if let Some(replacement) = get(first_word) {
            let rest = &line[first_word.len()..];
            return format!("{} {}", replacement, rest.trim_start());
        }
    }
    line.to_string()
}
