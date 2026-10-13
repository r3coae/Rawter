/**
 * router.js — picks the best AI tool for a task.
 *
 * Pipeline:
 *   1. classify(task)  -> how strongly the task belongs to each category (0..1)
 *   2. rank(profile)   -> score every enabled destination against that profile
 *   3. routeTask()     -> user rules override, low-clarity tasks go to the fallback
 *
 * The skill numbers in CATALOG are hand-set priors (0..1), not benchmark results.
 * Tune them to your own experience; the algorithm doesn't care where they come from.
 */

export const CATEGORY_LABELS = {
  ui: "UI design",
  app: "app building",
  code: "coding",
  agent: "repo-level coding",
  research: "web research",
  write: "writing",
  reason: "reasoning and math",
  docs: "long documents",
  data: "data analysis",
  image: "image generation",
  video: "video generation",
  music: "music",
  voice: "voice and speech",
  chat: "general chat",
};

const CATS = Object.keys(CATEGORY_LABELS);

/* ------------------------------------------------------------------ */
/* Catalog                                                             */
/* skills:  fit per category, 0..1                                     */
/* web:     can browse / has fresh information                         */
/* context: how well it handles very long input, 0..1                  */
/* speed, cost: higher = faster / cheaper                              */
/* url:     deep-link template, {task} is replaced with the task text  */
/* Deep-link formats change often. Treat them as best-effort.          */
/* ------------------------------------------------------------------ */

export const CATALOG = [
  { id: "claude", name: "Claude", method: "url", url: "https://claude.ai/new?q={task}", web: true, context: 0.9, speed: 0.6, cost: 0.4,
    skills: { chat: 0.9, write: 0.95, reason: 0.9, code: 0.88, docs: 0.92, data: 0.8, research: 0.7, ui: 0.72, app: 0.6, agent: 0.55 } },
  { id: "claude-code", name: "Claude Code", method: "cli", needsProject: true, url: 'claude "{task}"', web: true, context: 0.8, speed: 0.5, cost: 0.3,
    skills: { agent: 0.98, code: 0.95, app: 0.72, ui: 0.5, reason: 0.6, data: 0.45, chat: 0.3, write: 0.3 } },
  { id: "chatgpt", name: "ChatGPT", method: "url", url: "https://chatgpt.com/?q={task}", web: true, context: 0.8, speed: 0.7, cost: 0.5,
    skills: { chat: 0.95, reason: 0.9, code: 0.85, write: 0.86, data: 0.88, research: 0.82, docs: 0.82, image: 0.85, voice: 0.6, ui: 0.62, app: 0.5, agent: 0.55 } },
  { id: "gemini", name: "Gemini", method: "url", url: "https://gemini.google.com/app", web: true, context: 1, speed: 0.75, cost: 0.6,
    skills: { docs: 0.95, research: 0.86, chat: 0.88, reason: 0.9, code: 0.84, image: 0.82, video: 0.72, data: 0.82, write: 0.8, voice: 0.5, ui: 0.65, app: 0.5 } },
  { id: "grok", name: "Grok", method: "url", url: "https://grok.com/?q={task}", web: true, context: 0.7, speed: 0.7, cost: 0.5,
    skills: { research: 0.8, chat: 0.85, reason: 0.86, code: 0.78, write: 0.72, image: 0.7, docs: 0.6 } },
  { id: "deepseek", name: "DeepSeek", method: "url", url: "https://chat.deepseek.com/", web: true, context: 0.6, speed: 0.6, cost: 1,
    skills: { reason: 0.9, code: 0.86, chat: 0.8, write: 0.7, data: 0.7, docs: 0.6, research: 0.5 } },
  { id: "mistral", name: "Le Chat", method: "url", url: "https://chat.mistral.ai/chat?q={task}", web: true, context: 0.6, speed: 0.85, cost: 0.8,
    skills: { chat: 0.8, write: 0.8, code: 0.76, reason: 0.72, docs: 0.65, research: 0.6, image: 0.5 } },
  { id: "perplexity", name: "Perplexity", method: "url", url: "https://www.perplexity.ai/search?q={task}", web: true, context: 0.5, speed: 0.8, cost: 0.7,
    skills: { research: 0.98, chat: 0.72, docs: 0.5, reason: 0.5, data: 0.35, write: 0.3, code: 0.3 } },
  { id: "notebooklm", name: "NotebookLM", method: "url", url: "https://notebooklm.google.com/", web: false, context: 1, speed: 0.6, cost: 0.8,
    skills: { docs: 0.97, research: 0.6, write: 0.4 } },
  { id: "v0", name: "v0.dev", method: "url", url: "https://v0.dev/chat?q={task}", web: false, context: 0.5, speed: 0.8, cost: 0.5,
    skills: { ui: 0.98, app: 0.7, code: 0.45, agent: 0.2 } },
  { id: "bolt", name: "Bolt", method: "url", url: "https://bolt.new/?prompt={task}", web: false, context: 0.5, speed: 0.7, cost: 0.5,
    skills: { app: 0.93, ui: 0.85, code: 0.55 } },
  { id: "lovable", name: "Lovable", method: "url", url: "https://lovable.dev/?autosubmit=true#prompt={task}", web: false, context: 0.5, speed: 0.7, cost: 0.5,
    skills: { app: 0.94, ui: 0.87, code: 0.5 } },
  { id: "replit", name: "Replit", method: "url", url: "https://replit.com/?prompt={task}", web: false, context: 0.5, speed: 0.65, cost: 0.5,
    skills: { app: 0.9, code: 0.8, agent: 0.6, ui: 0.65 } },
  { id: "cursor", name: "Cursor", method: "url", needsProject: true, url: "https://cursor.com/link/prompt?text={task}", web: true, context: 0.8, speed: 0.7, cost: 0.4,
    skills: { code: 0.95, agent: 0.9, app: 0.78, ui: 0.72, reason: 0.5 } },
  { id: "copilot", name: "GitHub Copilot", method: "url", url: "https://github.com/copilot?prompt={task}", web: true, context: 0.6, speed: 0.75, cost: 0.6,
    skills: { code: 0.86, agent: 0.8, chat: 0.5, app: 0.45 } },
  { id: "midjourney", name: "Midjourney", method: "url", url: "https://www.midjourney.com/imagine", web: false, context: 0.1, speed: 0.5, cost: 0.3,
    skills: { image: 0.98 } },
  { id: "runway", name: "Runway", method: "url", url: "https://app.runwayml.com/", web: false, context: 0.1, speed: 0.4, cost: 0.2,
    skills: { video: 0.95, image: 0.6 } },
  { id: "suno", name: "Suno", method: "url", url: "https://suno.com/create", web: false, context: 0.1, speed: 0.6, cost: 0.5,
    skills: { music: 0.97 } },
  { id: "elevenlabs", name: "ElevenLabs", method: "url", url: "https://elevenlabs.io/app/speech-synthesis", web: false, context: 0.3, speed: 0.8, cost: 0.4,
    skills: { voice: 0.97, music: 0.4 } },
];

const byId = Object.fromEntries(CATALOG.map((d) => [d.id, d]));

/* ------------------------------------------------------------------ */
/* 1. Classifier                                                       */
/* Each pattern is [regex, weight]. Matches within a category combine  */
/* with noisy-OR: 1 - (1-w1)(1-w2)..., so more evidence raises the     */
/* score but it can never pass 1.                                      */
/* ------------------------------------------------------------------ */

const SIGNALS = {
  ui: [
    [/\b(ui|ux|user interface|frontend|front-end)\b/i, 0.6],
    [/\b(landing page|login (screen|page|form)|sign-?up (screen|page|form)|dashboard|navbar|sidebar|hero section|pricing (page|table)|modal|mockup|wireframe|prototype)\b/i, 0.7],
    [/\b(component|tailwind|shadcn|css|react|jsx|tsx|svelte|vue)\b/i, 0.35],
    [/\bfigma\b/i, 0.4],
  ],
  app: [
    [/\b(build|make|create|ship|launch)\b.{0,30}\b(app|saas|mvp|website|web app|site|startup|product|platform)\b/i, 0.6],
    [/\b(full-?stack|database|auth(entication)?|deploy|hosting|supabase|stripe|backend and frontend)\b/i, 0.35],
    [/\b(mvp|saas)\b/i, 0.5],
  ],
  code: [
    [/\b(code|coding|programming|program|script|function|method|bug|debug|exception|stack ?trace|compile[rd]?|regex|sql|query|endpoint|algorithm|python|javascript|typescript|rust|golang|java|c\+\+|node(\.js)?|docker|git|npm|api)\b/i, 0.5],
    [/\b(fix|implement|write)\b.{0,30}\b(bug|function|class|method|test|query|script|endpoint|hook)\b/i, 0.7],
    [/\b(unit|integration) tests?\b/i, 0.6],
    [/\b(react|jsx|tsx|vue|svelte|css|html|tailwind)\b/i, 0.4],
  ],
  agent: [
    [/\b(repo(sitory)?|codebase|monorepo|pull request|PR|migrat(e|ion)|across (the )?(project|codebase|files|repo)|all (the )?files|failing tests?|lint errors?|upgrade (the )?(dependencies|deps))\b/i, 0.75],
    [/\b(refactor|rewrite|port|convert)\b.{0,40}\b(project|codebase|module|service|repo|app)\b/i, 0.7],
    [/(src\/|\.tsx?\b|\.py\b|\.rs\b|package\.json|dockerfile)/i, 0.35],
  ],
  research: [
    [/\b(research|sources?|citations?|cite|literature|papers|studies|survey|compare|comparison|vs\.?|versus|alternatives to|pros and cons|market (size|landscape)|competitors?)\b/i, 0.55],
    [/\b(latest|today|currently|current|news|recent(ly)?|this (week|month|year)|2025|2026|price of|who (is|won)|what happened|trending|release date)\b/i, 0.65],
    [/\b(find|look up|search for)\b/i, 0.4],
    [/https?:\/\//i, 0.25],
  ],
  write: [
    [/\b(write|draft|rewrite|rephrase|proofread|edit)\b.{0,40}\b(email|essay|article|blog|post|letter|story|poem|speech|copy|cover letter|bio|newsletter|report|memo|caption|tweet|thread)\b/i, 0.8],
    [/\b(tone|persuasive|polish|copywriting|ghostwrite|storytelling|screenplay|lyrics)\b/i, 0.5],
  ],
  reason: [
    [/\b(prove|proof|theorem|derive|integral|equation|calculus|probability|combinatorics|puzzle|riddle|logic|olympiad|step by step|solve for|optimi[sz]e)\b/i, 0.65],
    [/\b(analy[sz]e|trade-?offs?|strategy|decide|evaluate|think through|reason about|why does)\b/i, 0.35],
    [/\d+\s*[+\-*/^]\s*\d+/, 0.4],
  ],
  docs: [
    [/\b(summari[sz]e|tl;?dr|key points|extract|pdf|transcript|meeting notes|contract|whitepaper|chapter|slides?)\b/i, 0.5],
    [/\b(this|the attached|my|these) (file|files|doc|docs|document|pdf|pdfs|report|notes|transcript|paper|papers)\b/i, 0.6],
    [/\b(study guide|flashcards?|quiz me|podcast from)\b/i, 0.7],
  ],
  data: [
    [/\b(csv|excel|spreadsheet|xlsx|pivot|dataset|data ?frame|pandas|chart|plot|graph|regression|statistics|correlation|forecast|kpi)\b/i, 0.7],
    [/\banaly[sz]e (the )?(data|numbers|sales|results)\b/i, 0.7],
  ],
  image: [
    [/\b(image|picture|photo|illustration|logo|icon|poster|artwork|render|wallpaper|portrait|concept art|painting|sketch|thumbnail|avatar|sticker)\b/i, 0.55],
    [/\b(generate|create|make|draw|design|paint)\b.{0,30}\b(image|picture|photo|logo|illustration|poster|art|portrait|cover)\b/i, 0.85],
    [/\b(photorealistic|cinematic|4k|8k|studio lighting|oil painting|watercolor|anime style)\b|--ar\b/i, 0.6],
  ],
  video: [
    [/\b(video|clip|animation|animate|footage|b-?roll|reel|short film|trailer|motion graphics|text-to-video)\b/i, 0.65],
    [/\b(generate|create|make)\b.{0,30}\b(video|clip|animation|trailer)\b/i, 0.9],
  ],
  music: [
    [/\b(song|music|beat|melody|soundtrack|lo-?fi|jingle|instrumental|compose|chorus|bpm)\b/i, 0.65],
    [/\b(generate|create|make|write)\b.{0,30}\b(song|beat|track|melody|soundtrack|jingle)\b/i, 0.9],
  ],
  voice: [
    [/\b(voice-?over|narrat(e|ion)|text-to-speech|tts|read (this )?aloud|dubbing|dub|clone (my )?voice|speech synthesis)\b/i, 0.85],
  ],
  chat: [
    [/^(what|why|how|who|when|where|can|could|should|is|are|does|do)\b[^]{0,200}\?\s*$/i, 0.3],
  ],
};

export const MIN_CLARITY = 0.25;

const MODAL = ["image", "video", "music", "voice"]; // output types most tools simply can't produce
const RECENCY = /\b(latest|today|currently|news|recent(ly)?|this (week|month|year)|2025|2026|price of|who won|what happened|trending|release date)\b/i;

/**
 * @returns {{
 *   raw: Record<string, number>,      // evidence per category, 0..1
 *   weights: Record<string, number>,  // raw divided by the strongest category
 *   active: string[],                 // up to 3 categories that matter, strongest first
 *   clarity: number,                  // strength of the strongest signal, 0..1
 *   recency: boolean,                 // needs fresh information
 *   modal: string[],                  // hard output-type requirements
 * }}
 */
export function classify(task) {
  const text = task.trim();
  const raw = Object.fromEntries(CATS.map((c) => [c, 0]));
  const boost = (cat, w) => (raw[cat] = 1 - (1 - raw[cat]) * (1 - w));

  for (const [cat, patterns] of Object.entries(SIGNALS)) {
    for (const [re, w] of patterns) if (re.test(text)) boost(cat, w);
  }

  // Structural signals that don't depend on specific words.
  if (/```/.test(text) || (/\n\s{2,}\S/.test(text) && /[{};]/.test(text))) boost("code", 0.6);
  if (/\b(Error|Exception|Traceback)\b/.test(text) && text.includes("\n")) boost("code", 0.7);
  if (text.length > 1500 && raw.code < 0.5) boost("docs", 0.5); // long pasted material

  // Clarity is the strongest *specific* signal; plain chat doesn't count.
  const specific = Math.max(...CATS.filter((c) => c !== "chat").map((c) => raw[c]));
  if (specific < MIN_CLARITY) raw.chat = Math.max(raw.chat, 0.5); // nothing specific: general chat
  const top = Math.max(...CATS.map((c) => raw[c]));

  const weights = Object.fromEntries(CATS.map((c) => [c, raw[c] / top]));
  const active = CATS.filter((c) => weights[c] >= 0.35)
    .sort((a, b) => weights[b] - weights[a])
    .slice(0, 3);

  return {
    raw,
    weights,
    active,
    clarity: Math.min(1, specific),
    recency: RECENCY.test(text),
    modal: MODAL.filter((c) => raw[c] >= 0.5),
  };
}

/* ------------------------------------------------------------------ */
/* 2. Scoring                                                          */
/* ------------------------------------------------------------------ */

function scoreDestination(dest, profile, task, optimize) {
  // Weighted average of skill over the active categories. Weights are
  // squared so the dominant category counts for much more than a side
  // category, which is what lets specialists beat generalists.
  let num = 0;
  let den = 0;
  for (const c of profile.active) {
    const w = profile.weights[c] ** 2;
    num += w * (dest.skills[c] ?? 0);
    den += w;
  }
  let score = den ? num / den : 0;

  // Hard requirement: if the task needs an image/video/music/voice output,
  // a tool that can't make it is almost never the right answer.
  for (const c of profile.modal) if ((dest.skills[c] ?? 0) < 0.3) score *= 0.15;

  // IDE and CLI agents need a local project; unlikely to be the right pick
  // for a self-contained question that doesn't mention a repo or codebase.
  if (dest.needsProject && profile.raw.agent < 0.3) score *= 0.88;

  // Freshness: reward tools that can browse, penalise those that can't.
  if (profile.recency) score += dest.web ? 0.06 : -0.12;

  // Very long input: scale down tools with small context.
  if (task.length > 6000) score *= 0.75 + 0.25 * dest.context;

  // Optional preference for speed or low cost (20% blend).
  if (optimize === "speed") score = score * 0.8 + dest.speed * 0.2;
  if (optimize === "cost") score = score * 0.8 + dest.cost * 0.2;

  return score;
}

/** Rank enabled destinations for a profile, best first. */
export function rank(profile, task, enabledIds, optimize = "quality") {
  return CATALOG.filter((d) => enabledIds.has(d.id))
    .map((d) => ({ id: d.id, score: scoreDestination(d, profile, task, optimize) }))
    .sort((a, b) => b.score - a.score);
}

/* ------------------------------------------------------------------ */
/* 3. Decision                                                         */
/* ------------------------------------------------------------------ */

const keywordMatch = (keywords, task) =>
  keywords
    .split(",")
    .map((k) => k.trim())
    .filter(Boolean)
    .some((k) => new RegExp("\\b" + k.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"), "i").test(task));

/**
 * @param {string} task
 * @param {{
 *   destinations: {id: string, enabled: boolean}[],
 *   rules: {keywords: string, target: string}[],
 *   fallback: string,
 *   classifier: "auto" | "rules",
 *   optimize: "quality" | "speed" | "cost",
 * }} settings
 * @returns {{
 *   id: string | null,
 *   ranked: string[],            // enabled destinations, best first (use for "next best")
 *   labels: string[],            // human-readable categories behind the decision
 *   confidence: number,          // 0..1
 *   via: "rule" | "auto" | "fallback",
 * }}
 */
export function routeTask(task, settings) {
  const enabled = new Set(settings.destinations.filter((d) => d.enabled).map((d) => d.id));
  const profile = classify(task);
  const scored = rank(profile, task, enabled, settings.optimize);
  const ranked = scored.map((s) => s.id);
  const labels = profile.active.map((c) => CATEGORY_LABELS[c]);

  const first = (id) => [id, ...ranked.filter((r) => r !== id)];
  const fallbackId = enabled.has(settings.fallback) ? settings.fallback : ranked[0] ?? null;

  // 1. Explicit user rules always win.
  const rule = settings.rules.find((r) => enabled.has(r.target) && keywordMatch(r.keywords, task));
  if (rule) return { id: rule.target, ranked: first(rule.target), labels: [], confidence: 1, via: "rule" };

  if (settings.classifier === "auto" && scored.length) {
    // 2. Not enough signal to beat a general-purpose default.
    if (profile.clarity < MIN_CLARITY) {
      return { id: fallbackId, ranked: first(fallbackId), labels: [], confidence: 0.3, via: "fallback" };
    }
    // 3. Best score. Confidence grows with signal clarity and the lead over #2.
    const margin = scored.length > 1 ? scored[0].score - scored[1].score : 0.1;
    const confidence = profile.clarity * (0.55 + Math.min(0.45, margin * 4));
    return { id: scored[0].id, ranked, labels, confidence, via: "auto" };
  }

  return { id: fallbackId, ranked: first(fallbackId), labels: [], confidence: 0.3, via: "fallback" };
}

export const destinationName = (id) => byId[id]?.name ?? id;
