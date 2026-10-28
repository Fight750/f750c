//! This module defines processor features, such as preprocessor directive processing and macro expansion.

use std::fmt::Display;
use std::str::FromStr;
use log::debug;
use strum::{AsRefStr, Display, EnumString};
use thiserror::Error;
use crate::context::CompilerContext;
use crate::parser::ParseError;
use crate::semantic::{SemanticDerefKind, SemanticImmediateType, SemanticOperand, SemanticRepr, SemanticSource};
use crate::tokenizer;
use crate::tokenizer::{Token, TokenStream};

#[derive(Debug, Clone, Error)]
pub enum PreprocessorError {
    #[error("Unexpected token: {0}")]
    UnexpectedToken(String),
    #[error("Unexpected end of line")]
    UnexpectedEOL,
    #[error("Unknown preprocessor directive: {0}")]
    UnknownDirective(String),
    #[error("Unsupported preprocessor directive: {0}")]
    UnsupportedDirective(PreprocessorDirectiveKind),
    #[error("Unknown namespace: {0}")]
    UnknownNamespace(String),
    #[error("Alias already exists: {0}")]
    AliasAlreadyExists(String),
    #[error("Ambiguous alias: {0}")]
    AmbiguousAlias(String),
}

impl From<ParseError> for PreprocessorError {
    fn from(err: ParseError) -> Self {
        match err {
            ParseError::UnexpectedToken(token) => PreprocessorError::UnexpectedToken(token),
            ParseError::UnexpectedEOL => PreprocessorError::UnexpectedEOL,
            _ => PreprocessorError::UnexpectedToken(format!("Parse error: {:?}", err)),
        }
    }
}

pub type PreprocessorResult<T> = Result<T, PreprocessorError>;

#[derive(Debug, Clone, Copy, Display, AsRefStr, EnumString)]
pub enum PreprocessorDirectiveKind {
    #[strum(serialize = "use")]
    Use,
    #[strum(serialize = "inherit")]
    Inherit,
    #[strum(serialize = "local")]
    LocalBinding,
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

#[derive(Debug, Clone)]
pub enum PreprocessorDirective {
    Use(String, String),
}

impl PreprocessorDirective {
    pub fn parse_line(stream: &mut TokenStream) -> PreprocessorResult<Self> {
        stream.expect_non_whitespace(Token::Hashtag)?;
        let Token::Raw(directive_str) = stream.next_non_whitespace_or_err()? else {
            return Err(PreprocessorError::UnexpectedEOL);
        };

        let kind = PreprocessorDirectiveKind::from_str(&directive_str)
            .map_err(|_| PreprocessorError::UnknownDirective(directive_str.clone()))?;

        // whitespace after directive
        stream.expect(Token::Whitespace)?;

        match kind {
            PreprocessorDirectiveKind::Use => {
                let Token::Raw(namespace) = stream.next_non_whitespace_or_err()? else {
                    return Err(PreprocessorError::UnexpectedEOL);
                };

                let alias = if stream.peek_non_whitespace().is_some() {
                    let Token::Raw(alias) = stream.next_non_whitespace_or_err()? else {
                        return Err(PreprocessorError::UnexpectedToken("Expected alias after namespace".to_string()));
                    };

                    alias

                } else {
                    namespace.split(tokenizer::CHAR_DOT)
                        .last()
                        .ok_or_else(|| PreprocessorError::UnexpectedToken("Expected alias after namespace".to_string()))?
                        .to_string()
                };

                Ok(PreprocessorDirective::Use(namespace.clone(), alias))
            }
            _ => Err(PreprocessorError::UnsupportedDirective(kind)),
        }
    }

    pub fn kind(&self) -> PreprocessorDirectiveKind {
        match self {
            PreprocessorDirective::Use(_, _) => PreprocessorDirectiveKind::Use,
        }
    }
}

impl Display for PreprocessorDirective {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PreprocessorDirective::Use(namespace, alias) => {
                write!(f, "#use {} {}", namespace, alias)
            }
        }
    }
}

pub fn preprocess(source: &mut SemanticSource, ctx: &mut CompilerContext) -> PreprocessorResult<()> {
    expand_use_directives(source, ctx)?;

    Ok(())
}

fn expand_use_directives(source: &mut SemanticSource, ctx: &mut CompilerContext) -> PreprocessorResult<()> {
    // scan and map aliases
    for line in source.iter_mut() {
        let SemanticRepr::PreprocessorDirective(PreprocessorDirective::Use(ns, alias)) = &line.repr else {
            continue;
        };

        if !ctx.has_namespace(ns) {
            return Err(PreprocessorError::UnknownNamespace(ns.clone()));
        }

        if ctx.has_namespace(alias) {
            return Err(PreprocessorError::AmbiguousAlias(alias.clone()));
        }

        let source_ctx = ctx.get_source_ctx_mut(&line.namespace);

        if source_ctx.has_alias(alias) {
            return Err(PreprocessorError::AliasAlreadyExists(alias.clone()));
        }

        source_ctx.add_alias(alias, ns);
    }

    // replace namespace references
    for line in source.iter_mut() {
        let SemanticRepr::Instruction(instruction) = &mut line.repr else {
            continue;
        };
        let source_ctx = ctx.get_source_ctx_mut(&line.namespace);

        for operand in instruction.operands.iter_mut() {
            match operand {
                SemanticOperand::Immediate(imm) => {
                    if let Some(symbol) = imm.get_symbol_mut() {
                        let prev = symbol.name.clone();

                        source_ctx.resolve_namespace_symbol(&mut symbol.name);
                    }
                }
                SemanticOperand::Deref(deref) => {
                    if let SemanticDerefKind::Binding(symbol) = &mut deref.kind {
                        source_ctx.resolve_namespace_symbol(&mut symbol.name);
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}