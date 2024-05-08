use clap::{Parser, Subcommand};

/// Dead simple C package manager
#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(arg_required_else_help = true)]
pub struct Cli {
	#[command(subcommand)]
	pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
	#[command(about = "Creates a template project at a given directory.")]
	New {
		/// Name of folder to create new project inside of.
		name: String,
	},
	#[command(about = "Initializes a template project at the cwd.\n\x1b[31m")]
	Init,

	#[command(
		about = "Builds the project to the target directory using gcc or clang, if available.\x1b[31m"
	)]
	Build {
		#[arg(long)]
		bin: Option<String>,
	},

	#[command(
		about = "Runs the project's main file, a standalone c file or a cpkg.toml script.\x1b[31m"
	)]
	Run { path: Option<String> },

	#[command(about = "Runs the project's test suite.\n\x1b[33m")]
	Test {
		#[arg(short, long)]
		print: bool,
	},

	#[command(about = "Removes compiled programs from the project.\x1b[33m")]
	Clean,

	#[command(
		about = "Generates documentation for the project using doxygen, if available.\x1b[33m"
	)]
	Doc {
		#[arg(short, long)]
		open: bool,
	},

	#[command(about = "Formats the project's code using clang-format, if available.\x1b[33m")]
	Format,

	#[command(about = "Generates a project file for use with other build managers.\n\x1b[36m")]
	Generate {
		#[command(subcommand)]
		kind: GenerateCommand,
	},

	#[command(about = "Adds a dependency to cpkg.toml.\x1b[36m")]
	Add {
		name: String,

		/// Adds the dependency as a git dependency.
		#[arg(long)]
		git: Option<String>,

		/// Adds the dependency, as a local file path to symlink.
		#[arg(long)]
		path: Option<String>,
	},

	#[command(about = "Removes a dependency from cpkg.toml and deletes it.\x1b[36m")]
	Remove { name: String },

	#[command(about = "Installs dependencies from cpkg project.\n\x1b[34m")]
	Install,

	#[command(about = "Creates a REPL with gcc or clang, if available.\x1b[34m")]
	Repl,

	#[command(about = "Updates to the latest version of cpkg.\n\x1b[35m")]
	Upgrade,
}

#[derive(Subcommand)]
pub enum GenerateCommand {
	#[command(about = "Creates a Makefile in the project directory")]
	Make,
}
