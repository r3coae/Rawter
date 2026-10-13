use crate::router::Method;
use crate::settings::DestSetting;
use crate::ui_main::DoneKind;
use std::process::{Command, Stdio};

/// Percent-encode every byte except A-Z a-z 0-9 - _ . ~
pub fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

pub fn url_for(template: &str, task: &str) -> String {
    template.replace("{task}", &percent_encode(task))
}

/// Escape `\`, `"`, `$` and backticks so the task is safe inside double quotes.
pub fn escape_cli(task: &str) -> String {
    let mut out = String::with_capacity(task.len());
    for c in task.chars() {
        if matches!(c, '\\' | '"' | '$' | '`') {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

pub fn cli_command(template: &str, task: &str) -> String {
    template.replace("{task}", &escape_cli(task))
}

/// URL: open with xdg-open (copy the task too if the template has no {task}).
/// CLI: copy the command to the clipboard, never run it.
pub fn run(ctx: &egui::Context, d: &DestSetting, task: &str) -> Result<DoneKind, String> {
    match d.method {
        Method::Cli => {
            ctx.copy_text(cli_command(&d.value, task));
            Ok(DoneKind::Command)
        }
        Method::Url => {
            let url = url_for(&d.value, task);
            Command::new("xdg-open")
                .arg(&url)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|e| e.to_string())?;
            if d.value.contains("{task}") {
                Ok(DoneKind::Launched)
            } else {
                ctx.copy_text(task.to_string());
                Ok(DoneKind::Copied)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_all_but_unreserved() {
        assert_eq!(percent_encode("a b&c=d/é"), "a%20b%26c%3Dd%2F%C3%A9");
        assert_eq!(percent_encode("AZaz09-_.~"), "AZaz09-_.~");
    }

    #[test]
    fn builds_url() {
        assert_eq!(url_for("https://x.test/?q={task}", "hi there"), "https://x.test/?q=hi%20there");
    }

    #[test]
    fn escapes_cli() {
        assert_eq!(cli_command("claude \"{task}\"", r#"say "hi" $HOME `x` \n"#), r#"claude "say \"hi\" \$HOME \`x\` \\n""#);
    }
}
