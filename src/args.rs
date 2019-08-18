use crate::models::Args;
use clap::{App, Arg};

pub fn clap_app() -> Args {
    let matches = App::new("vcsi")
        .version("0.1.0")
        .about("Video Contact Sheet Generator")
        .author("Jonathan Frankel")
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .get_matches();
    let input_path = matches.value_of("INPUT").unwrap().to_string();
    let args: Args = Args {
        input_path: input_path,
        ..Default::default()
    };
    args
}
