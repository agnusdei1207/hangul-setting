//! 키 입력 하나를 받아 확정할 글자, 조합 중 글자, 키 소비 여부를 정한다.

use crate::hangul::{Composer, Jamo};

/// X11 키심 중 엔진이 구분하는 것들.
pub mod keysym {
    pub const SPACE: u32 = 0x0020;
    pub const BACKSPACE: u32 = 0xff08;
    pub const HANGUL: u32 = 0xff31;
    pub const ALT_R: u32 = 0xffea;
    /// Shift_L부터 Hyper_R까지와 ISO_Level3_Shift 등 수식 키만 누른 경우.
    pub fn is_modifier(sym: u32) -> bool {
        (0xffe1..=0xffee).contains(&sym) || (0xfe01..=0xfe13).contains(&sym)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Key {
    pub keysym: u32,
    pub shift: bool,
    /// Ctrl·Alt·Super 중 하나라도 눌림. 이때 키는 단축키로 본다.
    pub command: bool,
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

/// 입력 모드는 앱 창 전체에 하나다. 조합 상태는 입력 컨텍스트마다 따로 받는다.
#[derive(Debug, Default)]
pub struct Engine {
    pub mode: Mode,
}

impl Engine {
    pub fn handle_key(&mut self, composer: &mut Composer, key: Key) -> KeyOutcome {
        if is_toggle(key) {
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
        if self.mode == Mode::Hangul && !key.command {
            if let Some(jamo) = dubeolsik(key.keysym, key.shift) {
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

fn is_toggle(key: Key) -> bool {
    match key.keysym {
        keysym::HANGUL | keysym::ALT_R => true,
        keysym::SPACE => key.shift && !key.command,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn letter(c: char) -> Key {
        Key {
            keysym: c as u32,
            shift: c.is_ascii_uppercase(),
            command: false,
        }
    }

    fn sym(keysym: u32) -> Key {
        Key {
            keysym,
            ..Key::default()
        }
    }

    fn hangul_engine() -> Engine {
        Engine { mode: Mode::Hangul }
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
        let mut composer = Composer::default();
        let keys = [letter('g'), letter('k'), letter('s'), sym(keysym::SPACE), letter('1')];
        assert_eq!(type_keys(&mut hangul_engine(), &mut composer, &keys), "한<20><31>");
    }

    #[test]
    fn command_shortcut_commits_then_passes_through() {
        let mut composer = Composer::default();
        let ctrl_c = Key {
            keysym: 'c' as u32,
            shift: false,
            command: true,
        };
        let keys = [letter('r'), letter('k'), ctrl_c];
        assert_eq!(type_keys(&mut hangul_engine(), &mut composer, &keys), "가<63>");
    }

    #[test]
    fn modifier_alone_keeps_composition() {
        let mut composer = Composer::default();
        let keys = [letter('r'), sym(0xffe1), letter('k')];
        assert_eq!(type_keys(&mut hangul_engine(), &mut composer, &keys), "<ffe1>가");
    }

    #[test]
    fn backspace_edits_composition_then_passes_through() {
        let mut composer = Composer::default();
        let keys = [
            letter('r'),
            letter('k'),
            sym(keysym::BACKSPACE),
            sym(keysym::BACKSPACE),
            sym(keysym::BACKSPACE),
        ];
        assert_eq!(type_keys(&mut hangul_engine(), &mut composer, &keys), "<ff08>");
    }

    #[test]
    fn toggle_keys_switch_mode_and_commit() {
        for toggle in [
            sym(keysym::HANGUL),
            sym(keysym::ALT_R),
            Key {
                keysym: keysym::SPACE,
                shift: true,
                command: false,
            },
        ] {
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
    fn latin_mode_passes_every_key() {
        let mut engine = Engine::default();
        let keys = [letter('r'), letter('k'), sym(keysym::BACKSPACE)];
        assert_eq!(
            type_keys(&mut engine, &mut Composer::default(), &keys),
            "<72><6b><ff08>"
        );
    }

    #[test]
    fn shift_space_with_command_is_not_toggle() {
        let mut engine = hangul_engine();
        let key = Key {
            keysym: keysym::SPACE,
            shift: true,
            command: true,
        };
        assert!(!engine.handle_key(&mut Composer::default(), key).consumed);
        assert_eq!(engine.mode, Mode::Hangul);
    }
}
