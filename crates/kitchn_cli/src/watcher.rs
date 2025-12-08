//! Debug log watcher utilities

use colored::{ColoredString, Colorize};

/// Determines the log level from a line and returns a colorized version.
/// - ERROR -> Red
/// - WARN -> Yellow
/// - INFO -> Cyan
/// - DEBUG -> Blue
/// - Other -> Normal
pub fn colorize_line(line: &str) -> ColoredString {
    if line.contains("ERROR") {
        line.red()
    } else if line.contains("WARN") {
        line.yellow()
    } else if line.contains("INFO") {
        line.cyan()
    } else if line.contains("DEBUG") {
        line.blue()
    } else {
        line.normal()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use colored::Color;

    #[test]
    fn test_colorize_error() {
        let result = colorize_line("[ERROR] Something went wrong");
        assert_eq!(result.fgcolor, Some(Color::Red));
    }

    #[test]
    fn test_colorize_warn() {
        let result = colorize_line("[WARN] Heads up");
        assert_eq!(result.fgcolor, Some(Color::Yellow));
    }

    #[test]
    fn test_colorize_info() {
        let result = colorize_line("[INFO] It's fine");
        assert_eq!(result.fgcolor, Some(Color::Cyan));
    }

    #[test]
    fn test_colorize_debug() {
        let result = colorize_line("[DEBUG] nitty gritty");
        assert_eq!(result.fgcolor, Some(Color::Blue));
    }

    #[test]
    fn test_colorize_normal() {
        let result = colorize_line("Just a plain line");
        assert_eq!(result.fgcolor, None);
    }
}
