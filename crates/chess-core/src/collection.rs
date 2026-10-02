//! Platform-neutral puzzle collection filename and picker metadata rules.

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollectionEntry {
    key: String,
    label: String,
    error: Option<String>,
}

impl CollectionEntry {
    pub fn valid(key: impl Into<String>, label: impl Into<String>) -> Self {
        let key = key.into();
        let label = label.into();
        let label = if label.trim().is_empty() {
            key.clone()
        } else {
            label
        };
        Self {
            key,
            label,
            error: None,
        }
    }

    pub fn invalid(key: impl Into<String>, error: impl Into<String>) -> Self {
        let key = key.into();
        Self {
            label: key.clone(),
            key,
            error: Some(error.into()),
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub const fn is_valid(&self) -> bool {
        self.error.is_none()
    }
}

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
