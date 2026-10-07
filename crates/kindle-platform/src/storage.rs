//! Kindle filesystem adapter for puzzle/review libraries and durable progress/settings.

use chess_core::{
    is_puzzle_collection_filename, parse_puzzle_file, parse_review_file,
    sorted_puzzle_collection_filenames, CollectionEntry, Progress, PuzzleCollection,
    ReviewCollection, ReviewFileError, ReviewGame, ReviewGameEntry, ReviewGameKey, Settings,
};
use std::{
    env, fmt,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    process,
    sync::atomic::{AtomicU64, Ordering},
};

pub const DEFAULT_PUZZLE_DIR: &str = "/mnt/us/kindle-chess/puzzles";
pub const DEFAULT_REVIEW_DIR: &str = "/mnt/us/kindle-chess/games";
pub const DEFAULT_PROGRESS_FILE: &str = "/mnt/us/kindle-chess/state/progress.json";
pub const DEFAULT_SETTINGS_FILE: &str = "/mnt/us/kindle-chess/state/settings.json";
pub const PUZZLE_DIR_ENV: &str = "KINDLE_CHESS_PUZZLE_DIR";
pub const REVIEW_DIR_ENV: &str = "KINDLE_CHESS_REVIEW_DIR";
pub const PROGRESS_FILE_ENV: &str = "KINDLE_CHESS_PROGRESS_FILE";
pub const SETTINGS_FILE_ENV: &str = "KINDLE_CHESS_SETTINGS_FILE";

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoragePaths {
    pub puzzle_dir: PathBuf,
    pub review_dir: PathBuf,
    pub progress_file: PathBuf,
    pub settings_file: PathBuf,
}

impl StoragePaths {
    pub fn new(puzzle_dir: impl Into<PathBuf>, progress_file: impl Into<PathBuf>) -> Self {
        let puzzle_dir = puzzle_dir.into();
        let review_dir = puzzle_dir
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."))
            .join("games");
        let progress_file = progress_file.into();
        let settings_file = progress_file
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("settings.json");
        Self {
            puzzle_dir,
            review_dir,
            progress_file,
            settings_file,
        }
    }

    pub fn kindle_default() -> Self {
        let mut paths = Self::new(DEFAULT_PUZZLE_DIR, DEFAULT_PROGRESS_FILE);
        paths.review_dir = PathBuf::from(DEFAULT_REVIEW_DIR);
        paths
    }

    pub fn from_env() -> Self {
        let defaults = Self::kindle_default();
        let puzzle_dir = env::var_os(PUZZLE_DIR_ENV)
            .map(PathBuf::from)
            .unwrap_or(defaults.puzzle_dir);
        let review_dir = env::var_os(REVIEW_DIR_ENV)
            .map(PathBuf::from)
            .unwrap_or(defaults.review_dir);
        let progress_file = env::var_os(PROGRESS_FILE_ENV)
            .map(PathBuf::from)
            .unwrap_or(defaults.progress_file);
        let mut paths = Self::new(puzzle_dir, progress_file);
        paths.review_dir = review_dir;
        if let Some(settings_file) = env::var_os(SETTINGS_FILE_ENV) {
            paths.settings_file = PathBuf::from(settings_file);
        }
        paths
    }
}

#[derive(Debug)]
pub enum StorageError {
    Io(String),
    InvalidCollection { filename: String, error: String },
    InvalidReviewFile { filename: String, error: String },
    Progress(String),
    Settings(String),
    ProtectedProgress { path: PathBuf, reason: String },
    ProtectedSettings { path: PathBuf, reason: String },
}

impl fmt::Display for StorageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(message) | Self::Progress(message) | Self::Settings(message) => {
                formatter.write_str(message)
            }
            Self::InvalidCollection { filename, error } => {
                write!(formatter, "{filename}: {error}")
            }
            Self::InvalidReviewFile { filename, error } => {
                write!(formatter, "{filename}: {error}")
            }
            Self::ProtectedProgress { path, reason } => write!(
                formatter,
                "refusing to overwrite protected progress {}: {reason}",
                path.display()
            ),
            Self::ProtectedSettings { path, reason } => write!(
                formatter,
                "refusing to overwrite protected settings {}: {reason}",
                path.display()
            ),
        }
    }
}

impl std::error::Error for StorageError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveredCollection {
    pub filename: String,
    pub label: String,
    pub error: Option<String>,
}

impl DiscoveredCollection {
    pub const fn is_valid(&self) -> bool {
        self.error.is_none()
    }

    pub fn as_core_entry(&self) -> CollectionEntry {
        match &self.error {
            Some(error) => CollectionEntry::invalid(self.filename.clone(), error.clone()),
            None => CollectionEntry::valid(self.filename.clone(), self.label.clone()),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DiscoveredReviewLibrary {
    pub games: Vec<ReviewGameEntry>,
    pub errors: Vec<ReviewFileError>,
}

#[derive(Clone, Debug)]
pub struct KindleStorage {
    paths: StoragePaths,
}

impl KindleStorage {
    pub const fn new(paths: StoragePaths) -> Self {
        Self { paths }
    }

    pub const fn paths(&self) -> &StoragePaths {
        &self.paths
    }

    pub fn prepare_collections(
        &self,
        bundled_examples: &[u8],
    ) -> Result<Vec<DiscoveredCollection>, StorageError> {
        let mut discovered = self.discover_collections()?;
        if discovered.is_empty() {
            self.install_bundled_default(bundled_examples)?;
            discovered = self.discover_collections()?;
        }
        Ok(discovered)
    }

    pub fn discover_collections(&self) -> Result<Vec<DiscoveredCollection>, StorageError> {
        let names = self.collection_filenames()?;
        Ok(names
            .into_iter()
            .map(|filename| match self.load_collection(&filename) {
                Ok(collection) => {
                    let label = collection
                        .title
                        .as_deref()
                        .map(str::trim)
                        .filter(|title| !title.is_empty())
                        .unwrap_or(&filename)
                        .to_owned();
                    DiscoveredCollection {
                        filename,
                        label,
                        error: None,
                    }
                }
                Err(error) => DiscoveredCollection {
                    filename: filename.clone(),
                    label: filename,
                    error: Some(error.to_string()),
                },
            })
            .collect())
    }

    pub fn discover_review_library(&self) -> Result<DiscoveredReviewLibrary, StorageError> {
        let names = self.review_filenames()?;
        let mut library = DiscoveredReviewLibrary::default();

        for filename in names {
            match self.load_review_collection(&filename) {
                Ok(collection) => {
                    library.games.extend(
                        collection
                            .games
                            .into_iter()
                            .map(|game| ReviewGameEntry::new(filename.clone(), game)),
                    );
                }
                Err(StorageError::InvalidReviewFile { error, .. }) => {
                    library.errors.push(ReviewFileError::new(filename, error));
                }
                Err(error) => {
                    library
                        .errors
                        .push(ReviewFileError::new(filename, error.to_string()));
                }
            }
        }

        Ok(library)
    }

    pub fn load_review_game(&self, key: &ReviewGameKey) -> Result<ReviewGame, StorageError> {
        let collection = self.load_review_collection(key.collection_id())?;
        collection
            .games
            .into_iter()
            .find(|game| game.id == key.game_id())
            .ok_or_else(|| StorageError::InvalidReviewFile {
                filename: key.collection_id().to_owned(),
                error: format!(
                    "game {} is no longer present in the review file",
                    key.game_id()
                ),
            })
    }

    fn load_review_collection(&self, filename: &str) -> Result<ReviewCollection, StorageError> {
        if !is_review_collection_filename(filename) {
            return Err(StorageError::InvalidReviewFile {
                filename: filename.to_owned(),
                error: "filename does not match the review collection convention".to_owned(),
            });
        }

        let path = self.paths.review_dir.join(filename);
        let bytes = fs::read(&path).map_err(|error| StorageError::InvalidReviewFile {
            filename: filename.to_owned(),
            error: format!("could not read review file: {error}"),
        })?;
        parse_review_file(&bytes).map_err(|error| StorageError::InvalidReviewFile {
            filename: filename.to_owned(),
            error,
        })
    }

    pub fn load_collection(&self, filename: &str) -> Result<PuzzleCollection, StorageError> {
        if !is_puzzle_collection_filename(filename) {
            return Err(StorageError::InvalidCollection {
                filename: filename.to_owned(),
                error: "filename does not match the puzzle collection convention".to_owned(),
            });
        }

        let path = self.paths.puzzle_dir.join(filename);
        let bytes = fs::read(&path)
            .map_err(|error| io_error("could not read puzzle collection", &path, error))?;
        parse_puzzle_file(&bytes).map_err(|error| StorageError::InvalidCollection {
            filename: filename.to_owned(),
            error,
        })
    }

    fn collection_filenames(&self) -> Result<Vec<String>, StorageError> {
        let entries = match fs::read_dir(&self.paths.puzzle_dir) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => {
                return Err(io_error(
                    "could not read puzzle directory",
                    &self.paths.puzzle_dir,
                    error,
                ));
            }
        };

        let mut names = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|error| {
                io_error(
                    "could not read puzzle directory entry",
                    &self.paths.puzzle_dir,
                    error,
                )
            })?;
            let file_type = entry.file_type().map_err(|error| {
                io_error(
                    "could not inspect puzzle directory entry",
                    &entry.path(),
                    error,
                )
            })?;
            if !file_type.is_file() {
                continue;
            }
            let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            names.push(name);
        }

        Ok(sorted_puzzle_collection_filenames(names))
    }

    fn review_filenames(&self) -> Result<Vec<String>, StorageError> {
        let entries = match fs::read_dir(&self.paths.review_dir) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => {
                return Err(io_error(
                    "could not read review directory",
                    &self.paths.review_dir,
                    error,
                ));
            }
        };

        let mut names = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|error| {
                io_error(
                    "could not read review directory entry",
                    &self.paths.review_dir,
                    error,
                )
            })?;
            let file_type = entry.file_type().map_err(|error| {
                io_error(
                    "could not inspect review directory entry",
                    &entry.path(),
                    error,
                )
            })?;
            if !file_type.is_file() {
                continue;
            }
            let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            if is_review_collection_filename(&name) {
                names.push(name);
            }
        }

        names.sort();
        names.dedup();
        Ok(names)
    }

    fn install_bundled_default(&self, bundled_examples: &[u8]) -> Result<(), StorageError> {
        parse_puzzle_file(bundled_examples).map_err(|error| StorageError::InvalidCollection {
            filename: "bundled examples".to_owned(),
            error,
        })?;

        fs::create_dir_all(&self.paths.puzzle_dir).map_err(|error| {
            io_error(
                "could not create puzzle directory",
                &self.paths.puzzle_dir,
                error,
            )
        })?;
        let path = self.paths.puzzle_dir.join("puzzles.json");
        let mut file = match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => return Ok(()),
            Err(error) => {
                return Err(io_error(
                    "could not create bundled puzzle collection",
                    &path,
                    error,
                ));
            }
        };
        file.write_all(bundled_examples)
            .map_err(|error| io_error("could not write bundled puzzle collection", &path, error))?;
        file.sync_all()
            .map_err(|error| io_error("could not sync bundled puzzle collection", &path, error))?;
        sync_directory(&self.paths.puzzle_dir)?;
        Ok(())
    }
}

pub fn is_review_collection_filename(name: &str) -> bool {
    if name == "games.json" {
        return true;
    }

    let Some(suffix) = name.strip_prefix("games-") else {
        return false;
    };
    let Some(stem) = suffix.strip_suffix(".json") else {
        return false;
    };

    !stem.is_empty() && !stem.contains('/') && !stem.contains('\\')
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgressLoad {
    pub progress: Progress,
    pub warning: Option<String>,
}

#[derive(Debug)]
pub struct ProgressStore {
    path: PathBuf,
    dirty: bool,
    protected_reason: Option<String>,
}

impl ProgressStore {
    pub fn open(path: impl Into<PathBuf>) -> (Self, ProgressLoad) {
        let path = path.into();
        match fs::read(&path) {
            Ok(bytes) => match Progress::parse(&bytes) {
                Ok(progress) => (
                    Self {
                        path,
                        dirty: false,
                        protected_reason: None,
                    },
                    ProgressLoad {
                        progress,
                        warning: None,
                    },
                ),
                Err(reason) => {
                    let warning = Some(format!(
                        "Progress warning: existing progress is malformed or from a future version and will not be overwritten: {reason}"
                    ));
                    (
                        Self {
                            path,
                            dirty: false,
                            protected_reason: Some(reason),
                        },
                        ProgressLoad {
                            progress: Progress::new(),
                            warning,
                        },
                    )
                }
            },
            Err(error) if error.kind() == io::ErrorKind::NotFound => (
                Self {
                    path,
                    dirty: false,
                    protected_reason: None,
                },
                ProgressLoad {
                    progress: Progress::new(),
                    warning: None,
                },
            ),
            Err(error) => {
                let reason = format!("could not read existing progress: {error}");
                let warning = Some(format!(
                    "Progress warning: existing progress could not be read and will not be overwritten: {error}"
                ));
                (
                    Self {
                        path,
                        dirty: false,
                        protected_reason: Some(reason),
                    },
                    ProgressLoad {
                        progress: Progress::new(),
                        warning,
                    },
                )
            }
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }

    pub fn protected(&self) -> bool {
        self.protected_reason.is_some()
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn save_latest(&mut self, progress: &Progress) -> Result<bool, StorageError> {
        self.mark_dirty();
        self.retry_if_dirty(progress)
    }

    pub fn retry_if_dirty(&mut self, progress: &Progress) -> Result<bool, StorageError> {
        if !self.dirty {
            return Ok(false);
        }

        if let Some(reason) = &self.protected_reason {
            return Err(StorageError::ProtectedProgress {
                path: self.path.clone(),
                reason: reason.clone(),
            });
        }

        let bytes = progress.to_bytes().map_err(StorageError::Progress)?;
        atomic_replace(&self.path, &bytes)?;
        self.dirty = false;
        Ok(true)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettingsLoad {
    pub settings: Settings,
    pub warning: Option<String>,
}

#[derive(Debug)]
pub struct SettingsStore {
    path: PathBuf,
    dirty: bool,
    protected_reason: Option<String>,
}

impl SettingsStore {
    pub fn open(path: impl Into<PathBuf>) -> (Self, SettingsLoad) {
        let path = path.into();
        match fs::read(&path) {
            Ok(bytes) => match Settings::parse(&bytes) {
                Ok(settings) => (
                    Self {
                        path,
                        dirty: false,
                        protected_reason: None,
                    },
                    SettingsLoad {
                        settings,
                        warning: None,
                    },
                ),
                Err(reason) => {
                    let warning = Some(format!(
                        "Settings warning: existing settings are malformed or from a future version and will not be overwritten: {reason}"
                    ));
                    (
                        Self {
                            path,
                            dirty: false,
                            protected_reason: Some(reason),
                        },
                        SettingsLoad {
                            settings: Settings::default(),
                            warning,
                        },
                    )
                }
            },
            Err(error) if error.kind() == io::ErrorKind::NotFound => (
                Self {
                    path,
                    dirty: false,
                    protected_reason: None,
                },
                SettingsLoad {
                    settings: Settings::default(),
                    warning: None,
                },
            ),
            Err(error) => {
                let reason = format!("could not read existing settings: {error}");
                let warning = Some(format!(
                    "Settings warning: existing settings could not be read and will not be overwritten: {error}"
                ));
                (
                    Self {
                        path,
                        dirty: false,
                        protected_reason: Some(reason),
                    },
                    SettingsLoad {
                        settings: Settings::default(),
                        warning,
                    },
                )
            }
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }

    pub fn protected(&self) -> bool {
        self.protected_reason.is_some()
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn save_latest(&mut self, settings: &Settings) -> Result<bool, StorageError> {
        self.mark_dirty();
        self.retry_if_dirty(settings)
    }

    pub fn retry_if_dirty(&mut self, settings: &Settings) -> Result<bool, StorageError> {
        if !self.dirty {
            return Ok(false);
        }

        if let Some(reason) = &self.protected_reason {
            return Err(StorageError::ProtectedSettings {
                path: self.path.clone(),
                reason: reason.clone(),
            });
        }

        let bytes = settings.to_bytes().map_err(StorageError::Settings)?;
        atomic_replace(&self.path, &bytes)?;
        self.dirty = false;
        Ok(true)
    }
}

fn atomic_replace(path: &Path, bytes: &[u8]) -> Result<(), StorageError> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .map_err(|error| io_error("could not create progress directory", parent, error))?;

    let (temp_path, mut temp_file) = create_temp_file(parent, path)?;
    if let Err(error) = temp_file.write_all(bytes) {
        drop(temp_file);
        let _ = fs::remove_file(&temp_path);
        return Err(io_error(
            "could not write temporary progress file",
            &temp_path,
            error,
        ));
    }
    if let Err(error) = temp_file.sync_all() {
        drop(temp_file);
        let _ = fs::remove_file(&temp_path);
        return Err(io_error(
            "could not sync temporary progress file",
            &temp_path,
            error,
        ));
    }
    drop(temp_file);

    if let Err(error) = fs::rename(&temp_path, path) {
        let _ = fs::remove_file(&temp_path);
        return Err(io_error(
            "could not atomically replace progress file",
            path,
            error,
        ));
    }

    sync_directory(parent)?;
    Ok(())
}

fn create_temp_file(parent: &Path, target: &Path) -> Result<(PathBuf, File), StorageError> {
    let target_name = target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("progress.json");

    for _ in 0..128 {
        let sequence = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let temp_path = parent.join(format!(".{target_name}.tmp-{}-{sequence}", process::id()));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
        {
            Ok(file) => return Ok((temp_path, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(io_error(
                    "could not create temporary progress file",
                    &temp_path,
                    error,
                ));
            }
        }
    }

    Err(StorageError::Io(format!(
        "could not allocate a temporary progress file in {}",
        parent.display()
    )))
}

fn sync_directory(path: &Path) -> Result<(), StorageError> {
    let directory = File::open(path)
        .map_err(|error| io_error("could not open directory for sync", path, error))?;
    directory
        .sync_all()
        .map_err(|error| io_error("could not sync directory", path, error))
}

fn io_error(context: &str, path: &Path, error: io::Error) -> StorageError {
    StorageError::Io(format!("{context} {}: {error}", path.display()))
}
