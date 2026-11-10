//! Every user-facing string of the interactive interface.
//!
//! Strings live in one module so that a later RFC can add a second language
//! without touching screen logic (RFC-0018, decision on interface language).

pub(crate) const PRODUCT: &str = "Anti-Gravital";
pub(crate) const NEW_PROJECT: &str = "New project";

pub(crate) const STEP_NAME: &str = "Name";
pub(crate) const STEP_TEMPLATE: &str = "Template";
pub(crate) const STEP_REVIEW: &str = "Review";

pub(crate) const NAME_LABEL: &str = "Project name";
pub(crate) const NAME_HINT: &str =
    "Letters, numbers, hyphens and underscores. It is also the directory name.";
pub(crate) const ERROR_PREFIX: &str = "Error: ";
pub(crate) const NAME_TAKEN: &str = "a file or directory with this name already exists here";

pub(crate) const TEMPLATE_LABEL: &str = "Choose a starting template";
pub(crate) const FILES_LABEL: &str = "Files it creates";

pub(crate) const REVIEW_LABEL: &str = "Review before creating";
pub(crate) const REVIEW_DIRECTORY: &str = "Directory";
pub(crate) const REVIEW_TEMPLATE: &str = "Template";
pub(crate) const REVIEW_FILES: &str = "Files to be created";
pub(crate) const REVIEW_COMMAND: &str = "Equivalent command";
pub(crate) const REVIEW_NOTHING_WRITTEN: &str = "Nothing has been written yet.";

pub(crate) const KEYS_NAME: &str = "Enter: continue    Esc: cancel";
pub(crate) const KEYS_TEMPLATE: &str =
    "Up/Down: select    Enter: continue    Esc: back    q: cancel";
pub(crate) const KEYS_REVIEW: &str = "Enter: create project    Esc: back    q: cancel";

pub(crate) const ABORTED: &str = "Cancelled. Nothing was written.";
pub(crate) const EQUIVALENT_COMMAND: &str = "Equivalent non-interactive command:";

pub(crate) const LINE_NAME_PROMPT: &str = "Project name: ";
pub(crate) const LINE_TEMPLATE_PROMPT: &str = "Template number [1]: ";
pub(crate) const LINE_TEMPLATE_INVALID: &str = "Enter a number from the list.";
pub(crate) const LINE_CONFIRM_PROMPT: &str = "Create this project? [Y/n]: ";
