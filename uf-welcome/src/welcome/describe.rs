//! Plain-words description of a change to the featured list.
//!
//! Shared by the admin page's draft bar and the stored history summary, so the
//! words a curator sees before publishing match what History records.

/// Describe how `after` differs from `before`, naming apps through `name`.
///
/// Returns clauses such as "Added Gauge at the end, removed Notifications" or
/// "Moved Counter above Gauge". Returns "No changes" when the lists match.
pub fn describe_change(
    before: &[String],
    after: &[String],
    name: impl Fn(&str) -> String,
) -> String {
    let mut clauses: Vec<String> = Vec::new();

    for (idx, id) in after.iter().enumerate() {
        if !before.contains(id) {
            if idx + 1 == after.len() && after.len() > 1 {
                clauses.push(format!("added {} at the end", name(id)));
            } else {
                clauses.push(format!("added {}", name(id)));
            }
        }
    }
    for id in before {
        if !after.contains(id) {
            clauses.push(format!("removed {}", name(id)));
        }
    }

    // Apps on both lists: those outside the longest run kept in order were moved.
    let kept_after: Vec<&String> = after.iter().filter(|id| before.contains(id)).collect();
    let before_pos: Vec<usize> = kept_after
        .iter()
        .filter_map(|id| before.iter().position(|b| b == *id))
        .collect();
    let stays = longest_increasing(&before_pos);
    for (idx, id) in kept_after.iter().enumerate() {
        if stays[idx] {
            continue;
        }
        let clause = match kept_after.get(idx + 1) {
            Some(next) => format!("moved {} above {}", name(id), name(next)),
            None => match idx.checked_sub(1).and_then(|p| kept_after.get(p)) {
                Some(prev) => format!("moved {} below {}", name(id), name(prev)),
                None => format!("moved {}", name(id)),
            },
        };
        clauses.push(clause);
    }

    if clauses.is_empty() {
        return "No changes".to_string();
    }
    let joined = clauses.join(", ");
    let mut chars = joined.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => joined,
    }
}

/// Marks which entries of `seq` belong to one longest strictly increasing subsequence.
fn longest_increasing(seq: &[usize]) -> Vec<bool> {
    let n = seq.len();
    let mut len = vec![1usize; n];
    let mut prev = vec![usize::MAX; n];
    for i in 0..n {
        for j in 0..i {
            if seq[j] < seq[i] && len[j] + 1 > len[i] {
                len[i] = len[j] + 1;
                prev[i] = j;
            }
        }
    }
    let mut keep = vec![false; n];
    let mut cur = (0..n).max_by_key(|&i| (len[i], std::cmp::Reverse(i)));
    while let Some(i) = cur {
        keep[i] = true;
        cur = (prev[i] != usize::MAX).then_some(prev[i]);
    }
    keep
}

/// Join app names as "A", "A and B" or "A, B and C".
pub fn join_names(names: &[String]) -> String {
    match names {
        [] => String::new(),
        [one] => one.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_string()).collect()
    }

    fn title(id: &str) -> String {
        let mut c = id.chars();
        c.next()
            .map(|f| f.to_uppercase().chain(c).collect())
            .unwrap_or_default()
    }

    #[test]
    fn describe_added_at_end_happy() {
        let out = describe_change(
            &ids(&["magnon", "counter"]),
            &ids(&["magnon", "counter", "gauge"]),
            title,
        );
        assert_eq!(out, "Added Gauge at the end");
    }

    #[test]
    fn describe_added_to_empty_list_happy() {
        assert_eq!(describe_change(&[], &ids(&["gauge"]), title), "Added Gauge");
    }

    #[test]
    fn describe_removed_happy() {
        let out = describe_change(&ids(&["notes", "magnon"]), &ids(&["magnon"]), title);
        assert_eq!(out, "Removed Notes");
    }

    #[test]
    fn describe_moved_happy() {
        let out = describe_change(
            &ids(&["magnon", "gauge", "counter"]),
            &ids(&["magnon", "counter", "gauge"]),
            title,
        );
        assert_eq!(out, "Moved Gauge below Counter");
        let out = describe_change(&ids(&["a", "b", "c"]), &ids(&["c", "a", "b"]), title);
        assert_eq!(out, "Moved C above A");
    }

    #[test]
    fn describe_mixed_happy() {
        let out = describe_change(&ids(&["a", "b"]), &ids(&["b", "c", "a"]), title);
        assert_eq!(out, "Added C, moved A below B");
    }

    #[test]
    fn describe_no_change_sad() {
        assert_eq!(
            describe_change(&ids(&["a"]), &ids(&["a"]), title),
            "No changes"
        );
        assert_eq!(describe_change(&[], &[], title), "No changes");
    }

    #[test]
    fn join_names_happy() {
        assert_eq!(join_names(&[]), "");
        assert_eq!(join_names(&ids(&["A"])), "A");
        assert_eq!(join_names(&ids(&["A", "B"])), "A and B");
        assert_eq!(join_names(&ids(&["A", "B", "C"])), "A, B and C");
    }
}
