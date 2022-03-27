use lazy_static::lazy_static;
use regex::Regex;

pub fn match_email(text: &str) -> bool {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"[^@\s]+@([[:word:]]+\.)+[[:word:]]+$").unwrap();
    }
    RE.is_match(text)
}

pub fn match_id(text: &str) -> bool {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^\d{5,12}$").unwrap();
    }
    RE.is_match(text)
}

pub fn match_password(text: &str) -> bool {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^[[:alnum:]]{7,15}$").unwrap();
    }
    RE.is_match(text)
}
