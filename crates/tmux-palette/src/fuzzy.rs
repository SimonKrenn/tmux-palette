use crate::{model::Item, text::auto_alias};

const ALIAS_EXACT_BOOST: i32 = 100_000;

fn is_boundary(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '-' | '_' | '·' | '.' | '/' | ':')
}

fn exact_match_score(haystack: &str, needle: &str) -> i32 {
    let Some(idx) = haystack.find(needle) else {
        return 0;
    };
    let at_boundary = idx == 0 || haystack[..idx].chars().last().is_some_and(is_boundary);
    10_000 + if at_boundary { 5_000 } else { 0 } - idx as i32
}

fn char_bonus(at_boundary: bool, consecutive: bool) -> i32 {
    if at_boundary {
        50
    } else if consecutive {
        20
    } else {
        5
    }
}

fn subsequence_score(haystack: &str, needle: &str) -> i32 {
    let hs: Vec<char> = haystack.chars().collect();
    let nd: Vec<char> = needle.chars().collect();
    let mut score = 0;
    let mut h = 0usize;
    let mut prev: isize = -2;

    for target in nd {
        while h < hs.len() && hs[h] != target {
            h += 1;
        }
        if h >= hs.len() {
            return 0;
        }
        let at_boundary = h == 0 || is_boundary(hs[h - 1]);
        score += char_bonus(at_boundary, h as isize == prev + 1);
        prev = h as isize;
        h += 1;
    }

    score.max(1)
}

fn fuzzy_score(haystack: &str, needle: &str) -> i32 {
    if needle.is_empty() {
        return 1;
    }
    let hs = haystack.to_lowercase();
    let nd = needle.to_lowercase();
    let exact = exact_match_score(&hs, &nd);
    if exact > 0 {
        exact
    } else {
        subsequence_score(&hs, &nd)
    }
}

pub fn multi_fuzzy_score(haystack: &str, parts: &[&str]) -> i32 {
    let mut total = 0;
    for part in parts {
        let score = fuzzy_score(haystack, part);
        if score == 0 {
            return 0;
        }
        total += score;
    }
    total
}

fn build_item_haystack(item: &Item) -> String {
    let mut parts = Vec::new();
    parts.push(item.title.as_str());
    if let Some(description) = &item.description {
        parts.push(description);
    }
    if let Some(category) = &item.category {
        parts.push(category);
    }
    if let Some(shortcut) = &item.shortcut {
        parts.push(shortcut);
    }
    for alias in &item.aliases {
        parts.push(alias);
    }
    let auto = auto_alias(&item.title);
    if let Some(auto) = auto.as_deref() {
        parts.push(auto);
    }
    parts.join(" ")
}

fn alias_exact_boost(item: &Item, parts: &[&str]) -> i32 {
    if parts.len() != 1 {
        return 0;
    }
    let query = parts[0].to_lowercase();
    if auto_alias(&item.title).is_some_and(|a| a == query) {
        return ALIAS_EXACT_BOOST;
    }
    if item.aliases.iter().any(|a| a.to_lowercase() == query) {
        return ALIAS_EXACT_BOOST;
    }
    0
}

pub fn default_filter(items: &[Item], needle: &str) -> Vec<Item> {
    let parts: Vec<&str> = needle.split_whitespace().collect();
    let mut scored: Vec<(Item, i32)> = items
        .iter()
        .cloned()
        .map(|item| {
            let score = multi_fuzzy_score(&build_item_haystack(&item), &parts)
                + alias_exact_boost(&item, &parts);
            (item, score)
        })
        .filter(|(_, score)| *score > 0)
        .collect();
    scored.sort_by(|a, b| b.1.cmp(&a.1));
    scored.into_iter().map(|(item, _)| item).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Action;

    fn item(title: &str) -> Item {
        Item::new(title, Action::Shell { shell: ":".into() })
    }

    #[test]
    fn multi_score_requires_every_query_part_to_match() {
        assert!(multi_fuzzy_score("split horizontal pane", &["split", "pane"]) > 0);
        assert_eq!(
            multi_fuzzy_score("split horizontal pane", &["split", "window"]),
            0
        );
    }

    #[test]
    fn default_filter_matches_title_initials_through_auto_aliases() {
        let mut split = item("Split Horizontal");
        split.category = Some("Panes".into());
        let mut new_window = item("New Window");
        new_window.category = Some("Windows".into());
        let mut choose = item("Choose Session");
        choose.aliases = vec!["sessions".into()];
        let items = vec![split, new_window, choose];
        let titles: Vec<String> = default_filter(&items, "sh")
            .into_iter()
            .map(|i| i.title)
            .collect();
        assert_eq!(titles, vec!["Split Horizontal"]);
    }

    #[test]
    fn default_filter_matches_explicit_aliases() {
        let mut choose = item("Choose Session");
        choose.aliases = vec!["sessions".into()];
        let titles: Vec<String> = default_filter(&[choose], "sessions")
            .into_iter()
            .map(|i| i.title)
            .collect();
        assert_eq!(titles, vec!["Choose Session"]);
    }

    #[test]
    fn auto_alias_outranks_substring_matches_inside_category() {
        let mut detach = item("Detach");
        detach.category = Some("Sessions".into());
        let mut new_session = item("New Session");
        new_session.category = Some("Sessions".into());
        let mut next_session = item("Next Session");
        next_session.category = Some("Sessions".into());
        let ranked: Vec<String> = default_filter(&[detach, new_session, next_session], "ns")
            .into_iter()
            .map(|i| i.title)
            .collect();
        assert_eq!(ranked[0], "New Session");
        assert_eq!(ranked[1], "Next Session");
    }
}
