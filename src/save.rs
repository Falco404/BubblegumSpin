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

/// The save file's name, wherever it ends up.
pub const FILE_NAME: &str = "save.dat";

/// An environment variable as a path, treating an empty value as unset.
fn non_empty_var(name: &str) -> Option<PathBuf> {
    std::env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

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

    /// The default location of `save.dat`, and the order the targets need:
    ///
    /// 1. `BGS_SAVE` — a full path; the escape hatch for tests and for a phone that
    ///    hides its storage.
    /// 2. Android — `$HOME/save.dat`. `current_exe()` there is a read-only file inside
    ///    the APK (`/data/app/.../base.apk`), so the executable's folder is never used.
    /// 3. Desktop — beside the executable, as it has been since M5, falling back to
    ///    `$HOME` when the executable's folder cannot be worked out.
    ///
    /// `None` means "run without a hiscore", never an error: a phone that hides `HOME`
    /// should still play.
    pub fn default_path() -> Option<PathBuf> {
        let save = non_empty_var("BGS_SAVE");
        let home = non_empty_var("HOME");
        let exe = std::env::current_exe().ok();
        Self::default_path_from(
            save.as_deref(),
            home.as_deref(),
            exe.as_deref(),
            cfg!(target_os = "android"),
        )
    }

    /// [`Save::default_path`] with every input injected, so each target's rule can be
    /// tested on any host.
    pub fn default_path_from(
        save: Option<&Path>,
        home: Option<&Path>,
        exe: Option<&Path>,
        android: bool,
    ) -> Option<PathBuf> {
        if let Some(save) = save {
            return Some(save.to_path_buf());
        }
        if android {
            return home.map(|dir| dir.join(FILE_NAME));
        }
        exe.and_then(Path::parent)
            .map(|dir| dir.join(FILE_NAME))
            .or_else(|| home.map(|dir| dir.join(FILE_NAME)))
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
    fn a_full_path_in_bgs_save_wins_everywhere() {
        // The escape hatch: whatever the platform thinks, `BGS_SAVE` is obeyed.
        let path = Save::default_path_from(
            Some(Path::new(
                "/data/user/0/com.falco.bubblegumspin/files/mine.dat",
            )),
            Some(Path::new("/home/falco")),
            Some(Path::new("/opt/game/bubblegum-spin")),
            true,
        );
        assert_eq!(
            path.unwrap(),
            PathBuf::from("/data/user/0/com.falco.bubblegumspin/files/mine.dat")
        );
    }

    #[test]
    fn android_saves_under_home_and_never_beside_the_apk() {
        // `current_exe()` inside an APK is a read-only path in /data/app, so the
        // executable's folder must not be used there even when it is known.
        let path = Save::default_path_from(
            None,
            Some(Path::new("/data/user/0/com.falco.bubblegumspin/files")),
            Some(Path::new("/data/app/~~abc==/base.apk")),
            true,
        );
        assert_eq!(
            path.unwrap(),
            PathBuf::from("/data/user/0/com.falco.bubblegumspin/files/save.dat")
        );
    }

    #[test]
    fn android_without_a_home_has_no_save_file() {
        // Better no hiscore than a panic at startup on a device that hides `HOME`.
        let path = Save::default_path_from(None, None, None, true);
        assert!(path.is_none());
    }

    #[test]
    fn a_desktop_saves_beside_the_executable() {
        let path = Save::default_path_from(
            None,
            Some(Path::new("/home/falco")),
            Some(Path::new("/opt/game/bubblegum-spin")),
            false,
        );
        assert_eq!(path.unwrap(), PathBuf::from("/opt/game/save.dat"));
    }

    #[test]
    fn a_desktop_without_an_executable_path_falls_back_to_home() {
        let path = Save::default_path_from(None, Some(Path::new("/home/falco")), None, false);
        assert_eq!(path.unwrap(), PathBuf::from("/home/falco/save.dat"));
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
