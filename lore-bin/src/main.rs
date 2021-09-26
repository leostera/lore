use miette::Result;
use std::path::PathBuf;
use structopt::StructOpt;

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

        #[structopt(
            long = "dump-ast",
            help = "prints the AST of the file on successful parsing"
        )]
        dump_ast: bool,
    },

    Query {
        #[structopt(name = "INPUT", parse(from_os_str))]
        inputs: Vec<PathBuf>,

        #[structopt(short = "q", long = "query", name = "QUERY")]
        query: String,
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
    pub fn run(self) -> Result<()> {
        match self {
            Command::Validate { inputs, dump_ast } => {
                for input in inputs {
                    let mut parser = lore_parser::Parser::for_file(input.clone())?;
                    let validator = lore_parser::Validator::new();
                    let result = validator.validate(parser.parse()?)?;
                    if dump_ast {
                        dbg!(result);
                    }
                }
                Ok(())
            }

            Command::Query { inputs, query } => {
                let mut store = lore_store::Store::new();
                for input in inputs {
                    let mut parser = lore_parser::Parser::for_file(input.clone())?;
                    let validator = lore_parser::Validator::new();
                    let ast = validator.validate(parser.parse()?)?;
                    store.add_tree(ast)?;
                }

                println!("QUERY: {}", &query);

                let results = store.query(&query)?;

                println!("RESULTS:\n{:#?}", results);

                Ok(())
            }

            Command::Codegen {
                inputs,
                target,
                output_dir,
            } => {
                let mut store = lore_store::Store::new();
                for input in inputs {
                    let mut parser = lore_parser::Parser::for_file(input.clone())?;
                    let validator = lore_parser::Validator::new();
                    let ast = validator.validate(parser.parse()?)?;
                    store.add_tree(ast)?;
                }

                let source_set = match target {
                    TargetLang::OCaml => {
                        let emitter = lore_codegen::OCamlEmitter::new();
                        emitter.translate(&store)?
                    }
                    _ => lore_codegen::SourceSet::empty(),
                };

                for source in source_set.sources() {
                    source.write(&output_dir)?;
                }

                Ok(())
            }
        }
    }
}

fn main() -> Result<()> {
    miette::set_hook(Box::new(|_| {
        Box::new(miette::MietteHandlerOpts::new().context_lines(3).build())
    }))?;
    Args::from_args().cmd.run()
}
