use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Opts {
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,

    #[structopt(
        short,
        long,
        parse(from_os_str),
        default_value = "/etc/ordin/config.toml"
    )]
    pub config: PathBuf,
}

impl Opts {
    pub fn new() -> Self {
        Self::from_args()
    }
}
