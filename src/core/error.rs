use std::path::PathBuf;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, OpenSpecError>;

#[derive(Error, Debug)]
pub enum OpenSpecError {
    #[error("OpenSpec directory not found. Run 'openspec init' first.")]
    NotInitialized,

    #[error("Changes directory not found. Run 'openspec init' first.")]
    NoChangesDirectory,

    #[error("Change '{name}' not found")]
    ChangeNotFound { name: String },

    #[error("Change '{name}' already exists at {path}")]
    ChangeAlreadyExists { name: String, path: PathBuf },

    #[error("Spec '{id}' not found at openspec/specs/{id}/spec.md")]
    SpecNotFound { id: String },

    #[error("Schema '{name}' not found. Available schemas: {available}")]
    SchemaNotFound { name: String, available: String },

    #[error("Failed to load schema at '{path}'")]
    SchemaLoadError {
        path: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Schema validation failed: {message}")]
    SchemaValidation { message: String },

    #[error("Cyclic dependency detected: {cycle}")]
    CyclicDependency { cycle: String },

    #[error("Invalid artifact reference: '{artifact}' does not exist in schema")]
    InvalidArtifactReference { artifact: String },

    #[error("Duplicate artifact ID: {id}")]
    DuplicateArtifact { id: String },

    #[error("Failed to parse YAML in {context}: {message}")]
    YamlParse { context: String, message: String },

    #[error("Failed to parse spec at {path}: {message}")]
    SpecParse { path: PathBuf, message: String },

    #[error("Failed to parse change at {path}: {message}")]
    ChangeParse { path: PathBuf, message: String },

    #[error("Spec must have a {section} section")]
    MissingSpecSection { section: String },

    #[error("Change must have a {section} section")]
    MissingChangeSection { section: String },

    #[error("Invalid change name: {reason}")]
    InvalidChangeName { reason: String },

    #[error("Template not found: {path}")]
    TemplateNotFound { path: PathBuf },

    #[error("Failed to read {path}")]
    IoRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to write {path}")]
    IoWrite {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Insufficient permissions to write to {path}")]
    PermissionDenied { path: PathBuf },

    #[error("Archive '{name}' already exists")]
    ArchiveAlreadyExists { name: String },

    #[error("Invalid profile '{profile}'. Available profiles: {available}")]
    InvalidProfile { profile: String, available: String },

    #[error("At least one tool must be selected")]
    NoToolSelected,

    #[error("Delta operation failed for {spec}: {reason}")]
    DeltaFailed { spec: String, reason: String },

    #[error("Validation failed with {errors} error(s)")]
    ValidationFailed { errors: usize },

    #[error("{0}")]
    Custom(String),
}

impl OpenSpecError {
    pub fn change_not_found(name: impl Into<String>) -> Self {
        Self::ChangeNotFound { name: name.into() }
    }

    pub fn spec_not_found(id: impl Into<String>) -> Self {
        Self::SpecNotFound { id: id.into() }
    }

    pub fn schema_not_found(name: impl Into<String>, available: impl Into<String>) -> Self {
        Self::SchemaNotFound {
            name: name.into(),
            available: available.into(),
        }
    }

    pub fn yaml_parse(context: impl Into<String>, message: impl Into<String>) -> Self {
        Self::YamlParse {
            context: context.into(),
            message: message.into(),
        }
    }

    pub fn invalid_change_name(reason: impl Into<String>) -> Self {
        Self::InvalidChangeName {
            reason: reason.into(),
        }
    }
}

impl From<std::io::Error> for OpenSpecError {
    fn from(err: std::io::Error) -> Self {
        Self::Custom(err.to_string())
    }
}

impl From<serde_yaml::Error> for OpenSpecError {
    fn from(err: serde_yaml::Error) -> Self {
        Self::yaml_parse("yaml", err.to_string())
    }
}
