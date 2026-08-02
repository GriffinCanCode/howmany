mod converter;
mod reporter;
mod tests;

pub use converter::SarifConverter;
pub use reporter::SarifReporter;

/// The SARIF version this tool emits, and the only one it will validate.
pub const SARIF_VERSION: &str = "2.1.0";

/// The schema URI consumers use to locate the specification.
pub const SARIF_SCHEMA: &str =
    "https://docs.oasis-open.org/sarif/sarif/v2.1.0/errata01/os/schemas/sarif-schema-2.1.0.json";
