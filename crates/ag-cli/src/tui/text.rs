//! Every user-facing string of the interactive interface.
//!
//! Strings live in one module so that a later RFC can add a second language
//! without touching screen logic (RFC-0018, decision on interface language).

pub(crate) const PRODUCT: &str = "Anti-Gravital";
pub(crate) const NEW_PROJECT: &str = "New project";
pub(crate) const CONFIGURE_PROJECT: &str = "Configure project";

pub(crate) const HOME_TITLE: &str = "Home";
pub(crate) const HOME_IN_PROJECT: &str = "Project detected. What would you like to do?";
pub(crate) const HOME_OUTSIDE_PROJECT: &str = "No project detected. What would you like to do?";
pub(crate) const KEYS_HOME: &str =
    "j/k: move    Enter: run    r: refresh    ?: help    Esc/q: quit    Ctrl-C: close";

// --- Home status panel (context design §2 + home UX design §6) -------------
// New strings are additive. Frozen tests reference these constants by name,
// so wording changes here never weaken an assertion in home.rs.

pub(crate) const STATUS_TITLE: &str = "PROJECT STATUS";
pub(crate) const STATE_NO_PROJECT: &str = ". NO PROJECT";
pub(crate) const STATE_RUNNING: &str = "* RUNNING";
pub(crate) const STATE_UNKNOWN: &str = "? UNKNOWN";

pub(crate) const LABEL_PROJECT: &str = "Project";
pub(crate) const LABEL_TEMPLATE: &str = "Template";
pub(crate) const LABEL_STATE: &str = "State";
pub(crate) const LABEL_PID: &str = "PID";
pub(crate) const LABEL_BIND: &str = "bind";

pub(crate) const NAME_UNKNOWN: &str = "--";
pub(crate) const TEMPLATE_UNKNOWN: &str = "unknown";

pub(crate) const HINT_NO_PROJECT: &str =
    "No config.toml found here. Nothing has been created or changed.";
pub(crate) const HINT_RUNNING: &str = "Instance confirmed. Continue development will reuse it.";
pub(crate) const HINT_UNKNOWN: &str = "No confirmed active instance. Press r to retry, ? for help.";

pub(crate) const NOTICE_CREATE_DISABLED: &str =
    "A project is already detected here. Run 'ag new' from outside this directory (cd ..).";

pub(crate) const KEYS_HELP: &str =
    "? : toggle help    r : refresh    j/k : move    Enter : run    Esc/q : quit";

pub(crate) const STEP_NAME: &str = "Name";
pub(crate) const STEP_TEMPLATE: &str = "Template";
pub(crate) const STEP_REVIEW: &str = "Review";

pub(crate) const NAME_LABEL: &str = "Project name";
pub(crate) const NAME_HINT: &str =
    "Letters, numbers, hyphens and underscores. It is also the directory name.";
pub(crate) const ERROR_PREFIX: &str = "Error: ";
pub(crate) const NAME_TAKEN: &str = "a file or directory with this name already exists here";

pub(crate) const TEMPLATE_LABEL: &str = "Choose a starting template";
pub(crate) const TEMPLATE_POSITION: &str = "Template";
pub(crate) const TEMPLATE_MORE_ABOVE: &str = "more above";
pub(crate) const TEMPLATE_MORE_BELOW: &str = "more below";
pub(crate) const TEMPLATE_FILES_PENDING: &str =
    "(files are defined by the template; not embedded in this build)";
pub(crate) const FILES_LABEL: &str = "Files it creates";

pub(crate) const REVIEW_LABEL: &str = "Review before creating";
pub(crate) const REVIEW_DIRECTORY: &str = "Directory";
pub(crate) const REVIEW_TEMPLATE: &str = "Template";
pub(crate) const REVIEW_EXTERNAL: &str = "External requirements";
pub(crate) const REVIEW_FILES: &str = "Files to be created";
pub(crate) const REVIEW_COMMAND: &str = "Equivalent command";
pub(crate) const REVIEW_NOTHING_WRITTEN: &str = "Nothing has been written yet.";

pub(crate) const KEYS_NAME: &str = "Enter: continue    Esc: cancel";
pub(crate) const KEYS_TEMPLATE: &str =
    "Up/Down: select    Enter: continue    Esc: back    q: cancel";
pub(crate) const KEYS_REVIEW: &str = "Enter: create project    Esc: back    q: cancel";

pub(crate) const STEP_FIELDS: &str = "Fields";
pub(crate) const CONFIG_REVIEW_LABEL: &str = "Changes to write to config.toml";
pub(crate) const KEYS_CONFIG_FIELDS: &str =
    "Up/Down: select    Enter: edit    r: review    Esc/q: cancel";
pub(crate) const KEYS_CONFIG_EDIT: &str = "Type: replace value    Enter: apply    Esc: cancel";
pub(crate) const KEYS_CONFIG_REVIEW: &str = "Enter: save    Esc: back to fields    q: cancel";

pub(crate) const ABORTED: &str = "Cancelled. Nothing was written.";
pub(crate) const EQUIVALENT_COMMAND: &str = "Equivalent non-interactive command:";

pub(crate) const LINE_NAME_PROMPT: &str = "Project name: ";
pub(crate) const LINE_TEMPLATE_PROMPT: &str = "Template number [1]: ";
pub(crate) const LINE_TEMPLATE_INVALID: &str = "Enter a number from the list.";
pub(crate) const LINE_CONFIRM_PROMPT: &str = "Create this project? [Y/n]: ";
