//! 자모를 받아 한글 음절 하나를 조합한다.

/// 한글 낱자. 호환 자모 문자(U+3131~U+3163)로 표현한다.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Jamo {
    Consonant(char),
    Vowel(char),
}

impl Jamo {
    /// 호환 자모 문자 하나를 자음 또는 모음으로 가른다.
    pub fn from_compat(c: char) -> Jamo {
        if (FIRST_VOWEL..=LAST_VOWEL).contains(&c) {
            Jamo::Vowel(c)
        } else {
            Jamo::Consonant(c)
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Syllable {
    cho: Option<char>,
    jung: Option<char>,
    jong: Option<char>,
}

/// 음절 하나를 조합하는 상태 기계.
#[derive(Debug, Default)]
pub struct Composer {
    current: Syllable,
    /// 자모를 넣기 전 상태들. 백스페이스가 자모 한 단위씩 되돌린다.
    history: Vec<Syllable>,
}

const CHO: &str = "ㄱㄲㄴㄷㄸㄹㅁㅂㅃㅅㅆㅇㅈㅉㅊㅋㅌㅍㅎ";
/// 종성 순서. 유니코드 음절 공식의 종성 번호는 이 순서 + 1이다(0은 종성 없음).
const JONG: &str = "ㄱㄲㄳㄴㄵㄶㄷㄹㄺㄻㄼㄽㄾㄿㅀㅁㅂㅄㅅㅆㅇㅈㅊㅋㅌㅍㅎ";
const FIRST_VOWEL: char = 'ㅏ';
const LAST_VOWEL: char = 'ㅣ';
const COMPOUND_JONG: [(char, char, char); 11] = [
    ('ㄱ', 'ㅅ', 'ㄳ'),
    ('ㄴ', 'ㅈ', 'ㄵ'),
    ('ㄴ', 'ㅎ', 'ㄶ'),
    ('ㄹ', 'ㄱ', 'ㄺ'),
    ('ㄹ', 'ㅁ', 'ㄻ'),
    ('ㄹ', 'ㅂ', 'ㄼ'),
    ('ㄹ', 'ㅅ', 'ㄽ'),
    ('ㄹ', 'ㅌ', 'ㄾ'),
    ('ㄹ', 'ㅍ', 'ㄿ'),
    ('ㄹ', 'ㅎ', 'ㅀ'),
    ('ㅂ', 'ㅅ', 'ㅄ'),
];
const COMPOUND_JUNG: [(char, char, char); 7] = [
    ('ㅗ', 'ㅏ', 'ㅘ'),
    ('ㅗ', 'ㅐ', 'ㅙ'),
    ('ㅗ', 'ㅣ', 'ㅚ'),
    ('ㅜ', 'ㅓ', 'ㅝ'),
    ('ㅜ', 'ㅔ', 'ㅞ'),
    ('ㅜ', 'ㅣ', 'ㅟ'),
    ('ㅡ', 'ㅣ', 'ㅢ'),
];

fn position(table: &str, c: char) -> Option<u32> {
    table.chars().position(|t| t == c).map(|i| i as u32)
}

fn combine(table: &[(char, char, char)], a: char, b: char) -> Option<char> {
    table.iter().find(|&&(x, y, _)| x == a && y == b).map(|&(_, _, z)| z)
}

fn split_jong(c: char) -> Option<(char, char)> {
    COMPOUND_JONG.iter().find(|&&(_, _, z)| z == c).map(|&(x, y, _)| (x, y))
}

impl Syllable {
    fn cho(c: char) -> Self {
        Syllable {
            cho: Some(c),
            ..Self::default()
        }
    }

    fn jung(v: char) -> Self {
        Syllable {
            jung: Some(v),
            ..Self::default()
        }
    }

    fn render(&self) -> String {
        match (self.cho, self.jung) {
            (Some(cho), Some(jung)) => {
                let cho = position(CHO, cho).expect("초성 표에 있는 자음");
                let jung = jung as u32 - FIRST_VOWEL as u32;
                let jong = self
                    .jong
                    .map_or(0, |c| position(JONG, c).expect("종성 표에 있는 자음") + 1);
                let code = 0xAC00 + (cho * 21 + jung) * 28 + jong;
                char::from_u32(code).expect("한글 음절 범위").to_string()
            }
            (Some(c), None) | (None, Some(c)) => c.to_string(),
            (None, None) => String::new(),
        }
    }
}

impl Composer {
    /// 자모 하나를 넣고, 그 결과 확정된 글자를 돌려준다(없으면 빈 문자열).
    pub fn push(&mut self, jamo: Jamo) -> String {
        let s = self.current;
        let next = match jamo {
            Jamo::Consonant(c) => match s {
                Syllable {
                    cho: None, jung: None, ..
                } => Some(Syllable::cho(c)),
                Syllable {
                    cho: Some(_),
                    jung: Some(_),
                    jong: None,
                } if position(JONG, c).is_some() => Some(Syllable { jong: Some(c), ..s }),
                Syllable { jong: Some(j), .. } => {
                    combine(&COMPOUND_JONG, j, c).map(|jong| Syllable { jong: Some(jong), ..s })
                }
                _ => None,
            },
            Jamo::Vowel(v) => match s {
                Syllable { jung: None, .. } => Some(Syllable { jung: Some(v), ..s }),
                Syllable {
                    jung: Some(j),
                    jong: None,
                    ..
                } => combine(&COMPOUND_JUNG, j, v).map(|jung| Syllable { jung: Some(jung), ..s }),
                Syllable { jong: Some(j), .. } => {
                    // 종성이 다음 음절의 초성으로 넘어간다. 겹받침은 뒤 자음만 넘어간다.
                    let (kept, moved) = match split_jong(j) {
                        Some((first, second)) => (Some(first), second),
                        None => (None, j),
                    };
                    let committed = Syllable { jong: kept, ..s }.render();
                    self.history = vec![Syllable::default(), Syllable::cho(moved)];
                    self.current = Syllable {
                        cho: Some(moved),
                        jung: Some(v),
                        jong: None,
                    };
                    return committed;
                }
            },
        };
        match next {
            Some(next) => {
                self.history.push(s);
                self.current = next;
                String::new()
            }
            None => {
                let committed = self.flush();
                self.history.push(Syllable::default());
                self.current = match jamo {
                    Jamo::Consonant(c) => Syllable::cho(c),
                    Jamo::Vowel(v) => Syllable::jung(v),
                };
                committed
            }
        }
    }

    /// 마지막 자모 하나를 지운다. 지울 것이 없었으면 `false`.
    pub fn backspace(&mut self) -> bool {
        match self.history.pop() {
            Some(previous) => {
                self.current = previous;
                true
            }
            None => false,
        }
    }

    /// 조합 중 글자.
    pub fn preedit(&self) -> String {
        self.current.render()
    }

    /// 조합 중 글자를 확정하고 비운다.
    pub fn flush(&mut self) -> String {
        let committed = self.current.render();
        *self = Self::default();
        committed
    }

    pub fn is_empty(&self) -> bool {
        self.current == Syllable::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 호환 자모 문자열을 차례로 넣고 (확정 누적 + 조합 중 글자)를 돌려준다.
    fn type_jamo(composer: &mut Composer, jamo: &str) -> String {
        let mut out = String::new();
        for c in jamo.chars() {
            out += &composer.push(Jamo::from_compat(c));
        }
        out + &composer.preedit()
    }

    fn compose(jamo: &str) -> String {
        type_jamo(&mut Composer::default(), jamo)
    }

    #[test]
    fn composes_basic_syllables() {
        assert_eq!(compose("ㅎㅏㄴㅇㅣㄹ"), "한일");
        assert_eq!(compose("ㅇㅏㄴㄴㅕㅇ"), "안녕");
    }

    #[test]
    fn composes_compound_final_consonant() {
        assert_eq!(compose("ㄱㅏㅂㅅ"), "값");
        assert_eq!(compose("ㅇㅣㄹㄱ"), "읽");
    }

    #[test]
    fn final_consonant_moves_to_next_syllable_before_vowel() {
        assert_eq!(compose("ㄷㅏㄹㄱㅏ"), "달가");
        assert_eq!(compose("ㄷㅏㄹ"), "달");
        assert_eq!(compose("ㄱㅏㅂㅅㅣ"), "갑시");
        assert_eq!(compose("ㄱㅏㅂㅅㅇㅣ"), "값이");
    }

    #[test]
    fn composes_compound_vowel() {
        assert_eq!(compose("ㅇㅗㅏ"), "와");
        assert_eq!(compose("ㅇㅜㅓㄴ"), "원");
        assert_eq!(compose("ㅇㅡㅣ"), "의");
    }

    #[test]
    fn lone_jamo_stay_as_compatibility_jamo() {
        assert_eq!(compose("ㄱ"), "ㄱ");
        assert_eq!(compose("ㅏ"), "ㅏ");
        assert_eq!(compose("ㄱㄱ"), "ㄱㄱ");
        assert_eq!(compose("ㅏㅏ"), "ㅏㅏ");
        assert_eq!(compose("ㅗㅏ"), "ㅘ");
    }

    #[test]
    fn consonant_that_cannot_be_final_starts_new_syllable() {
        assert_eq!(compose("ㄱㅏㄸ"), "가ㄸ");
        assert_eq!(compose("ㄱㅏㄸㅏ"), "가따");
    }

    #[test]
    fn non_combinable_final_pair_starts_new_syllable() {
        assert_eq!(compose("ㄱㅏㄱㄷ"), "각ㄷ");
    }

    #[test]
    fn backspace_removes_one_jamo() {
        let mut c = Composer::default();
        assert_eq!(type_jamo(&mut c, "ㄱㅏㅂㅅ"), "값");
        assert!(c.backspace());
        assert_eq!(c.preedit(), "갑");
        assert!(c.backspace());
        assert_eq!(c.preedit(), "가");
        assert!(c.backspace());
        assert_eq!(c.preedit(), "ㄱ");
        assert!(c.backspace());
        assert_eq!(c.preedit(), "");
        assert!(!c.backspace());
    }

    #[test]
    fn backspace_splits_compound_vowel() {
        let mut c = Composer::default();
        type_jamo(&mut c, "ㅇㅗㅏ");
        assert!(c.backspace());
        assert_eq!(c.preedit(), "오");
    }

    #[test]
    fn backspace_after_commit_only_touches_current_syllable() {
        let mut c = Composer::default();
        assert_eq!(type_jamo(&mut c, "ㄷㅏㄹㄱㅏ"), "달가");
        assert!(c.backspace());
        assert_eq!(c.preedit(), "ㄱ");
        assert!(c.backspace());
        assert!(!c.backspace());
    }

    #[test]
    fn flush_commits_and_clears() {
        let mut c = Composer::default();
        type_jamo(&mut c, "ㅎㅏㄴ");
        assert_eq!(c.flush(), "한");
        assert!(c.is_empty());
        assert_eq!(c.preedit(), "");
        assert!(!c.backspace());
        assert_eq!(c.flush(), "");
    }
}
