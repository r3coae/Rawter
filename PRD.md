# rawter — PRD

A tiny native Linux app: type a task, press Ctrl+Enter, and rawter sends it to the best AI tool for the job.
Written for a builder model working in claude.ai chat. Read it fully once, then work milestone by milestone.

---

## 0. Rules for the builder (read first, they save tokens)

1. **One milestone per message** (section 11). Finish it, then stop and wait for the user.
2. **Output code, not commentary.** At most 3 lines of prose per reply. No recaps, no restating this PRD.
3. **Never re-print unchanged files.** For fixes, send only the changed function or block, with the file name.
4. **Do not try to run cargo in the chat.** There is no Rust toolchain here. The user compiles locally or in CI and pastes back **only the first compile error**.
5. **Do not plan out loud.** This PRD is the plan. Keep files under ~250 lines each.
6. **Copy tables mechanically.** The routing catalog and regex patterns live in the attached `reference/router.js`. Read it once during M1 and port it as is. Do not redesign it.
7. If an API has been renamed in the current egui/eframe release (they change often, for example `rounding` vs `corner_radius`, `id_source` vs `id_salt`), fix only that call.
8. If something here is ambiguous, pick the simplest option and mention it in one line. Don't ask a question unless you are blocked.

---

## 1. Product

- **Name:** rawter (lowercase everywhere: window title, binary, config dir, AppImage, desktop entry).
- **Platform:** Linux x86_64, shipped as `rawter-x86_64.AppImage`.
- **Language:** pure Rust. No webview, no Electron, no Tauri, no JS runtime.
- **Feel:** compact, dark, keyboard-first. Should look at home next to Zed, Obsidian and Neovim.

---

## 2. Tech stack

```toml
[package]
name = "rawter"
version = "0.1.0"
edition = "2021"

[dependencies]
eframe = "*"                 # use the latest 0.x, glow backend (default features)
egui_extras = { version = "*", features = ["svg"] }   # same version as eframe
regex = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
panic = "abort"
```

Pin `eframe` and `egui_extras` to the same concrete version once chosen.
Window icon: `eframe::icon_data::from_png_bytes(include_bytes!("../assets/rawter-256.png"))`. If it needs the `image` crate with the `png` feature, add it.

---

## 3. File layout

```
rawter/
  Cargo.toml
  build-appimage.sh            (provided, do not rewrite)
  packaging/rawter.desktop     (provided)
  .github/workflows/appimage.yml (provided)
  assets/rawter.png (512), rawter-{256,128,64,32}.png   (provided)
  assets/icons/{settings,close,plus,trash,check,route,loader}.svg  (provided)
  reference/router.js          (source of truth for routing, port from this)
  src/
    main.rs        window setup, theme, App struct, update loop      (~150 lines)
    router.rs      catalog + classify + rank + route_task + tests    (~350 lines)
    settings.rs    Settings struct, defaults, load/save              (~100 lines)
    ui_main.rs     main screen                                       (~150 lines)
    ui_settings.rs settings screen                                   (~200 lines)
    launch.rs      percent-encode, open URL, clipboard, shell-escape (~60 lines)
    icons.rs       load SVGs, tint helper                            (~40 lines)
```

---

## 4. Provided assets (already made, do not generate or edit)

- **Logo:** `assets/rawter.png` and sized PNGs. Dark rounded square with a node that forks into three routes, one highlighted blue.
- **UI icons** (24x24, stroke-only, `stroke="currentColor"`, stroke-width 1.75, Lucide style): `settings`, `close`, `plus`, `trash`, `check`, `route`, `loader`.
- **Using the SVGs:** embed with `include_str!`. `currentColor` renders black in resvg, so replace the string `currentColor` with `#ffffff` before loading, then tint at draw time (`egui::Image::tint(color)`). Load with `egui_extras::install_image_loaders(ctx)` and `Image::from_bytes("bytes://settings.svg", bytes)`.
- **Spinner:** use the `loader` icon rotated over time (request repaint while routing) or egui's `Spinner`.

---

## 5. Window and theme

- Inner size **700 x 450**, non-resizable (set min = max = 700x450), native window decorations, title `rawter`, app id `rawter`.
- Colors:

| token | hex | use |
|---|---|---|
| window | `#141416` | panel background |
| well | `#19191c` | text area and input background |
| raised | `#202024` | hover states |
| line | `#2a2a2f` | borders |
| text | `#d8d6d0` | primary text |
| dim | `#6f6e72` | hints, status, placeholder |
| accent | `#9fb4d6` | primary button, switches, selection |
| accentInk | `#101216` | text on accent |
| focus | `#3d4658` | focused input border |
| switch-off | `#34343a` | toggle track off |

- Set via `ctx.set_visuals(...)` once. Fonts: egui defaults. Text area uses the monospace font at 15px, everything else proportional 13px.
- Corner radius: button 8, inputs 6, kbd chips 3. No shadows.

---

## 6. Main screen

Top to bottom, 24px side padding:

1. **Gear icon** (`settings.svg`, 16px) floating top-right, 12px from the edges, dim color, text color on hover, tooltip `Settings (Ctrl + ,)`. No header bar.
2. **Text area:** multiline, fills the space above the footer (top padding 48px, 16px gap below). Placeholder exactly: `What do you need done? (e.g., Build a React login screen...)`. Autofocus on start and after closing settings. Border `line`, `focus` when focused, caret color `accent`.
3. **Footer** (about 90px):
   - Row: **primary button** `Route & Launch` (150x40, accent fill, accentInk bold text, `route.svg` icon on the left) and, 16px to its right, the **status area**.
   - Below the button, 10px gap: hint `Press [Ctrl + Enter] to launch` where `[...]` is a small kbd chip (well background, line border, mono 11px). If launch key is Enter: `Press [Enter] to launch (Shift + Enter for a new line)`.

### Status area (mono 13px, dim by default, always reserves one line of height)

| state | content |
|---|---|
| idle | empty |
| empty task | `Describe a task first.` |
| no destination enabled | `No destination enabled. Check Settings.` |
| routing | spinner + `Routing to {name}...` in text color, then the top category labels joined by ` + ` in dim |
| confirm | `Route to {name}?` in text color, then dim `Enter to confirm, Tab for next best` |
| done (URL) | check icon + `Launched in {name}` |
| done (URL without `{task}`) | check icon + `Opened {name}, task copied` |
| done (CLI) | check icon + `Command copied for {name}` |
| error | `Couldn't open the browser.` |

Typing in the text area while in `confirm` resets the status to idle.

---

## 7. Keyboard (the app is keyboard-first)

| key | action |
|---|---|
| Ctrl+Enter (or Cmd+Enter) | launch (default) |
| Enter | launch, only if the launch-key setting is "Enter"; Shift+Enter then inserts a newline |
| Ctrl+, | toggle settings |
| Esc | close settings; or cancel confirm |
| Enter / Tab (in confirm) | confirm / cycle to next best destination |

**egui gotcha:** `TextEdit` consumes Enter and Tab. Handle keys *before* drawing the text area by removing the matching key events:

```rust
fn take_key(ctx: &egui::Context, key: egui::Key, ok: impl Fn(&egui::Modifiers) -> bool) -> bool {
    ctx.input_mut(|i| {
        let mut hit = false;
        i.events.retain(|e| match e {
            egui::Event::Key { key: k, pressed: true, modifiers, .. } if *k == key && ok(modifiers) => {
                hit = true;
                false
            }
            _ => true,
        });
        hit
    })
}
```

For launch: `take_key(ctx, Key::Enter, |m| m.command)`; for plain Enter mode: `|m| !m.command && !m.shift && !m.alt`.

---

## 8. Settings screen

Opens in place of the main screen (same window, no popup). Header 48px: title `Settings` (left), a `Esc` kbd chip and the close icon (right). Below it a vertical ScrollArea with these sections, each separated by a 1px `line` rule, title 13px bold, optional dim 12px hint:

1. **Destinations**. Hint: `Tools tasks can be sent to. Use {task} where the task text goes.` One row per destination: toggle switch (30x18, accent when on), name (fixed 112px, dim when disabled), method combo (`URL` / `CLI`, 68px), mono text field for the target (fills the rest).
2. **Routing**
   - Routing mode: `Auto (scored)` / `Keyword rules only`.
   - Optimize for: `Quality` / `Speed` / `Cost`.
   - Confirm before launch (toggle). Hint: `Shows the chosen destination first. Enter confirms, Tab shows the next best.`
   - Fallback destination (combo of all destinations).
   - Override rules: hint `Override rules run before the scorer, top to bottom. Separate keywords with commas.` Each rule row: keywords field (placeholder `keywords, like, these`), destination combo, trash icon button. Below: `+ Add rule` text button in accent color.
3. **Launch**: Launch key (`Ctrl + Enter` / `Enter`), Clear input after launch (toggle).
4. **History**: Keep task history (toggle); `{n} saved task(s)` with a `Clear history` button (disabled when 0).

Save settings to disk when this screen closes and after each launch.

### Settings model (serde)

```
destinations: Vec<{ id, enabled, method: url|cli, value }>   // seeded from the catalog
rules: Vec<{ id: u64, keywords: String, target: String }>    // default empty
fallback: String            // default "claude"
confirm: bool               // default false
classifier: auto|rules      // default auto
optimize: quality|speed|cost // default quality
launch_key: ctrl_enter|enter // default ctrl_enter
clear_after: bool           // default true
keep_history: bool          // default true
history: Vec<{ task, id }>  // capped at 50
```

File: `$XDG_CONFIG_HOME/rawter/settings.json`, else `~/.config/rawter/settings.json`. If missing or invalid, use defaults. When loading, add any catalog destinations missing from the saved file.

---

## 9. Routing

Port `reference/router.js` to `src/router.rs` faithfully. Use `f64`. Compile regexes once (`std::sync::LazyLock`).

### Pipeline

1. **classify(task)**: 14 categories (`ui, app, code, agent, research, write, reason, docs, data, image, video, music, voice, chat`). Each has `(regex, weight)` pairs. Evidence per category is a noisy-OR: `1 - Π(1 - w)`. Then structural boosts (code fence, stack trace, text longer than 1500 chars adds `docs`). `clarity` is the strongest category excluding `chat`. If `clarity < 0.25`, `chat` gets 0.5. `weights = raw / top`. `active` is up to 3 categories with weight >= 0.35. `recency` and `modal` flags as in the JS.
2. **rank**: for each enabled destination, take the weighted average of its skill over the active categories, using weights **squared**. Then:
   - for each `modal` category (image, video, music, voice) where the destination's skill < 0.3: score x 0.15
   - `needs_project` destination and `raw.agent < 0.3`: x 0.88
   - recency: +0.06 if it can browse, else -0.12
   - task longer than 6000 chars: x (0.75 + 0.25 x context)
   - optimize speed or cost: `score x 0.8 + (speed or cost) x 0.2`
3. **route_task**: user rules win first (keyword match, word-boundary, case-insensitive, target must be enabled), then if mode is auto and `clarity >= 0.25` the top score wins, otherwise the fallback. `confidence = clarity x (0.55 + min(0.45, margin x 4))` where margin is score #1 minus #2 (0.1 if only one). Returns `{ id: Option, ranked: Vec<id>, labels: Vec<&str>, confidence, via }`.
4. If the fallback is disabled, use the top-ranked enabled destination. If none are enabled, `id = None`.

### Rust regex notes

No lookaround. Use `(?i)` for case-insensitive. The JS `[^]{0,200}` becomes `(?s:.){0,200}`. Escape user keywords with `regex::escape`.

### Catalog

20 destinations, ids: `claude, claude-code, chatgpt, gemini, grok, deepseek, mistral, perplexity, notebooklm, v0, bolt, lovable, replit, cursor, copilot, midjourney, runway, suno, elevenlabs`.
Copy names, skill numbers, `web`, `context`, `speed`, `cost`, `needsProject` and URL templates exactly from `reference/router.js`. `claude-code` is the only `cli` method.

### Acceptance tests (put in `#[cfg(test)]` in router.rs; all destinations enabled, fallback `claude`, mode auto, optimize quality)

Expected top pick, with confidence within 0.02 of the value shown:

| task | expected id | confidence |
|---|---|---|
| Build a React login screen with Tailwind | v0 | 0.69 |
| Refactor the auth module across the repo and fix the failing tests | claude-code | 0.74 |
| What's the latest news on the EU AI Act? | perplexity | 0.54 |
| Draft a cover letter for a product manager role | claude | 0.73 |
| Generate a photorealistic poster of a lighthouse at dusk | midjourney | 0.97 |
| Summarize this PDF of meeting notes | notebooklm | 0.50 |
| Prove that the square root of 2 is irrational | claude | 0.36 |
| Make a lo-fi beat song about rainy mornings | suno | 0.96 |
| Build a SaaS app for invoicing with Stripe and auth | lovable | 0.51 |
| Write a Python function that parses ISO dates and add unit tests | claude | 0.59 |
| Compare Postgres vs MySQL for a small startup, with sources | perplexity | 0.55 |
| Make a 10 second video of waves crashing on a cliff | runway | 0.96 |
| Narrate this paragraph in a calm British voice | elevenlabs | 0.85 |
| Analyze this CSV of sales and plot revenue by month | chatgpt | 0.52 |
| hey | claude (via fallback) | 0.30 |
| Why is the sky blue? | claude (via fallback) | 0.30 |
| Build me a pricing page and a dashboard mockup | v0 | 0.69 |

Behavior checks: with `v0` disabled the React task goes to `lovable`; a rule `login -> cursor` overrides to `cursor`; mode `rules` with no matching rule gives the fallback; with nothing enabled `id` is `None`.

---

## 10. Launching (`launch.rs`)

1. On launch: validate (empty task, no destination), route, then in confirm mode wait for Enter, otherwise continue.
2. Show the `routing` status for about 700 ms (use `ctx.request_repaint_after`), then execute:
   - **URL method:** replace `{task}` with the percent-encoded task (encode every byte except `A-Z a-z 0-9 - _ . ~`) and run `xdg-open <url>` via `std::process::Command` (spawn, don't wait). If the template has no `{task}`, also copy the task to the clipboard (`ctx.copy_text`).
   - **CLI method:** replace `{task}` with the task, escaping `\`, `"`, `$` and backticks, and copy the resulting command to the clipboard. rawter does not run it.
3. After success: set the `done` status, clear the text area if `clear_after`, push `{task, id}` to history if `keep_history` (cap 50), save settings.

---

## 11. Milestones (send these one at a time)

Each milestone's "done" means the code compiles and does what is listed. The user runs `cargo check` or `cargo run` locally and pastes back only the first error if there is one.

| # | Scope | Files | Done when |
|---|---|---|---|
| M0 | `Cargo.toml`, window setup, theme, empty `App` that shows the placeholder text area and the button | `Cargo.toml`, `main.rs` | window opens at 700x450 in the right colors, fixed size, with the logo as window icon |
| M1 | Router port with all tests from section 9 | `router.rs` | `cargo test` passes all rows in the table |
| M2 | Settings model, defaults, load/save | `settings.rs` | round-trips through JSON, seeds from the catalog |
| M3 | Main screen: gear, text area, button, hint, status states, icons, keyboard handling (no real launching yet; just show the routing status and the `done` state) | `ui_main.rs`, `icons.rs`, edit `main.rs` | Ctrl+Enter shows `Routing to ...` then `Launched in ...`; confirm mode works with Enter/Tab/Esc |
| M4 | Settings screen with all sections from section 8 | `ui_settings.rs`, edit `main.rs` | Ctrl+, opens and closes it, all controls change behavior, saved on close |
| M5 | Real launching | `launch.rs`, edit `main.rs` | URL opens in the browser, CLI command lands in the clipboard, history works |
| M6 | Packaging check. The files are already provided. Only verify the binary name is `rawter`, the window has `app_id` `rawter`, and fix any path mismatch | maybe `main.rs` | `./build-appimage.sh` produces `rawter-x86_64.AppImage` that launches |

**Prompt the user sends for each step:** `Do M<n> from the PRD.` For M1 they also attach `reference/router.js`.

---

## 12. Build and ship (the user does this, no tokens needed)

- **Local:** install Rust (rustup), run `./build-appimage.sh`. It builds the release binary, assembles the AppDir, downloads `appimagetool` and writes `rawter-x86_64.AppImage`. The packaging script was tested with a dummy binary.
- **CI (recommended):** push the project to GitHub. The included workflow builds on Ubuntu 22.04 (older glibc, so the AppImage runs on more distros) and uploads the AppImage as an artifact. Tag `v*` also attaches it to a release.
- **Runtime:** needs a working OpenGL (Mesa or vendor driver) and X11 or Wayland. These are loaded at runtime from the system, so they are not bundled.

---

## 13. Out of scope for v1

Global hotkey, stay on top, hide after launch, launch at login, tray icon, running CLI commands directly, API-based destinations, LLM-based classification, history browser, themes, i18n, auto-update, aarch64.

---

## 14. Known risks

- egui API names drift between releases. Fix only the renamed call.
- The deep-link URL formats in the catalog are best-effort and may be wrong or out of date for some tools. Gemini, DeepSeek, Midjourney, Runway, Suno and ElevenLabs have no prefilled-task link, so they hit the "task copied" path.
- The skill numbers in the catalog are hand-set priors, not benchmarks. They live in one place and are meant to be edited.
- The AppImage was not test-run with the real egui binary yet. Only the packaging step was verified.
