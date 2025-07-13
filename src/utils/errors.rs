use std::io;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, HowManyError>;

#[derive(Error, Debug)]
pub enum HowManyError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("File processing error: {message}")]
    FileProcessing { message: String },
    
    #[error("Invalid configuration: {message}")]
    InvalidConfig { message: String },
    
    #[error("Filter error: {message}")]
    Filter { message: String },
    
    #[error("Counter error: {message}")]
    Counter { message: String },
    
    #[error("Display error: {message}")]
    Display { message: String },
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
}

impl HowManyError {
    pub fn file_processing(message: impl Into<String>) -> Self {
        Self::FileProcessing { message: message.into() }
    }
    
    pub fn invalid_config(message: impl Into<String>) -> Self {
        Self::InvalidConfig { message: message.into() }
    }
    
    pub fn filter(message: impl Into<String>) -> Self {
        Self::Filter { message: message.into() }
    }
    
    pub fn counter(message: impl Into<String>) -> Self {
        Self::Counter { message: message.into() }
    }
    
    pub fn display(message: impl Into<String>) -> Self {
        Self::Display { message: message.into() }
    }
} 