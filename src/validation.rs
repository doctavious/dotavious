use std::borrow::Cow;

pub type ValidationResult<T> = std::result::Result<T, Vec<ValidationError>>;

#[derive(Debug, PartialEq, Clone)]
pub struct ValidationError {
    pub message: Cow<'static, str>,
    pub field: Cow<'static, str>,
}