use std::iter::Iterator;

pub fn match_keyword<I: Iterator<Item = char>>(chars: I) -> Option<usize> {
    let mut dfa = KeywordDFA { state: 0 };

    dfa.check(chars)
}

struct KeywordDFA {
    state: u8,
}

impl KeywordDFA {
    fn check<I: Iterator<Item = char>>(&mut self, iter: I) -> Option<usize> {
        let mut consumed = 0;

        for c in iter {
            if self.consume(c) {
                consumed += 1;
            }

            let next_state = self.state;

            if next_state == 1 {
                return Some(consumed);
            } else if next_state == 2 {
                return None;
            }
        }

        None
    }

    #[inline(always)]
    fn consume(&mut self, t: char) -> bool {
        let next_state = match self.state {
            0 => _state_0(t),
            1 => return false,
            2 => return false,
            3 => _state_3(t),
            4 => _state_4(t),
            5 => _state_5(t),
            6 => _state_6(t),
            7 => _state_7(t),
            8 => _state_8(t),
            9 => _state_9(t),
            10 => _state_10(t),
            11 => _state_11(t),
            12 => _state_12(t),
            13 => _state_13(t),
            14 => _state_14(t),
            15 => _state_15(t),
            16 => _state_16(t),
            17 => _state_17(t),
            18 => _state_18(t),
            19 => _state_19(t),
            20 => _state_20(t),
            21 => _state_21(t),
            22 => _state_22(t),
            23 => _state_23(t),
            24 => _state_24(t),
            25 => _state_25(t),
            26 => _state_26(t),
            27 => _state_27(t),
            28 => _state_28(t),
            29 => _state_29(t),
            30 => _state_30(t),
            31 => _state_31(t),
            32 => _state_32(t),
            33 => _state_33(t),
            34 => _state_34(t),
            35 => _state_35(t),
            36 => _state_36(t),
            37 => _state_37(t),
            38 => _state_38(t),
            39 => _state_39(t),
            40 => _state_40(t),
            41 => _state_41(t),
            42 => _state_42(t),
            43 => _state_43(t),
            44 => _state_44(t),
            45 => _state_45(t),
            _ => unreachable!(),
        };
        self.state = next_state;
        true
    }
}

#[inline(always)]
fn _state_0(t: char) -> u8 {
    match t {
        'a' => 3,
        'c' => 5,
        'e' => 9,
        'f' => 12,
        'i' => 18,
        'n' => 19,
        'o' => 21,
        'p' => 22,
        'r' => 26,
        's' => 31,
        't' => 35,
        'v' => 40,
        'w' => 42,
        _ => 2,
    }
}
#[inline(always)]
fn _state_3(t: char) -> u8 {
    match t {
        'n' => 4,
        _ => 2,
    }
}
#[inline(always)]
fn _state_4(t: char) -> u8 {
    match t {
        'd' => 1,
        _ => 2,
    }
}
#[inline(always)]
fn _state_5(t: char) -> u8 {
    match t {
        'l' => 6,
        _ => 2,
    }
}
#[inline(always)]
fn _state_6(t: char) -> u8 {
    match t {
        'a' => 7,
        _ => 2,
    }
}
#[inline(always)]
fn _state_7(t: char) -> u8 {
    match t {
        's' => 8,
        _ => 2,
    }
}
#[inline(always)]
fn _state_8(t: char) -> u8 {
    match t {
        's' => 1,
        _ => 2,
    }
}
#[inline(always)]
fn _state_9(t: char) -> u8 {
    match t {
        'l' => 10,
        _ => 2,
    }
}
#[inline(always)]
fn _state_10(t: char) -> u8 {
    match t {
        's' => 11,
        _ => 2,
    }
}
#[inline(always)]
fn _state_11(t: char) -> u8 {
    match t {
        'e' => 1,
        _ => 2,
    }
}
#[inline(always)]
fn _state_12(t: char) -> u8 {
    match t {
        'a' => 13,
        'o' => 16,
        'u' => 17,
        _ => 2,
    }
}
#[inline(always)]
fn _state_13(t: char) -> u8 {
    match t {
        'l' => 14,
        _ => 2,
    }
}
#[inline(always)]
fn _state_14(t: char) -> u8 {
    match t {
        's' => 15,
        _ => 2,
    }
}
#[inline(always)]
fn _state_15(t: char) -> u8 {
    match t {
        'e' => 1,
        _ => 2,
    }
}
#[inline(always)]
fn _state_16(t: char) -> u8 {
    match t {
        'r' => 1,
        _ => 2,
    }
}
#[inline(always)]
fn _state_17(t: char) -> u8 {
    match t {
        'n' => 1,
        _ => 2,
    }
}
#[inline(always)]
fn _state_18(t: char) -> u8 {
    match t {
        'f' => 1,
        _ => 2,
    }
}
#[inline(always)]
fn _state_19(t: char) -> u8 {
    match t {
        'i' => 20,
        _ => 2,
    }
}
#[inline(always)]
fn _state_20(t: char) -> u8 {
    match t {
        'l' => 1,
        _ => 2,
    }
}
#[inline(always)]
fn _state_21(t: char) -> u8 {
    match t {
        'r' => 1,
        _ => 2,
    }
}
#[inline(always)]
fn _state_22(t: char) -> u8 {
    match t {
        'r' => 23,
        _ => 2,
    }
}
#[inline(always)]
fn _state_23(t: char) -> u8 {
    match t {
        'i' => 24,
        _ => 2,
    }
}
#[inline(always)]
fn _state_24(t: char) -> u8 {
    match t {
        'n' => 25,
        _ => 2,
    }
}
#[inline(always)]
fn _state_25(t: char) -> u8 {
    match t {
        't' => 1,
        _ => 2,
    }
}
#[inline(always)]
fn _state_26(t: char) -> u8 {
    match t {
        'e' => 27,
        _ => 2,
    }
}
#[inline(always)]
fn _state_27(t: char) -> u8 {
    match t {
        't' => 28,
        _ => 2,
    }
}
#[inline(always)]
fn _state_28(t: char) -> u8 {
    match t {
        'u' => 29,
        _ => 2,
    }
}
#[inline(always)]
fn _state_29(t: char) -> u8 {
    match t {
        'r' => 30,
        _ => 2,
    }
}
#[inline(always)]
fn _state_30(t: char) -> u8 {
    match t {
        'n' => 1,
        _ => 2,
    }
}
#[inline(always)]
fn _state_31(t: char) -> u8 {
    match t {
        'u' => 32,
        _ => 2,
    }
}
#[inline(always)]
fn _state_32(t: char) -> u8 {
    match t {
        'p' => 33,
        _ => 2,
    }
}
#[inline(always)]
fn _state_33(t: char) -> u8 {
    match t {
        'e' => 34,
        _ => 2,
    }
}
#[inline(always)]
fn _state_34(t: char) -> u8 {
    match t {
        'r' => 1,
        _ => 2,
    }
}
#[inline(always)]
fn _state_35(t: char) -> u8 {
    match t {
        'h' => 36,
        'r' => 38,
        _ => 2,
    }
}
#[inline(always)]
fn _state_36(t: char) -> u8 {
    match t {
        'i' => 37,
        _ => 2,
    }
}
#[inline(always)]
fn _state_37(t: char) -> u8 {
    match t {
        's' => 1,
        _ => 2,
    }
}
#[inline(always)]
fn _state_38(t: char) -> u8 {
    match t {
        'u' => 39,
        _ => 2,
    }
}
#[inline(always)]
fn _state_39(t: char) -> u8 {
    match t {
        'e' => 1,
        _ => 2,
    }
}
#[inline(always)]
fn _state_40(t: char) -> u8 {
    match t {
        'a' => 41,
        _ => 2,
    }
}
#[inline(always)]
fn _state_41(t: char) -> u8 {
    match t {
        'r' => 1,
        _ => 2,
    }
}
#[inline(always)]
fn _state_42(t: char) -> u8 {
    match t {
        'h' => 43,
        _ => 2,
    }
}
#[inline(always)]
fn _state_43(t: char) -> u8 {
    match t {
        'i' => 44,
        _ => 2,
    }
}
#[inline(always)]
fn _state_44(t: char) -> u8 {
    match t {
        'l' => 45,
        _ => 2,
    }
}
#[inline(always)]
fn _state_45(t: char) -> u8 {
    match t {
        'e' => 1,
        _ => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_keywords() {
        let keywords = vec![
            "and", "class", "else", "false", "for", "fun", "if", "nil", "or", "print", "return",
            "super", "this", "true", "var", "while", // starts with key words
        ];
        for kw in keywords {
            assert_eq!(match_keyword(kw.chars()), Some(kw.len()));
        }
    }
    #[test]
    fn test_start_with_keyword() {
        assert_eq!(match_keyword("function".chars()), Some(3)); // fun
        assert_eq!(match_keyword("forever".chars()), Some(3)); // for
    }
    #[test]
    fn test_non_keywords() {
        let non_keywords = vec!["adsfasf", "loOk", "Function", "True", "123"];
        for kw in non_keywords {
            assert_eq!(match_keyword(kw.chars()), None);
        }
    }
}
