//! The save file: hiscore and whether the player has finished the tutorial.
//!
//! cart: `cartdata("bee_randon_bubblegum_spin_1_1")`, `dget(0)`/`dset(0,...)` for
//! the hiscore and `dget(1)`/`dset(1,1)` for `completed`. PICO-8 keeps that in a
//! platform-specific cartdata slot; the port writes a small text file next to the
//! executable instead.
//!
//! One deliberate deviation: the cart reads the hiscore and then immediately does
//! `dset(0,0)` at boot, so a hiscore only survives if you beat it again. This port
//! keeps it (see `docs/decisions.md`).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// cart slots 0 and 1.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Save {
    pub hiscore: u32,
    pub completed: bool,
}

/// The file name used next to the executable.
pub const FILE_NAME: &str = "save.dat";

impl Save {
    /// Read a save file. A missing or unreadable file is an empty save, never an
    /// error: a corrupt hiscore must not stop the game from starting.
    pub fn load(path: &Path) -> Self {
        let Ok(text) = fs::read_to_string(path) else {
            return Self::default();
        };
        let mut save = Self::default();
        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            match key.trim() {
                "hiscore" => save.hiscore = value.trim().parse().unwrap_or(0),
                "completed" => save.completed = value.trim() == "1" || value.trim() == "true",
                _ => {}
            }
        }
        save
    }

    /// Write the save file, creating it if needed.
    pub fn store(&self, path: &Path) -> io::Result<()> {
        if let Some(dir) = path.parent() {
            if !dir.as_os_str().is_empty() {
                fs::create_dir_all(dir)?;
            }
        }
        let text = format!(
            "hiscore={}\ncompleted={}\n",
            self.hiscore,
            u8::from(self.completed)
        );
        fs::write(path, text)
    }

    /// The default location: `save.dat` beside the executable.
    pub fn default_path() -> Option<PathBuf> {
        std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|dir| dir.join(FILE_NAME)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("bgs-save-{}-{name}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        dir.join(FILE_NAME)
    }

    #[test]
    fn a_save_round_trips() {
        let path = temp("roundtrip");
        let save = Save {
            hiscore: 12_345,
            completed: true,
        };
        save.store(&path).unwrap();
        assert_eq!(Save::load(&path), save);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn a_missing_file_is_an_empty_save() {
        let path = temp("missing");
        let _ = fs::remove_file(&path);
        assert_eq!(Save::load(&path), Save::default());
    }

    #[test]
    fn a_corrupt_file_does_not_stop_the_game() {
        let path = temp("corrupt");
        fs::write(&path, "hiscore=not-a-number\nnonsense\n").unwrap();
        assert_eq!(Save::load(&path), Save::default());
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn the_default_path_is_save_dat_beside_the_executable() {
        let path = Save::default_path().expect("the test binary has a path");
        assert_eq!(path.file_name().unwrap(), FILE_NAME);
        assert_eq!(
            path.parent().unwrap(),
            std::env::current_exe().unwrap().parent().unwrap()
        );
    }
}
