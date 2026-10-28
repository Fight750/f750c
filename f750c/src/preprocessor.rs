//! This module defines processor features, such as preprocessor directive processing and macro expansion.

use strum::{AsRefStr, EnumString};

#[derive(Debug, Clone, Copy, AsRefStr, EnumString)]
pub enum PreprocessorDirective {
    #[strum(serialize = "use")]
    Use,
    #[strum(serialize = "extends")]
    Extends,
    #[strum(serialize = "define")]
    Define,
    #[strum(serialize = "if")]
    If,
    #[strum(serialize = "elseif")]
    Elseif,
    #[strum(serialize = "else")]
    Else,
    #[strum(serialize = "ifdef")]
    IfDefined,
    #[strum(serialize = "ifndef")]
    IfNotDefined,
    #[strum(serialize = "endif")]
    BlockEnd,
}