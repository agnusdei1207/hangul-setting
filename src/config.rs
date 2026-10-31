//! 사용자 설정 `~/.config/hangul-setting/config`. 지금은 한/영 전환 키 하나만 정한다.
//!
//! ```text
//! # 쉼표로 여러 개. 이름: Hangul, Alt_R, Control_R, Shift+space, Ctrl+space, F9 …
//! toggle = Hangul, Alt_R, Shift+space
//! ```

use std::path::PathBuf;

use crate::engine::{DEFAULT_TOGGLE_KEYS, Hotkey, parse_toggle_keys};

#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    pub toggle_keys: Vec<Hotkey>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            toggle_keys: parse_toggle_keys(DEFAULT_TOGGLE_KEYS).expect("기본 한/영 키"),
        }
    }
}

/// 설정 파일 본문을 읽는다. 오류는 줄 번호와 함께 돌려준다.
pub fn parse(text: &str) -> Result<Config, String> {
    let mut config = Config::default();
    for (number, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let at = |e: String| format!("{}번째 줄: {e}", number + 1);
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| at(format!("'이름 = 값' 형식이 아님: '{line}'")))?;
        match key.trim() {
            "toggle" => config.toggle_keys = parse_toggle_keys(value).map_err(at)?,
            other => return Err(at(format!("모르는 설정 '{other}' (쓸 수 있는 것: toggle)"))),
        }
    }
    Ok(config)
}

/// `$XDG_CONFIG_HOME/hangul-setting/config`, 없으면 `~/.config/hangul-setting/config`.
pub fn path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    Some(base.join("hangul-setting").join("config"))
}

/// 사용자 설정을 읽는다. 파일이 없으면 기본값, 잘못됐으면 오류(호출자가 알리고 기본값을 쓴다).
pub fn load() -> Result<Config, String> {
    path().map_or(Ok(Config::default()), |path| load_from(&path))
}

pub fn load_from(path: &std::path::Path) -> Result<Config, String> {
    match std::fs::read_to_string(path) {
        Ok(text) => parse(&text).map_err(|e| format!("{}: {e}", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Config::default()),
        Err(e) => Err(format!("{}: {e}", path.display())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_or_comment_only_file_is_default() {
        assert_eq!(parse("").unwrap(), Config::default());
        assert_eq!(parse("# 주석\n\n  # 들여쓴 주석\n").unwrap(), Config::default());
    }

    #[test]
    fn toggle_line_sets_toggle_keys() {
        let config = parse("toggle = Ctrl+space, F9\n").unwrap();
        assert_eq!(config.toggle_keys, parse_toggle_keys("Ctrl+space, F9").unwrap());
    }

    #[test]
    fn errors_name_the_line() {
        assert!(parse("# ok\ntoggle Hangul").unwrap_err().starts_with("2번째 줄"));
        assert!(parse("toggle = NoSuchKey").unwrap_err().contains("NoSuchKey"));
        assert!(parse("layout = 3").unwrap_err().contains("layout"));
    }

    #[test]
    fn load_from_reads_file_and_defaults_when_missing() {
        let dir = std::env::temp_dir().join(format!("hangul-setting-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("config");
        std::fs::write(
            &file,
            "toggle = Control_R
",
        )
        .unwrap();
        assert_eq!(
            load_from(&file).unwrap().toggle_keys,
            parse_toggle_keys("Control_R").unwrap()
        );
        std::fs::remove_file(&file).unwrap();
        assert_eq!(load_from(&file).unwrap(), Config::default());
        std::fs::remove_dir(&dir).unwrap();
    }
}
