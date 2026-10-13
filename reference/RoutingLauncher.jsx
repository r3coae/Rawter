import { useState, useRef, useEffect } from "react";
import { Settings, Route, Loader2, Check, X, Plus, Trash2 } from "lucide-react";
import { CATALOG, routeTask } from "./router";

// Palette: ink window, graphite surfaces, bone text, one muted steel-blue accent.
const c = {
  desk: "#2b2c30",
  window: "#141416",
  well: "#19191c",
  raised: "#202024",
  line: "#2a2a2f",
  text: "#d8d6d0",
  dim: "#6f6e72",
  accent: "#9fb4d6",
  accentInk: "#101216",
};
const mono = "'JetBrains Mono', 'Geist Mono', ui-monospace, SFMono-Regular, Menlo, Consolas, monospace";

const DEFAULTS = {
  destinations: CATALOG.map((d) => ({ id: d.id, name: d.name, enabled: true, method: d.method, value: d.url })),
  rules: [], // optional keyword overrides; they run before the scorer
  fallback: "claude",
  confirm: false,
  classifier: "auto", // "auto" = scored routing, "rules" = keyword rules only
  optimize: "quality", // "quality" | "speed" | "cost"
  hotkey: "Ctrl+Space",
  launchKey: "ctrl-enter", // "ctrl-enter" | "enter"
  onTop: true,
  hideAfter: true,
  clearAfter: true,
  atLogin: false,
  keepHistory: true,
};

/* ---------- small controls ---------- */

const fieldStyle = {
  background: c.well,
  border: `1px solid ${c.line}`,
  color: c.text,
  colorScheme: "dark",
};

function Toggle({ checked, onChange, label }) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={label}
      onClick={() => onChange(!checked)}
      className="relative shrink-0 rounded-full transition-colors focus:outline-none focus-visible:ring-2"
      style={{ width: 30, height: 18, background: checked ? c.accent : "#34343a", "--tw-ring-color": c.accent }}
    >
      <span
        className="absolute rounded-full transition-all"
        style={{
          top: 3,
          left: checked ? 15 : 3,
          width: 12,
          height: 12,
          background: checked ? c.accentInk : c.dim,
        }}
      />
    </button>
  );
}

function Select({ value, onChange, options, className = "", label }) {
  return (
    <select
      aria-label={label}
      value={value}
      onChange={(e) => onChange(e.target.value)}
      className={`h-8 rounded-md px-2 text-[13px] focus:outline-none focus:border-[#3d4658] ${className}`}
      style={fieldStyle}
    >
      {options.map((o) => (
        <option key={o.value} value={o.value}>
          {o.label}
        </option>
      ))}
    </select>
  );
}

function TextField({ className = "", mono: isMono, ...props }) {
  return (
    <input
      spellCheck={false}
      {...props}
      className={`h-8 rounded-md px-2.5 text-[13px] focus:outline-none focus:border-[#3d4658] ${className}`}
      style={{ ...fieldStyle, fontFamily: isMono ? mono : undefined }}
    />
  );
}

function Row({ label, hint, children }) {
  return (
    <div className="flex items-center justify-between gap-6 py-1.5">
      <div className="min-w-0">
        <div className="text-[13px]" style={{ color: c.text }}>{label}</div>
        {hint && <div className="text-xs mt-0.5" style={{ color: c.dim }}>{hint}</div>}
      </div>
      <div className="shrink-0">{children}</div>
    </div>
  );
}

function Section({ title, hint, children }) {
  return (
    <section className="py-5" style={{ borderTop: `1px solid ${c.line}` }}>
      <h3 className="text-[13px] font-semibold" style={{ color: c.text }}>{title}</h3>
      {hint && <p className="text-xs mt-0.5 mb-3" style={{ color: c.dim }}>{hint}</p>}
      {!hint && <div className="mb-3" />}
      {children}
    </section>
  );
}

function Kbd({ children }) {
  return (
    <kbd
      className="px-1.5 py-0.5 rounded text-[11px]"
      style={{ background: c.well, border: `1px solid ${c.line}`, color: c.text, fontFamily: mono }}
    >
      {children}
    </kbd>
  );
}

function HotkeyField({ value, onChange }) {
  const [rec, setRec] = useState(false);
  const onKeyDown = (e) => {
    if (!rec) return;
    e.preventDefault();
    e.stopPropagation();
    if (e.key === "Escape") return setRec(false);
    if (["Control", "Shift", "Alt", "Meta"].includes(e.key)) return;
    if (!(e.ctrlKey || e.metaKey || e.altKey)) return;
    const parts = [];
    if (e.ctrlKey) parts.push("Ctrl");
    if (e.metaKey) parts.push("Cmd");
    if (e.altKey) parts.push("Alt");
    if (e.shiftKey) parts.push("Shift");
    parts.push(e.key === " " ? "Space" : e.key.length === 1 ? e.key.toUpperCase() : e.key);
    onChange(parts.join("+"));
    setRec(false);
  };
  return (
    <button
      type="button"
      onClick={() => setRec(true)}
      onBlur={() => setRec(false)}
      onKeyDown={onKeyDown}
      className="h-8 min-w-[130px] rounded-md px-2.5 text-[13px] text-left focus:outline-none focus:border-[#3d4658]"
      style={{ ...fieldStyle, fontFamily: mono, color: rec ? c.accent : c.text }}
    >
      {rec ? "Press keys..." : value}
    </button>
  );
}

/* ---------- settings overlay ---------- */

function SettingsPanel({ s, set, setDest, setRule, addRule, delRule, historyCount, clearHistory, onClose }) {
  const destOptions = s.destinations.map((d) => ({ value: d.id, label: d.name }));
  return (
    <div
      className="absolute inset-0 z-10 flex flex-col rl-in"
      style={{ background: c.window }}
      role="dialog"
      aria-label="Settings"
    >
      <div className="flex items-center justify-between px-6 h-12 shrink-0">
        <h2 className="text-sm font-semibold" style={{ color: c.text }}>Settings</h2>
        <div className="flex items-center gap-2.5">
          <Kbd>Esc</Kbd>
          <button
            type="button"
            aria-label="Close settings"
            onClick={onClose}
            autoFocus
            className="p-1.5 rounded-md transition-colors focus:outline-none focus-visible:ring-2"
            style={{ color: c.dim, "--tw-ring-color": c.accent }}
            onMouseEnter={(e) => (e.currentTarget.style.color = c.text)}
            onMouseLeave={(e) => (e.currentTarget.style.color = c.dim)}
          >
            <X size={16} strokeWidth={1.75} />
          </button>
        </div>
      </div>

      <div className="flex-1 min-h-0 overflow-y-auto px-6 pb-4 rl-scroll">
        <Section title="Destinations" hint="Tools tasks can be sent to. Use {task} where the task text goes.">
          <div className="space-y-2">
            {s.destinations.map((d) => (
              <div key={d.id} className="flex items-center gap-3">
                <Toggle checked={d.enabled} onChange={(v) => setDest(d.id, { enabled: v })} label={`Enable ${d.name}`} />
                <span className="w-28 shrink-0 truncate text-[13px]" style={{ color: d.enabled ? c.text : c.dim }}>
                  {d.name}
                </span>
                <Select
                  label={`${d.name} launch method`}
                  value={d.method}
                  onChange={(v) => setDest(d.id, { method: v })}
                  options={[
                    { value: "url", label: "URL" },
                    { value: "cli", label: "CLI" },
                    { value: "api", label: "API" },
                  ]}
                  className="w-[68px]"
                />
                <TextField
                  mono
                  aria-label={`${d.name} launch target`}
                  value={d.value}
                  onChange={(e) => setDest(d.id, { value: e.target.value })}
                  className="flex-1 min-w-0"
                />
              </div>
            ))}
          </div>
        </Section>

        <Section title="Routing" hint="How a task gets matched to a destination.">
          <Row label="Confirm before launch" hint="Shows the chosen destination first. Enter confirms, Tab shows the next best.">
            <Toggle checked={s.confirm} onChange={(v) => set({ confirm: v })} label="Confirm before launch" />
          </Row>
          <Row label="Fallback destination" hint="Used when no rule matches.">
            <Select label="Fallback destination" value={s.fallback} onChange={(v) => set({ fallback: v })} options={destOptions} className="w-36" />
          </Row>
          <Row label="Routing mode" hint="Auto scores every enabled destination against the task.">
            <Select
              label="Routing mode"
              value={s.classifier}
              onChange={(v) => set({ classifier: v })}
              options={[
                { value: "auto", label: "Auto (scored)" },
                { value: "rules", label: "Keyword rules only" },
              ]}
              className="w-40"
            />
          </Row>
          <Row label="Optimize for" hint="Nudges close calls toward faster or cheaper tools.">
            <Select
              label="Optimize for"
              value={s.optimize}
              onChange={(v) => set({ optimize: v })}
              options={[
                { value: "quality", label: "Quality" },
                { value: "speed", label: "Speed" },
                { value: "cost", label: "Cost" },
              ]}
              className="w-40"
            />
          </Row>

          <div className="mt-3 space-y-2">
            <div className="text-xs" style={{ color: c.dim }}>
              Override rules run before the scorer, top to bottom. Separate keywords with commas.
            </div>
            {s.rules.map((r) => (
              <div key={r.id} className="flex items-center gap-2">
                <TextField
                  aria-label="Rule keywords"
                  placeholder="keywords, like, these"
                  value={r.keywords}
                  onChange={(e) => setRule(r.id, { keywords: e.target.value })}
                  className="flex-1 min-w-0"
                />
                <Select label="Rule destination" value={r.target} onChange={(v) => setRule(r.id, { target: v })} options={destOptions} className="w-32" />
                <button
                  type="button"
                  aria-label="Delete rule"
                  onClick={() => delRule(r.id)}
                  className="p-1.5 rounded-md focus:outline-none focus-visible:ring-2"
                  style={{ color: c.dim, "--tw-ring-color": c.accent }}
                  onMouseEnter={(e) => (e.currentTarget.style.color = c.text)}
                  onMouseLeave={(e) => (e.currentTarget.style.color = c.dim)}
                >
                  <Trash2 size={14} strokeWidth={1.75} />
                </button>
              </div>
            ))}
            <button
              type="button"
              onClick={addRule}
              className="inline-flex items-center gap-1.5 h-8 px-2.5 rounded-md text-[13px] focus:outline-none focus-visible:ring-2"
              style={{ color: c.accent, "--tw-ring-color": c.accent }}
            >
              <Plus size={14} strokeWidth={2} />
              Add rule
            </button>
          </div>
        </Section>

        <Section title="Shortcuts">
          <Row label="Summon window" hint="Global hotkey. Click, then press the new combination.">
            <HotkeyField value={s.hotkey} onChange={(v) => set({ hotkey: v })} />
          </Row>
          <Row label="Launch key" hint="With Enter, use Shift + Enter for a new line.">
            <Select
              label="Launch key"
              value={s.launchKey}
              onChange={(v) => set({ launchKey: v })}
              options={[
                { value: "ctrl-enter", label: "Ctrl + Enter" },
                { value: "enter", label: "Enter" },
              ]}
              className="w-36"
            />
          </Row>
        </Section>

        <Section title="Window">
          <Row label="Stay on top"><Toggle checked={s.onTop} onChange={(v) => set({ onTop: v })} label="Stay on top" /></Row>
          <Row label="Hide after launch"><Toggle checked={s.hideAfter} onChange={(v) => set({ hideAfter: v })} label="Hide after launch" /></Row>
          <Row label="Clear input after launch"><Toggle checked={s.clearAfter} onChange={(v) => set({ clearAfter: v })} label="Clear input after launch" /></Row>
          <Row label="Launch at login"><Toggle checked={s.atLogin} onChange={(v) => set({ atLogin: v })} label="Launch at login" /></Row>
        </Section>

        <Section title="History" hint="Recent tasks stay on this device.">
          <Row label="Keep task history">
            <Toggle checked={s.keepHistory} onChange={(v) => set({ keepHistory: v })} label="Keep task history" />
          </Row>
          <Row label={`${historyCount} saved ${historyCount === 1 ? "task" : "tasks"}`}>
            <button
              type="button"
              onClick={clearHistory}
              disabled={!historyCount}
              className="h-8 px-3 rounded-md text-[13px] disabled:opacity-40 focus:outline-none focus-visible:ring-2"
              style={{ ...fieldStyle, "--tw-ring-color": c.accent }}
            >
              Clear history
            </button>
          </Row>
        </Section>
      </div>
    </div>
  );
}

/* ---------- main window ---------- */

export default function RoutingLauncher() {
  const [s, setS] = useState(DEFAULTS);
  const [open, setOpen] = useState(false);
  const [task, setTask] = useState("");
  const [status, setStatus] = useState(null); // { phase: routing | done | confirm | empty | none, id? }
  const [history, setHistory] = useState([]);
  const timer = useRef(null);
  const taRef = useRef(null);

  const set = (patch) => setS((p) => ({ ...p, ...patch }));
  const setDest = (id, patch) =>
    setS((p) => ({ ...p, destinations: p.destinations.map((d) => (d.id === id ? { ...d, ...patch } : d)) }));
  const setRule = (id, patch) =>
    setS((p) => ({ ...p, rules: p.rules.map((r) => (r.id === id ? { ...r, ...patch } : r)) }));
  const addRule = () =>
    setS((p) => ({ ...p, rules: [...p.rules, { id: Date.now(), keywords: "", target: p.destinations[0].id }] }));
  const delRule = (id) => setS((p) => ({ ...p, rules: p.rules.filter((r) => r.id !== id) }));

  const nameOf = (id) => s.destinations.find((d) => d.id === id)?.name ?? "";

  useEffect(() => () => clearTimeout(timer.current), []);

  const closeSettings = () => {
    setOpen(false);
    setTimeout(() => taRef.current?.focus(), 0);
  };

  const go = (id, meta = {}) => {
    clearTimeout(timer.current);
    setStatus({ phase: "routing", id, labels: meta.labels ?? [] });
    if (s.keepHistory) setHistory((h) => [{ task, id }, ...h].slice(0, 50));
    timer.current = setTimeout(() => {
      setStatus({ phase: "done", id });
      if (s.clearAfter) setTask("");
    }, 1400);
  };

  const launch = () => {
    clearTimeout(timer.current);
    if (!task.trim()) return setStatus({ phase: "empty" });
    const r = routeTask(task, s);
    if (!r.id) return setStatus({ phase: "none" });
    if (s.confirm) return setStatus({ phase: "confirm", id: r.id, ranked: r.ranked, labels: r.labels });
    go(r.id, r);
  };

  // Window-level keys: open/close settings, confirm step.
  useEffect(() => {
    const onKey = (e) => {
      if ((e.ctrlKey || e.metaKey) && e.key === ",") {
        e.preventDefault();
        return open ? closeSettings() : setOpen(true);
      }
      if (e.key === "Escape") {
        if (open) closeSettings();
        else if (status?.phase === "confirm") setStatus(null);
        return;
      }
      if (!open && status?.phase === "confirm") {
        if (e.key === "Enter") {
          e.preventDefault();
          go(status.id, status);
        } else if (e.key === "Tab") {
          e.preventDefault();
          const list = status.ranked;
          const i = list.indexOf(status.id);
          setStatus({ ...status, id: list[(i + 1) % list.length] });
        }
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  const onTextKey = (e) => {
    if (status?.phase === "confirm") return;
    const combo = e.ctrlKey || e.metaKey;
    const fire = s.launchKey === "enter" ? e.key === "Enter" && !e.shiftKey : combo && e.key === "Enter";
    if (fire) {
      e.preventDefault();
      launch();
    }
  };

  return (
    <div
      className="w-full min-h-screen flex items-center justify-center p-6"
      style={{
        background: c.desk,
        fontFamily: "'Geist', 'Inter', ui-sans-serif, system-ui, -apple-system, 'Segoe UI', sans-serif",
      }}
    >
      <style>{`
        .rl-ta::placeholder { color: ${c.dim}; opacity: .8; }
        .rl-scroll { scrollbar-width: thin; scrollbar-color: #3a3a40 transparent; }
        @keyframes rlIn { from { opacity: 0; transform: translateY(4px); } to { opacity: 1; transform: none; } }
        .rl-in { animation: rlIn .14s ease-out; }
        @media (prefers-reduced-motion: reduce) { .rl-in { animation: none; } }
      `}</style>

      <div
        className="relative flex flex-col overflow-hidden shrink-0"
        style={{
          width: 700,
          height: 450,
          background: c.window,
          border: `1px solid ${c.line}`,
          borderRadius: 10,
          boxShadow: "0 24px 60px -12px rgba(0,0,0,0.65), 0 2px 8px rgba(0,0,0,0.4)",
        }}
      >
        <button
          type="button"
          aria-label="Settings (Ctrl + ,)"
          onClick={() => setOpen(true)}
          className="absolute top-3 right-3 p-1.5 rounded-md transition-colors focus:outline-none focus-visible:ring-2"
          style={{ color: c.dim, "--tw-ring-color": c.accent }}
          onMouseEnter={(e) => (e.currentTarget.style.color = c.text)}
          onMouseLeave={(e) => (e.currentTarget.style.color = c.dim)}
        >
          <Settings size={16} strokeWidth={1.75} />
        </button>

        <div className="flex-1 min-h-0 px-6 pt-12 pb-4">
          <textarea
            ref={taRef}
            autoFocus
            value={task}
            onChange={(e) => {
              setTask(e.target.value);
              if (status?.phase === "confirm") setStatus(null);
            }}
            onKeyDown={onTextKey}
            spellCheck={false}
            placeholder="What do you need done? (e.g., Build a React login screen...)"
            className="rl-ta w-full h-full resize-none rounded-lg px-4 py-3.5 text-[15px] leading-relaxed focus:outline-none"
            style={{
              background: c.well,
              color: c.text,
              border: `1px solid ${c.line}`,
              caretColor: c.accent,
              fontFamily: mono,
            }}
            onFocus={(e) => (e.currentTarget.style.borderColor = "#3d4658")}
            onBlur={(e) => (e.currentTarget.style.borderColor = c.line)}
          />
        </div>

        <div className="px-6 pb-5">
          <div className="flex items-center gap-4">
            <button
              type="button"
              onClick={launch}
              className="inline-flex items-center gap-2 px-5 h-10 rounded-lg text-sm font-semibold transition-[filter] hover:brightness-110 active:brightness-95 focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2"
              style={{
                background: c.accent,
                color: c.accentInk,
                "--tw-ring-color": c.accent,
                "--tw-ring-offset-color": c.window,
              }}
            >
              <Route size={16} strokeWidth={2.25} />
              Route &amp; Launch
            </button>

            <div
              role="status"
              aria-live="polite"
              className="flex items-center gap-2 text-[13px] min-h-[24px] min-w-0"
              style={{ color: c.dim, fontFamily: mono }}
            >
              {status?.phase === "routing" && (
                <>
                  <Loader2 size={14} className="animate-spin" style={{ color: c.accent }} />
                  <span className="shrink-0" style={{ color: c.text }}>Routing to {nameOf(status.id)}...</span>
                  {status.labels?.length > 0 && <span className="truncate">{status.labels.join(" + ")}</span>}
                </>
              )}
              {status?.phase === "done" && (
                <>
                  <Check size={14} style={{ color: c.accent }} />
                  <span style={{ color: c.text }}>Launched in {nameOf(status.id)}</span>
                </>
              )}
              {status?.phase === "confirm" && (
                <>
                  <span className="shrink-0" style={{ color: c.text }}>Route to {nameOf(status.id)}?</span>
                  <span className="truncate">Enter to confirm, Tab for next best</span>
                </>
              )}
              {status?.phase === "empty" && <span>Describe a task first.</span>}
              {status?.phase === "none" && <span>No destination enabled. Check Settings.</span>}
            </div>
          </div>

          <p className="mt-2.5 text-xs" style={{ color: c.dim }}>
            Press <Kbd>{s.launchKey === "enter" ? "Enter" : "Ctrl + Enter"}</Kbd> to launch
            {s.launchKey === "enter" && <> (Shift + Enter for a new line)</>}
          </p>
        </div>

        {open && (
          <SettingsPanel
            s={s}
            set={set}
            setDest={setDest}
            setRule={setRule}
            addRule={addRule}
            delRule={delRule}
            historyCount={history.length}
            clearHistory={() => setHistory([])}
            onClose={closeSettings}
          />
        )}
      </div>
    </div>
  );
}
