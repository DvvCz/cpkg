use clap::{Parser, Subcommand};
use indoc::indoc;

mod backend;

/// Dead simple C package manager
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
	#[command(subcommand)]
	command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
	/// Creates a template project at a given directory
	New {
		/// Name of folder to create new project inside of.
		name: String
	},
	/// Initializes a template project at the cwd
	Init,

	/// Runs the project's test suite
	Test,

	/// Builds the project to the target directory
	Build,

	/// Builds and runs the project
	Run,
}

fn init_project(proj: &std::path::Path) -> std::io::Result<()> {
	let src = proj.join("src");
	std::fs::create_dir(&src)?;

	let main = src.join("main.c");
	std::fs::write(&main, indoc! {r#"
		int adder(int a, int b) {
			return a + b;
		}

		int main() {
			printf("Hello, world!\n");
			return 0;
		}
	"#})?;

	let tests = proj.join("tests");
	std::fs::create_dir(&tests)?;

	let main_test = tests.join("main.test.c");
	std::fs::write(main_test, indoc!{r#"
		int main() {
			int result = adder(5, 10);
			assert(result == 15, "5 + 10 = 15");
		}
	"#})?;

	let config = proj.join("cpkg.toml");
	let name = proj.file_name().unwrap().to_string_lossy();
	std::fs::write(config, indoc::formatdoc! {r#"
		[package]
		name = "{name}"

		[dependencies]
	"#})?;

	if which::which("git").is_ok() {
		let ignore = proj.join(".gitignore");
		std::fs::write(ignore, indoc!{r#"
			/target
		"#})?;
	}

	Ok(())
}

fn main() -> anyhow::Result<()> {
	let args = Cli::parse();

	match &args.command {
		Some(Commands::New { name }) => {
			let p = std::path::Path::new(name);

			if p.exists() {
				anyhow::bail!("Cannot create new project at already existing '{name}'");
			} else {
				std::fs::create_dir(&p)?;
				init_project(&p)?;
			}
		},

		Some(Commands::Init) => {
			let p = std::env::current_dir()?;
			if p.read_dir()?.next().is_none() {
				init_project(&p)?;
			} else {
				anyhow::bail!("Cannot initialize project at non-empty directory");
			}
		},

		Some(Commands::Test) => {
			anyhow::bail!("Unimplemented: test");
		},

		Some(Commands::Build) => {
			let config = std::path::Path::new("cpkg.toml");
			if !config.exists() {
				anyhow::bail!("No cpkg.toml detected, this doesn't seem to be a valid project.");
			}

			let main = std::path::Path::new("src/main.c");
			if !main.exists() {
				anyhow::bail!("No entrypoint found (create src/main.c)");
			}

			let target = std::path::Path::new("target");
			if !target.exists() {
				std::fs::create_dir(target)?;
			}

			let out = target.join("out");

			let b = backend::find_backend()?;
			b.compile(main, &[], &out)?;
		},

		Some(Commands::Run) => {
			let config = std::path::Path::new("cpkg.toml");
			if !config.exists() {
				anyhow::bail!("No cpkg.toml detected, this doesn't seem to be a valid project.");
			}

			let main = std::path::Path::new("src/main.c");
			if !main.exists() {
				anyhow::bail!("No entrypoint found (create src/main.c)");
			}

			let target = std::path::Path::new("target");
			if !target.exists() {
				std::fs::create_dir(target)?;
			}

			let out = target.join("out");

			let b = backend::find_backend()?;
			b.compile(main, &[], &out)?;

			std::process::Command::new(out)
				.spawn()?;
		},

		None => ()
	}

	Ok(())
}
