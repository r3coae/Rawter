#![allow(dead_code)]
//! Settings model, defaults, load/save. File: $XDG_CONFIG_HOME/rawter/settings.json

use crate::router::{Classifier, Method, Optimize, RouteCfg, CATALOG};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const HISTORY_CAP: usize = 50;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DestSetting {
    pub id: String,
    pub enabled: bool,
    pub method: Method,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rule {
    pub id: u64,
    pub keywords: String,
    pub target: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HistoryItem {
    pub task: String,
    pub id: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LaunchKey {
    CtrlEnter,
    Enter,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub destinations: Vec<DestSetting>,
    pub rules: Vec<Rule>,
    pub fallback: String,
    pub confirm: bool,
    pub classifier: Classifier,
    pub optimize: Optimize,
    pub launch_key: LaunchKey,
    pub clear_after: bool,
    pub keep_history: bool,
    pub history: Vec<HistoryItem>,
}

fn seed_destinations() -> Vec<DestSetting> {
    CATALOG
        .iter()
        .map(|d| DestSetting { id: d.id.into(), enabled: true, method: d.method, value: d.url.into() })
        .collect()
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            destinations: seed_destinations(),
            rules: vec![],
            fallback: "claude".into(),
            confirm: false,
            classifier: Classifier::Auto,
            optimize: Optimize::Quality,
            launch_key: LaunchKey::CtrlEnter,
            clear_after: true,
            keep_history: true,
            history: vec![],
        }
    }
}

pub fn config_path() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .unwrap_or_else(|| PathBuf::from(".config"));
    base.join("rawter").join("settings.json")
}

impl Settings {
    /// Parse JSON; invalid input gives defaults. Always merges in missing catalog destinations.
    pub fn from_json(s: &str) -> Self {
        let mut st: Settings = serde_json::from_str(s).unwrap_or_default();
        st.merge_catalog();
        st
    }

    pub fn load() -> Self {
        std::fs::read_to_string(config_path()).map_or_else(|_| Self::default(), |s| Self::from_json(&s))
    }

    pub fn save(&self) -> std::io::Result<()> {
        let p = config_path();
        if let Some(dir) = p.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let json = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(p, json)
    }

    /// Drop unknown ids, add catalog destinations missing from the saved file.
    fn merge_catalog(&mut self) {
        self.destinations.retain(|d| CATALOG.iter().any(|c| c.id == d.id));
        for c in CATALOG {
            if !self.destinations.iter().any(|d| d.id == c.id) {
                self.destinations.push(DestSetting {
                    id: c.id.into(),
                    enabled: true,
                    method: c.method,
                    value: c.url.into(),
                });
            }
        }
    }

    pub fn dest(&self, id: &str) -> Option<&DestSetting> {
        self.destinations.iter().find(|d| d.id == id)
    }

    pub fn any_enabled(&self) -> bool {
        self.destinations.iter().any(|d| d.enabled)
    }

    pub fn next_rule_id(&self) -> u64 {
        self.rules.iter().map(|r| r.id).max().map_or(1, |m| m + 1)
    }

    pub fn add_rule(&mut self) {
        let id = self.next_rule_id();
        self.rules.push(Rule { id, keywords: String::new(), target: self.fallback.clone() });
    }

    pub fn push_history(&mut self, task: &str, id: &str) {
        if !self.keep_history {
            return;
        }
        self.history.push(HistoryItem { task: task.into(), id: id.into() });
        let extra = self.history.len().saturating_sub(HISTORY_CAP);
        self.history.drain(..extra);
    }

    pub fn route_cfg(&self) -> RouteCfg {
        RouteCfg {
            enabled: self.destinations.iter().filter(|d| d.enabled).map(|d| d.id.clone()).collect(),
            rules: self.rules.iter().map(|r| (r.keywords.clone(), r.target.clone())).collect(),
            fallback: self.fallback.clone(),
            classifier: self.classifier,
            optimize: self.optimize,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeds_from_catalog() {
        let s = Settings::default();
        assert_eq!(s.destinations.len(), CATALOG.len());
        assert_eq!(s.dest("claude-code").unwrap().method, Method::Cli);
        assert_eq!(s.fallback, "claude");
        assert!(s.clear_after && s.keep_history && !s.confirm);
    }

    #[test]
    fn round_trips() {
        let mut s = Settings::default();
        s.confirm = true;
        s.optimize = Optimize::Cost;
        s.launch_key = LaunchKey::Enter;
        s.add_rule();
        s.push_history("hello", "claude");
        let back = Settings::from_json(&serde_json::to_string(&s).unwrap());
        assert_eq!(s, back);
    }

    #[test]
    fn invalid_json_gives_defaults() {
        assert_eq!(Settings::from_json("not json"), Settings::default());
    }

    #[test]
    fn missing_destinations_are_added() {
        let s = Settings::from_json(r#"{"destinations":[{"id":"claude","enabled":false,"method":"url","value":"x"}],"confirm":true}"#);
        assert_eq!(s.destinations.len(), CATALOG.len());
        assert!(!s.dest("claude").unwrap().enabled);
        assert!(s.confirm);
    }

    #[test]
    fn history_is_capped() {
        let mut s = Settings::default();
        for i in 0..60 {
            s.push_history(&format!("t{i}"), "claude");
        }
        assert_eq!(s.history.len(), HISTORY_CAP);
        assert_eq!(s.history[0].task, "t10");
    }
}
