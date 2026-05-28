//! Display renderer — message formatting and colors
//!
//! Handles the visual presentation of avatar messages,
//! including colored text, ASCII decorations, and layouts.

/// Terminal color constants
pub struct Colors;

impl Colors {
    pub const RESET: &'static str = "\x1b[0m";
    pub const BOLD: &'static str = "\x1b[1m";
    pub const DIM: &'static str = "\x1b[2m";
    pub const CYAN: &'static str = "\x1b[36m";
    pub const GREEN: &'static str = "\x1b[32m";
    pub const YELLOW: &'static str = "\x1b[33m";
    pub const MAGENTA: &'static str = "\x1b[35m";
    pub const BLUE: &'static str = "\x1b[34m";
    pub const RED: &'static str = "\x1b[31m";
    pub const WHITE: &'static str = "\x1b[37m";
}

/// Color lookup for roles
fn role_color(role: &str) -> &'static str {
    match role {
        "user" => Colors::GREEN,
        "assistant" => Colors::CYAN,
        "system" => Colors::DIM,
        "error" => Colors::RED,
        _ => Colors::WHITE,
    }
}

/// The DisplayRenderer — formats and renders messages
pub struct DisplayRenderer;

impl DisplayRenderer {
    pub fn new() -> Self {
        Self
    }

    /// Show a formatted message
    pub fn show_message(&self, role: &str, name: &str, content: &str) {
        let color = role_color(role);
        let prefix = match role {
            "user" => "You",
            "assistant" => name,
            "system" => "System",
            _ => role,
        };

        println!("\n{}{} {}{}", color, Colors::BOLD, prefix, Colors::RESET);
        for line in content.lines() {
            println!("  {} {}", color, line);
        }
        println!("{}", Colors::RESET);
    }

    /// Show a thinking indicator
    pub fn show_thinking(&self, _name: &str) {
        print!("{}  Thinking{}", Colors::DIM, Colors::RESET);
        std::io::Write::flush(&mut std::io::stdout()).ok();
        // In a real TUI, this would have animated dots
        println!();
    }

    /// Render a separator line
    pub fn separator() {
        println!("{}┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈{}", Colors::DIM, Colors::RESET);
    }

    /// Format a header box
    pub fn header(title: &str) -> String {
        format!(
            "\n╔══════════════════════════════════════════════╗\n\
             ║  {:<46} ║\n\
             ╚══════════════════════════════════════════════╝\n",
            title
        )
    }

    /// Truncate text with ellipsis
    pub fn truncate(text: &str, max_len: usize) -> String {
        if text.len() <= max_len {
            text.to_string()
        } else {
            format!("{}...", &text[..max_len - 3])
        }
    }
}
