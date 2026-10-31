//! 키 입력 하나를 받아 확정할 글자, 조합 중 글자, 키 소비 여부를 정한다.

use crate::hangul::{Composer, Jamo};

/// X11 키심 중 엔진이 구분하는 것들.
pub mod keysym {
    pub const SPACE: u32 = 0x0020;
    pub const BACKSPACE: u32 = 0xff08;
    /// Shift_L부터 Hyper_R까지와 ISO_Level3_Shift 등 수식 키만 누른 경우.
    pub fn is_modifier(sym: u32) -> bool {
        (0xffe1..=0xffee).contains(&sym) || (0xfe01..=0xfe13).contains(&sym)
    }
}

/// 함께 눌린 수식 키. CapsLock·NumLock은 넣지 않는다.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub super_: bool,
}

impl Modifiers {
    /// X11 수식 키 상태(KeyPress의 `state`, IBus의 `state`). IBus는 Super(26)·Meta(28) 비트를 더 쓴다.
    pub fn from_x_state(state: u32) -> Self {
        Modifiers {
            shift: state & (1 << 0) != 0,
            ctrl: state & (1 << 2) != 0,
            alt: state & (1 << 3) != 0,
            super_: state & ((1 << 6) | (1 << 26) | (1 << 28)) != 0,
        }
    }

    /// Ctrl·Alt·Super 중 하나라도 눌렸으면 단축키로 본다.
    pub fn is_command(self) -> bool {
        self.ctrl || self.alt || self.super_
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Key {
    pub keysym: u32,
    pub modifiers: Modifiers,
}

impl Key {
    pub fn from_x_state(keysym: u32, state: u32) -> Self {
        Key {
            keysym,
            modifiers: Modifiers::from_x_state(state),
        }
    }
}

/// 한/영 전환 키 하나. 키심과 수식 키가 정확히 같을 때 맞는다.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hotkey {
    pub keysym: u32,
    pub modifiers: Modifiers,
}

/// 설정에 쓸 수 있는 키 이름. 이 밖에는 글자 하나(`a`, `1`)와 `F1`~`F12`를 쓴다.
const KEY_NAMES: [(&str, u32); 17] = [
    ("Hangul", 0xff31),
    ("Hangul_Hanja", 0xff34),
    ("space", 0x0020),
    ("Tab", 0xff09),
    ("Menu", 0xff67),
    ("Caps_Lock", 0xffe5),
    ("Shift_L", 0xffe1),
    ("Shift_R", 0xffe2),
    ("Control_L", 0xffe3),
    ("Control_R", 0xffe4),
    ("Alt_L", 0xffe9),
    ("Alt_R", 0xffea),
    ("Super_L", 0xffeb),
    ("Super_R", 0xffec),
    ("ISO_Level3_Shift", 0xfe03),
    ("Escape", 0xff1b),
    ("Pause", 0xff13),
];

fn keysym_by_name(name: &str) -> Option<u32> {
    if let Some(&(_, sym)) = KEY_NAMES.iter().find(|(n, _)| n.eq_ignore_ascii_case(name)) {
        return Some(sym);
    }
    if let Some(n) = name.strip_prefix(['F', 'f']).and_then(|n| n.parse::<u32>().ok())
        && (1..=12).contains(&n)
    {
        return Some(0xffbe + n - 1);
    }
    let mut chars = name.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) if c.is_ascii_graphic() => Some(c.to_ascii_lowercase() as u32),
        _ => None,
    }
}

impl Hotkey {
    /// `Hangul`, `Alt_R`, `Shift+space`, `Ctrl+space` 같은 이름. 대소문자는 가리지 않는다.
    pub fn parse(text: &str) -> Result<Hotkey, String> {
        let mut parts: Vec<&str> = text.split('+').map(str::trim).collect();
        let name = parts
            .pop()
            .filter(|n| !n.is_empty())
            .ok_or_else(|| format!("키 이름이 없음: '{text}'"))?;
        let mut modifiers = Modifiers::default();
        for part in parts {
            let flag = match part.to_ascii_lowercase().as_str() {
                "shift" => &mut modifiers.shift,
                "ctrl" | "control" => &mut modifiers.ctrl,
                "alt" => &mut modifiers.alt,
                "super" | "win" => &mut modifiers.super_,
                _ => return Err(format!("모르는 수식 키 '{part}' (Shift, Ctrl, Alt, Super): '{text}'")),
            };
            *flag = true;
        }
        let keysym = keysym_by_name(name).ok_or_else(|| format!("모르는 키 이름 '{name}': '{text}'"))?;
        Ok(Hotkey { keysym, modifiers })
    }

    fn matches(&self, key: Key) -> bool {
        self.keysym == key.keysym && self.modifiers == key.modifiers
    }
}

/// 설정이 없을 때의 한/영 전환 키.
pub const DEFAULT_TOGGLE_KEYS: &str = "Hangul, Alt_R, Shift+space";

/// 쉼표로 나눈 한/영 전환 키 목록.
pub fn parse_toggle_keys(text: &str) -> Result<Vec<Hotkey>, String> {
    let keys = text.split(',').map(Hotkey::parse).collect::<Result<Vec<_>, _>>()?;
    Ok(keys)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    Hangul,
    #[default]
    Latin,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct KeyOutcome {
    /// 앱에 확정할 글자. 키를 넘기는 경우 키보다 먼저 확정한다.
    pub commit: String,
    /// `false`면 원래 키를 앱에 그대로 넘긴다.
    pub consumed: bool,
}

/// 두벌식 표준 자판. Shift는 ㅂㅈㄷㄱㅅㅐㅔ 자리만 된소리·ㅒㅖ로 바꾼다.
pub fn dubeolsik(keysym: u32, shift: bool) -> Option<Jamo> {
    let c = char::from_u32(keysym)?.to_ascii_lowercase();
    let jamo = match (c, shift) {
        ('q', true) => 'ㅃ',
        ('w', true) => 'ㅉ',
        ('e', true) => 'ㄸ',
        ('r', true) => 'ㄲ',
        ('t', true) => 'ㅆ',
        ('o', true) => 'ㅒ',
        ('p', true) => 'ㅖ',
        _ => {
            let index = "qwertyuiopasdfghjklzxcvbnm".find(c)?;
            "ㅂㅈㄷㄱㅅㅛㅕㅑㅐㅔㅁㄴㅇㄹㅎㅗㅓㅏㅣㅋㅌㅊㅍㅠㅜㅡ"
                .chars()
                .nth(index)?
        }
    };
    Some(Jamo::from_compat(jamo))
}

/// 입력 모드는 하나를 여러 앱이 함께 쓴다. 조합 상태는 입력 컨텍스트마다 따로 받는다.
#[derive(Debug)]
pub struct Engine {
    pub mode: Mode,
    toggle_keys: Vec<Hotkey>,
}

impl Default for Engine {
    fn default() -> Self {
        Engine::new(parse_toggle_keys(DEFAULT_TOGGLE_KEYS).expect("기본 한/영 키"))
    }
}

impl Engine {
    pub fn new(toggle_keys: Vec<Hotkey>) -> Self {
        Engine {
            mode: Mode::Latin,
            toggle_keys,
        }
    }

    pub fn handle_key(&mut self, composer: &mut Composer, key: Key) -> KeyOutcome {
        if self.toggle_keys.iter().any(|t| t.matches(key)) {
            self.mode = match self.mode {
                Mode::Hangul => Mode::Latin,
                Mode::Latin => Mode::Hangul,
            };
            return KeyOutcome {
                commit: composer.flush(),
                consumed: true,
            };
        }
        if keysym::is_modifier(key.keysym) {
            return KeyOutcome::default();
        }
        if self.mode == Mode::Hangul && !key.modifiers.is_command() {
            if let Some(jamo) = dubeolsik(key.keysym, key.modifiers.shift) {
                return KeyOutcome {
                    commit: composer.push(jamo),
                    consumed: true,
                };
            }
            if key.keysym == keysym::BACKSPACE && composer.backspace() {
                return KeyOutcome {
                    commit: String::new(),
                    consumed: true,
                };
            }
        }
        KeyOutcome {
            commit: composer.flush(),
            consumed: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHIFT: Modifiers = Modifiers {
        shift: true,
        ctrl: false,
        alt: false,
        super_: false,
    };
    const CTRL: Modifiers = Modifiers {
        shift: false,
        ctrl: true,
        alt: false,
        super_: false,
    };

    fn letter(c: char) -> Key {
        let modifiers = if c.is_ascii_uppercase() {
            SHIFT
        } else {
            Modifiers::default()
        };
        Key {
            keysym: c as u32,
            modifiers,
        }
    }

    fn sym(keysym: u32) -> Key {
        Key {
            keysym,
            ..Key::default()
        }
    }

    fn hangul_engine() -> Engine {
        Engine {
            mode: Mode::Hangul,
            ..Engine::default()
        }
    }

    /// 키를 차례로 치고 (앱이 받은 확정 글자 + 넘겨받은 키 표시 + 조합 중 글자)를 돌려준다.
    fn type_keys(engine: &mut Engine, composer: &mut Composer, keys: &[Key]) -> String {
        let mut screen = String::new();
        for &key in keys {
            let outcome = engine.handle_key(composer, key);
            screen += &outcome.commit;
            if !outcome.consumed {
                screen += &format!("<{:x}>", key.keysym);
            }
        }
        screen + &composer.preedit()
    }

    fn type_text(text: &str) -> String {
        let keys: Vec<Key> = text.chars().map(letter).collect();
        type_keys(&mut hangul_engine(), &mut Composer::default(), &keys)
    }

    #[test]
    fn dubeolsik_composes_words() {
        assert_eq!(type_text("gksdlf"), "한일");
        assert_eq!(type_text("dkssud"), "안녕");
        assert_eq!(type_text("rkqt"), "값");
        assert_eq!(type_text("ekfrk"), "달가");
        assert_eq!(type_text("hk"), "ㅘ");
        assert_eq!(type_text("dhk"), "와");
    }

    #[test]
    fn shift_gives_tense_consonants_and_only_those() {
        assert_eq!(type_text("Rk"), "까");
        assert_eq!(type_text("dO"), "얘");
        assert_eq!(type_text("Gk"), "하");
    }

    #[test]
    fn non_hangul_key_commits_then_passes_through() {
        let keys = [letter('g'), letter('k'), letter('s'), sym(keysym::SPACE), letter('1')];
        assert_eq!(
            type_keys(&mut hangul_engine(), &mut Composer::default(), &keys),
            "한<20><31>"
        );
    }

    #[test]
    fn command_shortcut_commits_then_passes_through() {
        let ctrl_c = Key {
            keysym: 'c' as u32,
            modifiers: CTRL,
        };
        let keys = [letter('r'), letter('k'), ctrl_c];
        assert_eq!(
            type_keys(&mut hangul_engine(), &mut Composer::default(), &keys),
            "가<63>"
        );
    }

    #[test]
    fn modifier_alone_keeps_composition() {
        let keys = [letter('r'), sym(0xffe1), letter('k')];
        assert_eq!(
            type_keys(&mut hangul_engine(), &mut Composer::default(), &keys),
            "<ffe1>가"
        );
    }

    #[test]
    fn backspace_edits_composition_then_passes_through() {
        let bs = sym(keysym::BACKSPACE);
        let keys = [letter('r'), letter('k'), bs, bs, bs];
        assert_eq!(
            type_keys(&mut hangul_engine(), &mut Composer::default(), &keys),
            "<ff08>"
        );
    }

    #[test]
    fn default_toggle_keys_switch_mode_and_commit() {
        let shift_space = Key {
            keysym: keysym::SPACE,
            modifiers: SHIFT,
        };
        for toggle in [sym(0xff31), sym(0xffea), shift_space] {
            let mut engine = hangul_engine();
            let mut composer = Composer::default();
            let keys = [letter('r'), letter('k'), toggle, letter('r')];
            assert_eq!(type_keys(&mut engine, &mut composer, &keys), "가<72>");
            assert_eq!(engine.mode, Mode::Latin);
            type_keys(&mut engine, &mut composer, &[toggle]);
            assert_eq!(engine.mode, Mode::Hangul);
        }
    }

    #[test]
    fn configured_toggle_keys_replace_defaults() {
        let mut engine = Engine::new(parse_toggle_keys("Ctrl+space").unwrap());
        let mut composer = Composer::default();
        engine.handle_key(&mut composer, sym(0xff31));
        assert_eq!(engine.mode, Mode::Latin, "기본 키 Hangul은 더 이상 전환하지 않는다");
        let outcome = engine.handle_key(
            &mut composer,
            Key {
                keysym: keysym::SPACE,
                modifiers: CTRL,
            },
        );
        assert!(outcome.consumed);
        assert_eq!(engine.mode, Mode::Hangul);
    }

    #[test]
    fn toggle_needs_exact_modifiers() {
        let mut engine = hangul_engine();
        let ctrl_shift_space = Key {
            keysym: keysym::SPACE,
            modifiers: Modifiers {
                shift: true,
                ctrl: true,
                ..Modifiers::default()
            },
        };
        assert!(!engine.handle_key(&mut Composer::default(), ctrl_shift_space).consumed);
        assert_eq!(engine.mode, Mode::Hangul);
    }

    #[test]
    fn latin_mode_passes_every_key() {
        let keys = [letter('r'), letter('k'), sym(keysym::BACKSPACE)];
        assert_eq!(
            type_keys(&mut Engine::default(), &mut Composer::default(), &keys),
            "<72><6b><ff08>"
        );
    }

    #[test]
    fn x_state_decodes_modifiers() {
        assert_eq!(Modifiers::from_x_state(0), Modifiers::default());
        assert_eq!(Modifiers::from_x_state(1), SHIFT);
        assert_eq!(Modifiers::from_x_state(1 << 2), CTRL);
        // CapsLock(2)·NumLock(16)은 수식 키로 치지 않는다
        assert_eq!(Modifiers::from_x_state(2 | 16), Modifiers::default());
        for state in [1 << 3, 1 << 6, 1 << 26, 1 << 28] {
            assert!(Modifiers::from_x_state(state).is_command(), "state {state:#x}");
        }
    }

    #[test]
    fn hotkey_names_parse() {
        let parse = |s| Hotkey::parse(s).unwrap();
        assert_eq!(
            parse("Hangul"),
            Hotkey {
                keysym: 0xff31,
                modifiers: Modifiers::default()
            }
        );
        assert_eq!(
            parse(" shift + Space "),
            Hotkey {
                keysym: 0x20,
                modifiers: SHIFT
            }
        );
        assert_eq!(parse("Control+space").modifiers, CTRL);
        assert_eq!(parse("F12").keysym, 0xffc9);
        assert_eq!(parse("Super+A").keysym, 'a' as u32);
        assert_eq!(parse_toggle_keys(DEFAULT_TOGGLE_KEYS).unwrap().len(), 3);
    }

    #[test]
    fn bad_hotkey_names_are_errors() {
        for bad in ["", "Shift+", "Hyper+space", "NoSuchKey", "F13", "Ctrl+ab"] {
            assert!(Hotkey::parse(bad).is_err(), "{bad:?}");
        }
        assert!(parse_toggle_keys("Hangul,").is_err());
    }
}
