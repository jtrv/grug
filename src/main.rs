mod errors;
mod expand;
mod grepline;
mod hunk;
mod preview;
mod write;

use clap::{value_parser, Arg, ArgAction, Command};
use clap_complete::{generate, Shell};

use crate::errors::AppError;

fn build_cli() -> Command {
    Command::new("grug")
        .arg(
            Arg::new("expand")
                .short('e')
                .long("expand")
                .action(ArgAction::SetTrue)
                .help("Expand the lines from stdin into hunks"),
        )
        .arg(
            Arg::new("preview")
                .short('p')
                .long("preview")
                .action(ArgAction::SetTrue)
                .help("Preview diffs of file contents against the hunks piped to stdin"),
        )
        .arg(
            Arg::new("above")
                .short('A')
                .long("above")
                .value_name("LINES_ABOVE")
                .help("Include LINES_ABOVE lines above each line from stdin in the hunk"),
        )
        .arg(
            Arg::new("below")
                .short('B')
                .long("below")
                .value_name("LINES_BELOW")
                .help("Include LINES_BELOW lines below each line from stdin in the hunk"),
        )
        .arg(
            Arg::new("context")
                .short('C')
                .long("context")
                .value_name("CONTEXT_LINES")
                .help("Include CONTEXT_LINES above and below each line from stdin in the hunk"),
        )
        .arg(
            Arg::new("write")
                .short('w')
                .long("write")
                .action(ArgAction::SetTrue)
                .help("Apply stdin to files: edited hunks (@@@) or raw grep lines"),
        )
        .arg(
            Arg::new("completions")
                .long("completions")
                .value_name("SHELL")
                .value_parser(value_parser!(Shell))
                .help("Print shell completions for SHELL to stdout"),
        )
}

fn main() -> Result<(), AppError> {
    let matches = build_cli().get_matches();

    if let Some(&shell) = matches.get_one::<Shell>("completions") {
        let mut cmd = build_cli();
        generate(shell, &mut cmd, "grug", &mut std::io::stdout());
        return Ok(());
    }

    if matches.get_flag("expand") {
        expand::expand_to_hunks(&matches)?;
    } else if matches.get_flag("preview") {
        preview::diff_hunks()?;
    } else if matches.get_flag("write") {
        write::write_changes(&matches)?;
    } else {
        eprintln!("One of --expand, --preview, or --write must be provided.");
    }

    Ok(())
}
