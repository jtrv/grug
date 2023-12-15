mod errors;
mod expand;
/*mod preview;*/
mod write;

use clap::{App, Arg};

use crate::errors::AppError;

fn main() -> Result<(), AppError> {
    let matches = App::new("grug")
        .arg(
            Arg::new("expand")
                .short('e')
                .long("expand")
                .takes_value(false)
                .help("Expand the lines from stdin into hunks"),
        )
        /*.arg(
            Arg::new("preview")
                .short('p')
                .long("preview")
                .takes_value(false)
                .help("Preview diffs from the hunks passed to stdin"),
        )*/
        .arg(
            Arg::new("above")
                .short('A')
                .long("above")
                .takes_value(true)
                .value_name("LINES_ABOVE")
                .help("Include LINES_ABOVE lines above each line from stdin in the hunk"),
        )
        .arg(
            Arg::new("below")
                .short('B')
                .long("below")
                .takes_value(true)
                .value_name("LINES_BELOW")
                .help("Include LINES_BELOW lines below each line from stdin in the hunk"),
        )
        .arg(
            Arg::new("context")
                .short('C')
                .long("context")
                .takes_value(true)
                .value_name("CONTEXT_LINES")
                .help("Include CONTEXT_LINES above and below each line from stdin in the hunk"),
        )
        .arg(
        Arg::new("write")
            .short('w')
            .long("write")
            .takes_value(false)
            .help("Replace lines in files based on input from stdin"),
        )
        .get_matches();

    if matches.is_present("expand") {
        expand::expand_to_hunks(&matches)?;
    /*} else if matches.is_present("preview") {
        preview::diff_hunks()?;*/
    } else if matches.is_present("write") {
        write::write_changes(&matches)?;
    } else {
        eprintln!("Either --expand or --write flag must be provided.");
    }

    Ok(())
}
