#![allow(
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::unreadable_literal,
    clippy::needless_pass_by_value,
    clippy::single_match_else,
    clippy::uninlined_format_args,
    clippy::needless_continue,
    clippy::match_same_arms,
    clippy::unused_self,
    clippy::struct_excessive_bools,
    clippy::too_many_lines,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::similar_names,
    clippy::module_name_repetitions,
    clippy::wildcard_imports,
    clippy::enum_glob_use,
    clippy::if_not_else,
    clippy::manual_let_else,
    clippy::items_after_statements
)]

pub mod board;
pub mod config;
pub mod i18n;
pub mod item;
pub mod iwe;
pub mod link;
pub mod storage;
pub mod text_input;
pub mod viewer;
pub mod vi;
