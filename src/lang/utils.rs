use colored::ColoredString;

#[inline(always)]
pub fn is_digit(ch: u8) -> bool {
    ch.is_ascii_digit()
}

#[inline(always)]
pub fn is_alpha(ch: u8) -> bool {
    ch.is_ascii_lowercase() || ch.is_ascii_uppercase() || ch == b'_'
}

pub fn formatter(start: bool, end: bool, strings: &[ColoredString]) -> String {
    let mut build_string: String = String::new();

    match (start, end) {
        (true, true) => {
            build_string.push('\n');

            strings.iter().for_each(|s| {
                build_string.push_str(s.to_string().as_str());
            });

            build_string.push('\n');

            build_string
        }
        (false, false) => {
            strings.iter().for_each(|s| {
                build_string.push_str(s.to_string().as_str());
            });

            build_string
        }
        (true, false) => {
            build_string.push('\n');

            strings.iter().for_each(|s| {
                build_string.push_str(s.to_string().as_str());
            });

            build_string
        }
        (false, true) => {
            strings.iter().for_each(|s| {
                build_string.push_str(s.to_string().as_str());
            });

            build_string.push('\n');

            build_string
        }
    }
}
