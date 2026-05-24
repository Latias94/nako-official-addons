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

    let mut variants = Vec::new();
    push_search_title_variant(&mut variants, raw);
    if let Some(stripped) = strip_trailing_qualifiers(raw) {
        push_search_title_variant(&mut variants, stripped);
    }
    let normalized = normalize_search_title(raw);
    push_search_title_variant(&mut variants, &normalized);
    if let Some(stripped) = strip_trailing_qualifiers(raw) {
        let stripped_normalized = normalize_search_title(stripped);
        push_search_title_variant(&mut variants, &stripped_normalized);
    }
    variants
}

fn push_search_title_variant(values: &mut Vec<String>, value: &str) {
    let value = value.trim();
    if value.is_empty()
        || values
            .iter()
            .any(|existing| existing.eq_ignore_ascii_case(value))
    {
        return;
    }
    values.push(value.to_owned());
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

fn strip_trailing_qualifiers(value: &str) -> Option<&str> {
    let mut stripped = value.trim_end();
    let original = stripped;
    while let Some(next) = strip_one_trailing_qualifier(stripped) {
        stripped = next;
    }
    if stripped == original {
        None
    } else {
        Some(stripped)
    }
}

fn strip_one_trailing_qualifier(value: &str) -> Option<&str> {
    let value = value.trim_end();
    let (open, close) = match value.chars().next_back()? {
        ')' => ('(', ')'),
        ']' => ('[', ']'),
        _ => return None,
    };
    let close_index = value.len() - close.len_utf8();
    let open_index = value[..close_index].rfind(open)?;
    let qualifier = value[open_index + open.len_utf8()..close_index].trim();
    if qualifier.is_empty() {
        return None;
    }
    let stripped = value[..open_index].trim_end();
    if stripped.is_empty() {
        return None;
    }
    Some(stripped)
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
    fn search_title_variants_include_trailing_qualifier_stripped_forms() {
        assert_eq!(
            search_title_variants("The Matrix (1999)"),
            vec![
                "The Matrix (1999)".to_owned(),
                "The Matrix".to_owned(),
                "the matrix 1999".to_owned()
            ]
        );
        assert_eq!(
            search_title_variants("Movie [Director's Cut]"),
            vec![
                "Movie [Director's Cut]".to_owned(),
                "Movie".to_owned(),
                "movie director s cut".to_owned()
            ]
        );
        assert_eq!(
            search_title_variants("The Matrix (1999) [1080p]"),
            vec![
                "The Matrix (1999) [1080p]".to_owned(),
                "The Matrix".to_owned(),
                "the matrix 1999 1080p".to_owned()
            ]
        );
    }

    #[test]
    fn search_title_variants_skip_case_only_duplicates() {
        assert_eq!(search_title_variants("The Matrix"), vec!["The Matrix"]);
    }
}
