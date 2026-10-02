//! Platform-neutral puzzle collection filename rules.

pub fn is_puzzle_collection_filename(name: &str) -> bool {
    if name == "puzzles.json" {
        return true;
    }

    let Some(suffix) = name.strip_prefix("puzzles-") else {
        return false;
    };
    let Some(stem) = suffix.strip_suffix(".json") else {
        return false;
    };

    !stem.is_empty() && !stem.contains('/') && !stem.contains('\\')
}

pub fn sorted_puzzle_collection_filenames<I, S>(names: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut names = names
        .into_iter()
        .map(|name| name.as_ref().to_owned())
        .filter(|name| is_puzzle_collection_filename(name))
        .collect::<Vec<_>>();
    names.sort();
    names.dedup();
    names
}
