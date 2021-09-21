use std::path::PathBuf;
use structopt::StructOpt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error(transparent)]
    ParseError(#[from] lore_parser::ParseError),

    #[error("Many validation errors oh no")]
    ValidationError(Vec<lore_parser::ValidationError>),

    #[error(transparent)]
    StoreError(#[from] lore_store::StoreError),

    #[error(transparent)]
    TranslationError(#[from] lore_codegen::EmitterError),

    #[error(transparent)]
    CodegenError(#[from] std::io::Error),
}

#[derive(Debug, StructOpt)]
#[structopt(name = "lore", about = "a little language to capture reality")]
struct Args {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum TargetLang {
    OCaml,
    GraphQL,
    Erlang,
    Elixir,
}

impl ToString for TargetLang {
    fn to_string(&self) -> String {
        match self {
            TargetLang::OCaml => "ocaml",
            TargetLang::GraphQL => "graphql",
            TargetLang::Erlang => "erlang",
            TargetLang::Elixir => "elixir",
        }
        .to_string()
    }
}

impl std::str::FromStr for TargetLang {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<TargetLang, String> {
        match s {
            "ocaml" => Ok(TargetLang::OCaml),
            "erlang" => Ok(TargetLang::Erlang),
            "elixir" => Ok(TargetLang::Elixir),
            "graphql" => Ok(TargetLang::GraphQL),
            _ => Err(format!("Could not find target: {}. Try one of: ocaml | elixir | erlang | graphql | rescript", s)),
        }
    }
}

#[derive(Debug, StructOpt)]
enum Command {
    Validate {
        #[structopt(name = "INPUT", parse(from_os_str))]
        inputs: Vec<PathBuf>,
    },

    Codegen {
        #[structopt(
            short = "",
            long = "target",
            name = "TARGET",
            help = "the target language to generate sources in"
        )]
        target: TargetLang,

        #[structopt(
            name = "INPUTS",
            help = "source .lore files to read",
            parse(from_os_str)
        )]
        inputs: Vec<PathBuf>,

        #[structopt(
            short = "o",
            long = "output-dir",
            name = "OUTPUT_DIR",
            help = "the output directory for the generated sources",
            default_value = "./gen",
            parse(from_os_str)
        )]
        output_dir: PathBuf,
    },
}

impl Command {
    pub fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Command::Validate { inputs } => {
                for input in inputs {
                    let mut parser = lore_parser::Parser::for_file(input.clone())?;
                    let validator = lore_parser::Validator::new();
                    if let Err(error) = validator.validate(parser.parse()?) {
                        println!("{}:\n\t{:#?}", input.to_str().unwrap(), error)
                    } else {
                        println!("{}\tOK", input.to_str().unwrap())
                    }
                }
                Ok(())
            }

            Command::Codegen {
                inputs,
                target,
                output_dir,
            } => {
                let mut store = lore_store::Store::new();
                for input in inputs {
                    let mut parser = lore_parser::Parser::for_file(input.clone())
                        .map_err(CliError::ParseError)?;
                    let validator = lore_parser::Validator::new();
                    let ast = validator
                        .validate(parser.parse().map_err(CliError::ParseError)?)
                        .map_err(CliError::ValidationError)?;
                    store.add_tree(ast).map_err(CliError::StoreError)?;
                }

                let source_set = match target {
                    TargetLang::OCaml => {
                        let emitter = lore_codegen::OCamlEmitter::new();
                        emitter
                            .translate(&store)
                            .map_err(CliError::TranslationError)?
                    }
                    _ => lore_codegen::SourceSet::empty(),
                };

                for source in source_set.sources() {
                    source.write(&output_dir).map_err(CliError::CodegenError)?;
                }

                Ok(())
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Args::from_args().cmd.run()
}
