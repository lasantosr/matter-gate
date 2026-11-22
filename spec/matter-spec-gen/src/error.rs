use thiserror::Error;

/// Errors produced during Matter specification parsing, type resolution, or code generation.
#[derive(Debug, Error)]
pub enum Error {
    /// XML deserialization failure.
    #[error("XML deserialization error: {0}")]
    Xml(#[from] quick_xml::DeError),

    /// Unhandled atomic type in `chip-types.xml`.
    #[error("Unhandled atomic type in chip-types.xml: {0}")]
    UnhandledAtomicType(String),

    /// Unknown Matter wire type in a cluster context.
    #[error("Unknown Matter wire type '{type_str}' for '{context_name}' in cluster '{cluster_name}'")]
    UnknownWireType {
        type_str: String,
        context_name: String,
        cluster_name: String,
    },

    /// Array attribute missing entry type.
    #[error("Array attribute '{context_name}' in cluster '{cluster_name}' has no entryType specified")]
    MissingArrayEntryType { context_name: String, cluster_name: String },

    /// Unrecognized command source.
    #[error("Unknown command source '{source_name}' on command '{command}'")]
    UnknownCommandSource { command: String, source_name: String },

    /// Unrecognized event priority.
    #[error("Unknown event priority '{priority}' on event '{event}'")]
    UnknownEventPriority { event: String, priority: String },

    /// Unsupported bitmap width.
    #[error("unsupported bitmap width `{width}` for `{name}`")]
    UnsupportedBitmapWidth { name: String, width: String },

    /// Unsupported bitmap type.
    #[error("unsupported bitmap type `{type_str}` for bitmap `{name}`")]
    UnsupportedBitmapType { name: String, type_str: String },

    /// Invalid cluster extension hex code.
    #[error("Invalid clusterExtension code: {0}")]
    InvalidClusterExtensionCode(String),

    /// Cluster extension references a cluster code not present in the model.
    #[error("clusterExtension references unknown cluster code: 0x{code:04X}")]
    UnknownClusterExtensionCode { code: u32 },

    /// Unrecognized category slug or domain.
    #[error("Unknown category '{category}'")]
    UnknownCategory { category: String },

    /// Malformed conformance expression in XML.
    #[error("Malformed conformance in '{context}': {reason}")]
    MalformedConformance { context: String, reason: String },

    /// Unresolved device type include requirement (`requireAttribute`/`requireCommand`/`requireEvent`).
    #[error("Unresolved device type include in '{device_type}' ({kind}): '{target}'")]
    UnresolvedDeviceTypeInclude {
        device_type: String,
        kind: String,
        target: String,
    },

    /// Invalid `@minLength` value.
    #[error("Invalid @minLength '{value}' in '{context}'")]
    InvalidMinLength { context: String, value: String },

    /// Failed to render token stream into text.
    #[error("Failed to render generated Rust tokens: {0}")]
    Render(String),

    /// Generic parse error.
    #[error("Parse error: {0}")]
    Parse(String),
}
