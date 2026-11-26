//! Tokenizer: lowercase ASCII letters and digits, apostrophes dropped, stop words removed.
const STOP: &[&str] = &[
    "a", "an", "the", "and", "or", "but", "to", "of", "in", "on", "for", "with", "is", "are", "was", "were", "be", "it", "this",
    "that", "we", "you", "i", "our", "your", "they", "them", "do", "does", "so", "at", "as", "if", "can", "just", "have", "has",
    "me", "my", "what", "how", "about", "would", "will", "yeah", "ok", "okay", "um", "uh", "like", "not", "there", "from", "us",
    "its", "im", "youre", "thats", "dont", "were", "get", "got", "re", "s", "t", "be", "by", "all", "any", "out", "up",
];

pub fn tokens(s: &str) -> Vec<String> {
    let lower = s.to_lowercase().replace(['\'', '’'], "");
    lower.split(|c: char| !c.is_ascii_alphanumeric()).filter(|w| !w.is_empty() && !STOP.contains(w)).map(String::from).collect()
}

pub fn words(s: &str) -> usize {
    s.split_whitespace().count()
}

#[cfg(test)]
mod t {
    #[test]
    fn tok() {
        assert_eq!(super::tokens("We're worried about PRICING, and SSO!"), vec!["worried", "pricing", "sso"]);
    }
}
