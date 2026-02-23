//! LSP language features: hover, completion, go-to-definition, references, symbols, format.

mod completion;
mod definition;
mod format;
mod hover;
mod references;
mod symbols;
mod utils;

#[cfg(test)]
mod tests;

pub use completion::get_completion;
pub use definition::find_definition;
pub use format::format_document;
pub use hover::{find_element_hover, find_relation_hover, get_hover};
pub use references::find_references;
pub use symbols::get_document_symbols;
pub use utils::{collect_elements, is_ident_char, last_token, word_bounds};
