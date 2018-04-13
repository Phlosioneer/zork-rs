
pub mod supp;
pub mod np;

use libc::c_int;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PromptType {
    InCharacter,
    OutOfCharacter,
}

impl From<c_int> for PromptType {
    fn from(other: c_int) -> PromptType {
        if other == 0 {
            PromptType::OutOfCharacter
        } else if other == 1 {
            PromptType::InCharacter
        } else {
            panic!("Invalid conversion from c_int to PromptType: {:?}", other);
        }
    }
}

impl From<PromptType> for c_int {
    fn from(other: PromptType) -> c_int {
        match other {
            PromptType::OutOfCharacter => 0,
            PromptType::InCharacter => 1,
        }
    }
}

