//! X 서버의 키보드 매핑으로 키코드를 기본 키심으로 바꾼다.
//!
//! 엔진은 Shift를 따로 받으므로 키코드마다 수식 키가 없는 첫 키심만 쓴다.

#[derive(Debug, Default)]
pub struct Keymap {
    min_keycode: u8,
    base: Vec<u32>,
}

impl Keymap {
    /// `GetKeyboardMapping` 응답(키코드마다 `per_keycode`개씩 이어진 키심)으로 만든다.
    pub fn from_mapping(min_keycode: u8, per_keycode: u8, keysyms: &[u32]) -> Self {
        let base = match per_keycode {
            0 => Vec::new(),
            n => keysyms.chunks(n as usize).map(|row| row[0]).collect(),
        };
        Keymap { min_keycode, base }
    }

    /// 매핑에 없는 키코드는 0(NoSymbol)이다.
    pub fn base_keysym(&self, keycode: u8) -> u32 {
        keycode
            .checked_sub(self.min_keycode)
            .and_then(|i| self.base.get(i as usize))
            .copied()
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_first_keysym_of_each_keycode() {
        // 키코드 8: a/A, 키코드 9: BackSpace
        let map = Keymap::from_mapping(8, 2, &['a' as u32, 'A' as u32, 0xff08, 0]);
        assert_eq!(map.base_keysym(8), 'a' as u32);
        assert_eq!(map.base_keysym(9), 0xff08);
    }

    #[test]
    fn unknown_keycodes_are_no_symbol() {
        let map = Keymap::from_mapping(8, 2, &['a' as u32, 'A' as u32]);
        assert_eq!(map.base_keysym(7), 0);
        assert_eq!(map.base_keysym(9), 0);
        assert_eq!(Keymap::from_mapping(8, 0, &[]).base_keysym(8), 0);
    }
}
