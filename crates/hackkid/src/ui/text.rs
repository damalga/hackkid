/// Greedy word wrap to `width` columns. Words longer than a line are split.
pub fn wrap(text: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    let mut lines = Vec::new();
    let mut line = String::new();
    let mut len = 0;
    for word in text.split_whitespace() {
        let mut word: Vec<char> = word.chars().collect();
        while word.len() > width {
            if len > 0 {
                lines.push(std::mem::take(&mut line));
                len = 0;
            }
            lines.push(word.drain(..width).collect());
        }
        let wlen = word.len();
        if len > 0 && len + 1 + wlen > width {
            lines.push(std::mem::take(&mut line));
            len = 0;
        }
        if len > 0 {
            line.push(' ');
            len += 1;
        }
        line.extend(word);
        len += wlen;
    }
    if len > 0 || lines.is_empty() {
        lines.push(line);
    }
    lines
}

/// Cuts `text` to `width` columns, ending in `…` when something was cut.
pub fn ellipsize(text: &str, width: usize) -> String {
    if text.chars().count() <= width {
        return text.to_string();
    }
    let mut s: String = text.chars().take(width.saturating_sub(1)).collect();
    s.push('…');
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_on_words_and_keeps_everything() {
        let text = "Julian: 'Radio towers are dead, but your emergency terminal can pick up a repeating broadcast on 104.2 MHz.'";
        let lines = wrap(text, 30);
        assert!(lines.iter().all(|l| l.chars().count() <= 30));
        assert_eq!(lines.join(" "), text);
    }

    #[test]
    fn splits_words_longer_than_a_line() {
        assert_eq!(wrap("abcdefghij", 4), ["abcd", "efgh", "ij"]);
        assert_eq!(wrap("", 10), [""]);
    }

    #[test]
    fn ellipsizes() {
        assert_eq!(ellipsize("Emergency Terminal", 10), "Emergency…");
        assert_eq!(ellipsize("Coffee", 10), "Coffee");
    }
}
