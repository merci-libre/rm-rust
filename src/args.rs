use clap::Parser;
#[derive(Parser, Debug)]
pub struct Removeargs {
    #[arg(long, short)]
    pub force: bool,
    #[arg(short)]
    ///prompt before every removal. (Warning: you will delete entire directories if you prompt it
    ///to, there is no diving into directories like the original!)
    pub i: bool,
    #[arg(short = 'I')]
    ///prompt once before every removal.
    pub x: bool,

    #[arg(long, default_value = "always")]
    ///prompt according to WHEN: never, once (-I), or always
    ///(-i); without WHEN, prompt always
    pub interactive: String,
    #[arg(long, short)]
    ///remove directories and their contents in their entirety.
    pub recursive: bool,
    #[arg(long, short)]
    ///remove empty directory.
    pub dir: bool,
    #[arg(long = "no-preserve-root")]
    ///do not treat '/' specially. (as of now, not functional due to limitations set by std::fs)
    pub no_preserve_root: bool,
    #[arg(long = "preserve-root", default_value = "all")]
    ///do not remove '/';
    ///Options: (all|none)
    pub preserve_root: String,
    #[arg(long, short)]
    ///show progress of the program.
    pub verbose: bool,
    #[arg(long, short = 'D')]
    ///show debugging information
    pub debug: bool,
    pub file: Vec<String>,
}
