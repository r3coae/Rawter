#![allow(dead_code)]
//! Port of reference/router.js. Catalog numbers and regexes are copied as is.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::LazyLock;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cat {
    Ui, App, Code, Agent, Research, Write, Reason, Docs, Data, Image, Video, Music, Voice, Chat,
}
use Cat::*;

pub const CATS: [Cat; 14] = [
    Ui, App, Code, Agent, Research, Write, Reason, Docs, Data, Image, Video, Music, Voice, Chat,
];
const MODAL: [Cat; 4] = [Image, Video, Music, Voice];
pub const MIN_CLARITY: f64 = 0.25;

impl Cat {
    pub fn label(self) -> &'static str {
        match self {
            Ui => "UI design",
            App => "app building",
            Code => "coding",
            Agent => "repo-level coding",
            Research => "web research",
            Write => "writing",
            Reason => "reasoning and math",
            Docs => "long documents",
            Data => "data analysis",
            Image => "image generation",
            Video => "video generation",
            Music => "music",
            Voice => "voice and speech",
            Chat => "general chat",
        }
    }
    fn i(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Method {
    Url,
    Cli,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Classifier {
    Auto,
    Rules,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Optimize {
    Quality,
    Speed,
    Cost,
}

pub struct Dest {
    pub id: &'static str,
    pub name: &'static str,
    pub method: Method,
    pub needs_project: bool,
    pub url: &'static str,
    pub web: bool,
    pub context: f64,
    pub speed: f64,
    pub cost: f64,
    pub skills: &'static [(Cat, f64)],
}

impl Dest {
    fn skill(&self, c: Cat) -> f64 {
        self.skills.iter().find(|(k, _)| *k == c).map_or(0.0, |(_, v)| *v)
    }
}

pub fn by_id(id: &str) -> Option<&'static Dest> {
    CATALOG.iter().find(|d| d.id == id)
}

pub fn destination_name(id: &str) -> String {
    by_id(id).map_or(id.to_string(), |d| d.name.to_string())
}

macro_rules! dest {
    ($id:expr, $name:expr, $m:expr, $np:expr, $url:expr, $web:expr, $ctx:expr, $sp:expr, $cost:expr, $sk:expr) => {
        Dest { id: $id, name: $name, method: $m, needs_project: $np, url: $url, web: $web,
               context: $ctx, speed: $sp, cost: $cost, skills: &$sk }
    };
}

pub static CATALOG: &[Dest] = &[
    dest!("claude", "Claude", Method::Url, false, "https://claude.ai/new?q={task}", true, 0.9, 0.6, 0.4,
        [(Chat, 0.9), (Write, 0.95), (Reason, 0.9), (Code, 0.88), (Docs, 0.92), (Data, 0.8), (Research, 0.7), (Ui, 0.72), (App, 0.6), (Agent, 0.55)]),
    dest!("claude-code", "Claude Code", Method::Cli, true, "claude \"{task}\"", true, 0.8, 0.5, 0.3,
        [(Agent, 0.98), (Code, 0.95), (App, 0.72), (Ui, 0.5), (Reason, 0.6), (Data, 0.45), (Chat, 0.3), (Write, 0.3)]),
    dest!("chatgpt", "ChatGPT", Method::Url, false, "https://chatgpt.com/?q={task}", true, 0.8, 0.7, 0.5,
        [(Chat, 0.95), (Reason, 0.9), (Code, 0.85), (Write, 0.86), (Data, 0.88), (Research, 0.82), (Docs, 0.82), (Image, 0.85), (Voice, 0.6), (Ui, 0.62), (App, 0.5), (Agent, 0.55)]),
    dest!("gemini", "Gemini", Method::Url, false, "https://gemini.google.com/app", true, 1.0, 0.75, 0.6,
        [(Docs, 0.95), (Research, 0.86), (Chat, 0.88), (Reason, 0.9), (Code, 0.84), (Image, 0.82), (Video, 0.72), (Data, 0.82), (Write, 0.8), (Voice, 0.5), (Ui, 0.65), (App, 0.5)]),
    dest!("grok", "Grok", Method::Url, false, "https://grok.com/?q={task}", true, 0.7, 0.7, 0.5,
        [(Research, 0.8), (Chat, 0.85), (Reason, 0.86), (Code, 0.78), (Write, 0.72), (Image, 0.7), (Docs, 0.6)]),
    dest!("deepseek", "DeepSeek", Method::Url, false, "https://chat.deepseek.com/", true, 0.6, 0.6, 1.0,
        [(Reason, 0.9), (Code, 0.86), (Chat, 0.8), (Write, 0.7), (Data, 0.7), (Docs, 0.6), (Research, 0.5)]),
    dest!("mistral", "Le Chat", Method::Url, false, "https://chat.mistral.ai/chat?q={task}", true, 0.6, 0.85, 0.8,
        [(Chat, 0.8), (Write, 0.8), (Code, 0.76), (Reason, 0.72), (Docs, 0.65), (Research, 0.6), (Image, 0.5)]),
    dest!("perplexity", "Perplexity", Method::Url, false, "https://www.perplexity.ai/search?q={task}", true, 0.5, 0.8, 0.7,
        [(Research, 0.98), (Chat, 0.72), (Docs, 0.5), (Reason, 0.5), (Data, 0.35), (Write, 0.3), (Code, 0.3)]),
    dest!("notebooklm", "NotebookLM", Method::Url, false, "https://notebooklm.google.com/", false, 1.0, 0.6, 0.8,
        [(Docs, 0.97), (Research, 0.6), (Write, 0.4)]),
    dest!("v0", "v0.dev", Method::Url, false, "https://v0.dev/chat?q={task}", false, 0.5, 0.8, 0.5,
        [(Ui, 0.98), (App, 0.7), (Code, 0.45), (Agent, 0.2)]),
    dest!("bolt", "Bolt", Method::Url, false, "https://bolt.new/?prompt={task}", false, 0.5, 0.7, 0.5,
        [(App, 0.93), (Ui, 0.85), (Code, 0.55)]),
    dest!("lovable", "Lovable", Method::Url, false, "https://lovable.dev/?autosubmit=true#prompt={task}", false, 0.5, 0.7, 0.5,
        [(App, 0.94), (Ui, 0.87), (Code, 0.5)]),
    dest!("replit", "Replit", Method::Url, false, "https://replit.com/?prompt={task}", false, 0.5, 0.65, 0.5,
        [(App, 0.9), (Code, 0.8), (Agent, 0.6), (Ui, 0.65)]),
    dest!("cursor", "Cursor", Method::Url, true, "https://cursor.com/link/prompt?text={task}", true, 0.8, 0.7, 0.4,
        [(Code, 0.95), (Agent, 0.9), (App, 0.78), (Ui, 0.72), (Reason, 0.5)]),
    dest!("copilot", "GitHub Copilot", Method::Url, false, "https://github.com/copilot?prompt={task}", true, 0.6, 0.75, 0.6,
        [(Code, 0.86), (Agent, 0.8), (Chat, 0.5), (App, 0.45)]),
    dest!("midjourney", "Midjourney", Method::Url, false, "https://www.midjourney.com/imagine", false, 0.1, 0.5, 0.3,
        [(Image, 0.98)]),
    dest!("runway", "Runway", Method::Url, false, "https://app.runwayml.com/", false, 0.1, 0.4, 0.2,
        [(Video, 0.95), (Image, 0.6)]),
    dest!("suno", "Suno", Method::Url, false, "https://suno.com/create", false, 0.1, 0.6, 0.5,
        [(Music, 0.97)]),
    dest!("elevenlabs", "ElevenLabs", Method::Url, false, "https://elevenlabs.io/app/speech-synthesis", false, 0.3, 0.8, 0.4,
        [(Voice, 0.97), (Music, 0.4)]),
];

/* ---------------------------- classifier ---------------------------- */

fn re(p: &str) -> Regex {
    Regex::new(&format!("(?i){p}")).unwrap()
}

fn sig(cat: Cat, pats: &[(&str, f64)]) -> (Cat, Vec<(Regex, f64)>) {
    (cat, pats.iter().map(|(p, w)| (re(p), *w)).collect())
}

static SIGNALS: LazyLock<Vec<(Cat, Vec<(Regex, f64)>)>> = LazyLock::new(|| {
    vec![
        sig(Ui, &[
            (r"\b(ui|ux|user interface|frontend|front-end)\b", 0.6),
            (r"\b(landing page|login (screen|page|form)|sign-?up (screen|page|form)|dashboard|navbar|sidebar|hero section|pricing (page|table)|modal|mockup|wireframe|prototype)\b", 0.7),
            (r"\b(component|tailwind|shadcn|css|react|jsx|tsx|svelte|vue)\b", 0.35),
            (r"\bfigma\b", 0.4),
        ]),
        sig(App, &[
            (r"\b(build|make|create|ship|launch)\b.{0,30}\b(app|saas|mvp|website|web app|site|startup|product|platform)\b", 0.6),
            (r"\b(full-?stack|database|auth(entication)?|deploy|hosting|supabase|stripe|backend and frontend)\b", 0.35),
            (r"\b(mvp|saas)\b", 0.5),
        ]),
        sig(Code, &[
            (r"\b(code|coding|programming|program|script|function|method|bug|debug|exception|stack ?trace|compile[rd]?|regex|sql|query|endpoint|algorithm|python|javascript|typescript|rust|golang|java|c\+\+|node(\.js)?|docker|git|npm|api)\b", 0.5),
            (r"\b(fix|implement|write)\b.{0,30}\b(bug|function|class|method|test|query|script|endpoint|hook)\b", 0.7),
            (r"\b(unit|integration) tests?\b", 0.6),
            (r"\b(react|jsx|tsx|vue|svelte|css|html|tailwind)\b", 0.4),
        ]),
        sig(Agent, &[
            (r"\b(repo(sitory)?|codebase|monorepo|pull request|PR|migrat(e|ion)|across (the )?(project|codebase|files|repo)|all (the )?files|failing tests?|lint errors?|upgrade (the )?(dependencies|deps))\b", 0.75),
            (r"\b(refactor|rewrite|port|convert)\b.{0,40}\b(project|codebase|module|service|repo|app)\b", 0.7),
            (r"(src/|\.tsx?\b|\.py\b|\.rs\b|package\.json|dockerfile)", 0.35),
        ]),
        sig(Research, &[
            (r"\b(research|sources?|citations?|cite|literature|papers|studies|survey|compare|comparison|vs\.?|versus|alternatives to|pros and cons|market (size|landscape)|competitors?)\b", 0.55),
            (r"\b(latest|today|currently|current|news|recent(ly)?|this (week|month|year)|2025|2026|price of|who (is|won)|what happened|trending|release date)\b", 0.65),
            (r"\b(find|look up|search for)\b", 0.4),
            (r"https?://", 0.25),
        ]),
        sig(Write, &[
            (r"\b(write|draft|rewrite|rephrase|proofread|edit)\b.{0,40}\b(email|essay|article|blog|post|letter|story|poem|speech|copy|cover letter|bio|newsletter|report|memo|caption|tweet|thread)\b", 0.8),
            (r"\b(tone|persuasive|polish|copywriting|ghostwrite|storytelling|screenplay|lyrics)\b", 0.5),
        ]),
        sig(Reason, &[
            (r"\b(prove|proof|theorem|derive|integral|equation|calculus|probability|combinatorics|puzzle|riddle|logic|olympiad|step by step|solve for|optimi[sz]e)\b", 0.65),
            (r"\b(analy[sz]e|trade-?offs?|strategy|decide|evaluate|think through|reason about|why does)\b", 0.35),
            (r"\d+\s*[+\-*/^]\s*\d+", 0.4),
        ]),
        sig(Docs, &[
            (r"\b(summari[sz]e|tl;?dr|key points|extract|pdf|transcript|meeting notes|contract|whitepaper|chapter|slides?)\b", 0.5),
            (r"\b(this|the attached|my|these) (file|files|doc|docs|document|pdf|pdfs|report|notes|transcript|paper|papers)\b", 0.6),
            (r"\b(study guide|flashcards?|quiz me|podcast from)\b", 0.7),
        ]),
        sig(Data, &[
            (r"\b(csv|excel|spreadsheet|xlsx|pivot|dataset|data ?frame|pandas|chart|plot|graph|regression|statistics|correlation|forecast|kpi)\b", 0.7),
            (r"\banaly[sz]e (the )?(data|numbers|sales|results)\b", 0.7),
        ]),
        sig(Image, &[
            (r"\b(image|picture|photo|illustration|logo|icon|poster|artwork|render|wallpaper|portrait|concept art|painting|sketch|thumbnail|avatar|sticker)\b", 0.55),
            (r"\b(generate|create|make|draw|design|paint)\b.{0,30}\b(image|picture|photo|logo|illustration|poster|art|portrait|cover)\b", 0.85),
            (r"\b(photorealistic|cinematic|4k|8k|studio lighting|oil painting|watercolor|anime style)\b|--ar\b", 0.6),
        ]),
        sig(Video, &[
            (r"\b(video|clip|animation|animate|footage|b-?roll|reel|short film|trailer|motion graphics|text-to-video)\b", 0.65),
            (r"\b(generate|create|make)\b.{0,30}\b(video|clip|animation|trailer)\b", 0.9),
        ]),
        sig(Music, &[
            (r"\b(song|music|beat|melody|soundtrack|lo-?fi|jingle|instrumental|compose|chorus|bpm)\b", 0.65),
            (r"\b(generate|create|make|write)\b.{0,30}\b(song|beat|track|melody|soundtrack|jingle)\b", 0.9),
        ]),
        sig(Voice, &[
            (r"\b(voice-?over|narrat(e|ion)|text-to-speech|tts|read (this )?aloud|dubbing|dub|clone (my )?voice|speech synthesis)\b", 0.85),
        ]),
        sig(Chat, &[
            (r"(?s)^(what|why|how|who|when|where|can|could|should|is|are|does|do)\b.{0,200}\?\s*$", 0.3),
        ]),
    ]
});

static RECENCY: LazyLock<Regex> = LazyLock::new(|| {
    re(r"\b(latest|today|currently|news|recent(ly)?|this (week|month|year)|2025|2026|price of|who won|what happened|trending|release date)\b")
});
static INDENT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\n\s{2,}\S").unwrap());
static BRACES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[{};]").unwrap());
static ERRWORD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(Error|Exception|Traceback)\b").unwrap());

pub struct Profile {
    pub raw: [f64; 14],
    pub weights: [f64; 14],
    pub active: Vec<Cat>,
    pub clarity: f64,
    pub recency: bool,
    pub modal: Vec<Cat>,
}

fn boost(raw: &mut [f64; 14], c: Cat, w: f64) {
    raw[c.i()] = 1.0 - (1.0 - raw[c.i()]) * (1.0 - w);
}

pub fn classify(task: &str) -> Profile {
    let text = task.trim();
    let mut raw = [0.0f64; 14];

    for (cat, pats) in SIGNALS.iter() {
        for (r, w) in pats {
            if r.is_match(text) {
                boost(&mut raw, *cat, *w);
            }
        }
    }

    if text.contains("```") || (INDENT.is_match(text) && BRACES.is_match(text)) {
        boost(&mut raw, Code, 0.6);
    }
    if ERRWORD.is_match(text) && text.contains('\n') {
        boost(&mut raw, Code, 0.7);
    }
    if text.chars().count() > 1500 && raw[Code.i()] < 0.5 {
        boost(&mut raw, Docs, 0.5);
    }

    let specific = CATS.iter().filter(|c| **c != Chat).map(|c| raw[c.i()]).fold(0.0, f64::max);
    if specific < MIN_CLARITY {
        raw[Chat.i()] = raw[Chat.i()].max(0.5);
    }
    let top = raw.iter().cloned().fold(0.0, f64::max);

    let mut weights = [0.0f64; 14];
    for c in CATS {
        weights[c.i()] = raw[c.i()] / top;
    }
    let mut active: Vec<Cat> = CATS.iter().copied().filter(|c| weights[c.i()] >= 0.35).collect();
    active.sort_by(|a, b| weights[b.i()].partial_cmp(&weights[a.i()]).unwrap());
    active.truncate(3);

    Profile {
        raw,
        weights,
        active,
        clarity: specific.min(1.0),
        recency: RECENCY.is_match(text),
        modal: MODAL.iter().copied().filter(|c| raw[c.i()] >= 0.5).collect(),
    }
}

/* ----------------------------- scoring ------------------------------ */

fn score_destination(d: &Dest, p: &Profile, task: &str, optimize: Optimize) -> f64 {
    let (mut num, mut den) = (0.0, 0.0);
    for c in &p.active {
        let w = p.weights[c.i()].powi(2);
        num += w * d.skill(*c);
        den += w;
    }
    let mut score = if den > 0.0 { num / den } else { 0.0 };

    for c in &p.modal {
        if d.skill(*c) < 0.3 {
            score *= 0.15;
        }
    }
    if d.needs_project && p.raw[Agent.i()] < 0.3 {
        score *= 0.88;
    }
    if p.recency {
        score += if d.web { 0.06 } else { -0.12 };
    }
    if task.chars().count() > 6000 {
        score *= 0.75 + 0.25 * d.context;
    }
    match optimize {
        Optimize::Speed => score = score * 0.8 + d.speed * 0.2,
        Optimize::Cost => score = score * 0.8 + d.cost * 0.2,
        Optimize::Quality => {}
    }
    score
}

pub fn rank(p: &Profile, task: &str, enabled: &HashSet<String>, optimize: Optimize) -> Vec<(&'static str, f64)> {
    let mut v: Vec<_> = CATALOG
        .iter()
        .filter(|d| enabled.contains(d.id))
        .map(|d| (d.id, score_destination(d, p, task, optimize)))
        .collect();
    v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    v
}

/* ----------------------------- decision ----------------------------- */

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Via {
    Rule,
    Auto,
    Fallback,
}

pub struct RouteCfg {
    pub enabled: HashSet<String>,
    /// (keywords, target), checked top to bottom
    pub rules: Vec<(String, String)>,
    pub fallback: String,
    pub classifier: Classifier,
    pub optimize: Optimize,
}

pub struct Route {
    pub id: Option<String>,
    pub ranked: Vec<String>,
    pub labels: Vec<&'static str>,
    pub confidence: f64,
    pub via: Via,
}

fn keyword_match(keywords: &str, task: &str) -> bool {
    keywords
        .split(',')
        .map(str::trim)
        .filter(|k| !k.is_empty())
        .any(|k| Regex::new(&format!(r"(?i)\b{}", regex::escape(k))).map_or(false, |r| r.is_match(task)))
}

pub fn route_task(task: &str, cfg: &RouteCfg) -> Route {
    let profile = classify(task);
    let scored = rank(&profile, task, &cfg.enabled, cfg.optimize);
    let ranked: Vec<String> = scored.iter().map(|s| s.0.to_string()).collect();
    let labels: Vec<&'static str> = profile.active.iter().map(|c| c.label()).collect();

    let first = |id: &str| -> Vec<String> {
        std::iter::once(id.to_string()).chain(ranked.iter().filter(|r| *r != id).cloned()).collect()
    };
    let fallback_id: Option<String> = if cfg.enabled.contains(&cfg.fallback) {
        Some(cfg.fallback.clone())
    } else {
        ranked.first().cloned()
    };
    let fallback_route = || Route {
        ranked: fallback_id.as_deref().map_or(ranked.clone(), |id| first(id)),
        id: fallback_id.clone(),
        labels: vec![],
        confidence: 0.3,
        via: Via::Fallback,
    };

    if let Some((_, target)) = cfg.rules.iter().find(|(k, t)| cfg.enabled.contains(t) && keyword_match(k, task)) {
        return Route { id: Some(target.clone()), ranked: first(target), labels: vec![], confidence: 1.0, via: Via::Rule };
    }

    if cfg.classifier == Classifier::Auto && !scored.is_empty() {
        if profile.clarity < MIN_CLARITY {
            return fallback_route();
        }
        let margin = if scored.len() > 1 { scored[0].1 - scored[1].1 } else { 0.1 };
        let confidence = profile.clarity * (0.55 + (margin * 4.0).min(0.45));
        return Route { id: Some(scored[0].0.to_string()), ranked, labels, confidence, via: Via::Auto };
    }
    fallback_route()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> RouteCfg {
        RouteCfg {
            enabled: CATALOG.iter().map(|d| d.id.to_string()).collect(),
            rules: vec![],
            fallback: "claude".into(),
            classifier: Classifier::Auto,
            optimize: Optimize::Quality,
        }
    }

    #[test]
    fn acceptance_table() {
        let rows: [(&str, &str, f64); 17] = [
            ("Build a React login screen with Tailwind", "v0", 0.69),
            ("Refactor the auth module across the repo and fix the failing tests", "claude-code", 0.74),
            ("What's the latest news on the EU AI Act?", "perplexity", 0.54),
            ("Draft a cover letter for a product manager role", "claude", 0.73),
            ("Generate a photorealistic poster of a lighthouse at dusk", "midjourney", 0.97),
            ("Summarize this PDF of meeting notes", "notebooklm", 0.50),
            ("Prove that the square root of 2 is irrational", "claude", 0.36),
            ("Make a lo-fi beat song about rainy mornings", "suno", 0.96),
            ("Build a SaaS app for invoicing with Stripe and auth", "lovable", 0.51),
            ("Write a Python function that parses ISO dates and add unit tests", "claude", 0.59),
            ("Compare Postgres vs MySQL for a small startup, with sources", "perplexity", 0.55),
            ("Make a 10 second video of waves crashing on a cliff", "runway", 0.96),
            ("Narrate this paragraph in a calm British voice", "elevenlabs", 0.85),
            ("Analyze this CSV of sales and plot revenue by month", "chatgpt", 0.52),
            ("hey", "claude", 0.30),
            ("Why is the sky blue?", "claude", 0.30),
            ("Build me a pricing page and a dashboard mockup", "v0", 0.69),
        ];
        let c = cfg();
        for (task, id, conf) in rows {
            let r = route_task(task, &c);
            assert_eq!(r.id.as_deref(), Some(id), "{task}");
            assert!((r.confidence - conf).abs() <= 0.02, "{task}: {} vs {conf}", r.confidence);
        }
    }

    #[test]
    fn fallback_via() {
        let c = cfg();
        assert_eq!(route_task("hey", &c).via, Via::Fallback);
        assert_eq!(route_task("Why is the sky blue?", &c).via, Via::Fallback);
    }

    #[test]
    fn v0_disabled_goes_to_lovable() {
        let mut c = cfg();
        c.enabled.remove("v0");
        assert_eq!(route_task("Build a React login screen with Tailwind", &c).id.as_deref(), Some("lovable"));
    }

    #[test]
    fn rule_overrides() {
        let mut c = cfg();
        c.rules.push(("login".into(), "cursor".into()));
        let r = route_task("Build a React login screen with Tailwind", &c);
        assert_eq!(r.id.as_deref(), Some("cursor"));
        assert_eq!(r.via, Via::Rule);
    }

    #[test]
    fn rules_mode_gives_fallback() {
        let mut c = cfg();
        c.classifier = Classifier::Rules;
        let r = route_task("Build a React login screen with Tailwind", &c);
        assert_eq!(r.id.as_deref(), Some("claude"));
    }

    #[test]
    fn nothing_enabled_is_none() {
        let mut c = cfg();
        c.enabled.clear();
        assert!(route_task("Build a React login screen with Tailwind", &c).id.is_none());
    }
}
