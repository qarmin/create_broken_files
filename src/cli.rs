use clap::{Parser, Subcommand};
use jwalk::WalkDir;
use std::path::Path;
use std::process;

#[derive(Parser)]
#[command(name = "Create Broken Files")]
#[command(author = "Rafał Mikrut")]
#[command(version = "3.1.0")]
#[command(
    about = "Creates broken files from provided ones, to e.g. check parsers", long_about = None
)]
pub struct Cli {
    #[arg(short, long, value_name = "INPUT", help = "Points at file/folder that will be taken as input to create broken files(only checking with depth 1 - without checking subfolders).")]
    input_path: String,

    #[arg(short, long, value_name = "OUTPUT", help = "Folder to which broken files will be saved.")]
    pub(crate) output_path: String,

    #[arg(short, long, value_name = "NUMBER", help = "Number of broken files that will be created for each found file.")]
    pub(crate) number_of_broken_files: u32,

    #[arg(
        short,
        long,
        default_value = "false",
        value_name = "IS_CHARACTER_MODE",
        help = "Runs fuzzer in character mode, so output will be utf8 conformant(of course if input also is)",
        default_value_t = false
    )]
    pub(crate) character_mode: bool,

    #[arg(short = 'm', long, value_name = "CONNECTING_MULTIPLE_FILES", help = "Connecting multiple files instead just.", default_value_t = false)]
    pub(crate) connect_multiple_files: bool,

    #[arg(
        short,
        long,
        num_args = 1..,
        value_name = "WORDS",
        help = "List of items that will be added randomly to code. The best results to check language parsers you got when here is a full list of used keywords and symbols(new, let, var, ;, :, ? etc.)"
    )]
    pub(crate) special_words: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// does testing things
    Test {
        /// lists test values
        #[arg(short, long)]
        list: bool,
    },
}
pub(crate) fn parse_cli() -> (Vec<String>, Cli) {
    let cli = Cli::parse();

    if !Path::new(&cli.output_path).is_dir() || !Path::new(&cli.input_path).exists() {
        println!("Input and output paths must exists");
        process::exit(1);
    }

    let files_to_check = collect_files_to_check(&cli.input_path);

    (files_to_check, cli)
}

fn collect_files_to_check(input_path: &str) -> Vec<String> {
    if !Path::new(&input_path).exists() {
        println!("Path should exists {input_path}");
        process::exit(1);
    }

    let mut files_to_check = Vec::new();
    for i in WalkDir::new(input_path).max_depth(999).into_iter().flatten() {
        let path = i.path();
        if path.is_file() {
            files_to_check.push(path.to_string_lossy().to_string());
        }
    }

    files_to_check.retain(|e| e.contains('.'));

    if files_to_check.is_empty() {
        println!("No files to check");
        process::exit(1);
    }

    files_to_check
}
