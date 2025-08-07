/// Dynamic completion support for TermAI
/// Provides enhanced completion capabilities beyond static clap completion
pub mod dynamic;
pub mod values;

pub use dynamic::DynamicCompleter;
#[allow(unused_imports)]
pub use values::CompletionValues;
