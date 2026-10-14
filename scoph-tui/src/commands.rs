use clap::Parser;
use scoph::interpreter::Interpreter;

#[derive(clap::Parser)]
#[command(override_usage = "")]
pub enum ReplCommand {
    #[command(alias = "q")]
    Quit,
    #[command(alias = "l")]
    Load { path: String },

    #[command(alias = "s")]
    Step,

    #[command(visible_aliases = &["b", "break"])]
    Breakpoint { node_id: String },

    #[command(visible_aliases = &["c", "cont"])]
    Continue,
}

impl ReplCommand {
    pub fn apply(self, intrp: &mut Interpreter) {}
}

pub enum ExtractError {
    BadText,
    ClapError(clap::error::Error),
}

pub fn extract_command(
    text_state: &mut impl tui_prompts::State,
) -> Result<ReplCommand, ExtractError> {
    let mut text = "foo ".to_string();
    text.push_str(text_state.value());
    let res = match shellwords::split(&text) {
        Ok(s) => ReplCommand::try_parse_from(s).map_err(ExtractError::ClapError),
        Err(_) => Err(ExtractError::BadText),
    };

    text_state.truncate();

    res
}
