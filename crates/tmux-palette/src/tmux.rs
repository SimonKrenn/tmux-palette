use std::process::Command;

pub fn tmux(args: &[&str]) -> String {
    match Command::new("tmux").args(args).output() {
        Ok(output) => String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_string(),
        Err(_) => String::new(),
    }
}

pub fn tmux_quote(value: &str) -> String {
    format!("'{}'", value.replace("'", "'\\''"))
}
