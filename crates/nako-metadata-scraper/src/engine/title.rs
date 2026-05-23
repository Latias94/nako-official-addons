#[must_use]
pub fn normalize_title(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

#[must_use]
pub fn search_title_variants(value: &str) -> Vec<String> {
    let raw = value.trim();
    if raw.is_empty() {
        return Vec::new();
    }

    let normalized = normalize_search_title(raw);
    if normalized.is_empty() || normalized == raw || normalized.eq_ignore_ascii_case(raw) {
        vec![raw.to_owned()]
    } else {
        vec![raw.to_owned(), normalized]
    }
}

fn normalize_search_title(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_alphanumeric() {
                character
            } else {
                ' '
            }
        })
        .flat_map(char::to_lowercase)
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_title_strips_punctuation_and_lowercases() {
        assert_eq!(
            normalize_title("Spider-Man: No Way Home"),
            "spidermannowayhome"
        );
    }

    #[test]
    fn search_title_variants_include_raw_and_normalized_forms() {
        assert_eq!(
            search_title_variants("Spider-Man: No Way Home"),
            vec![
                "Spider-Man: No Way Home".to_owned(),
                "spider man no way home".to_owned()
            ]
        );
    }

    #[test]
    fn search_title_variants_skip_case_only_duplicates() {
        assert_eq!(search_title_variants("The Matrix"), vec!["The Matrix"]);
    }
}
