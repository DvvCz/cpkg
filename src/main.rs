use clap::{Parser, Subcommand};
use indoc::indoc;
use colored::Colorize;

mod compiler;
mod docgen;

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

	/// Builds the project to the target directory using gcc or clang, if available.
	Build,

	/// Builds and runs the project using gcc or clang, if available.
	Run,

	// Removes compiled programs
	Clean,

	/// Generates documentation using cldoc or doxygen, if available.
	Doc {
		#[arg(short, long)]
		open: bool
	},

	/// Creates a read eval print loop with igcc or bic, if available.
	Repl,

	/// Updates to the latest version of cpkg
	Upgrade
}

fn init_project(proj: &std::path::Path) -> std::io::Result<()> {
	let src = proj.join("src");
	std::fs::create_dir(&src)?;

	let main = src.join("main.c");
	std::fs::write(&main, indoc! {r#"
		#include <stdio.h>

		int main() {
			printf("Hello, world!\n");
			return 0;
		}
	"#})?;

	let tests = proj.join("tests");
	std::fs::create_dir(&tests)?;

	let main_test = tests.join("main.test.c");
	std::fs::write(main_test, indoc!{r#"
		#include <assert.h>

		int main() {
			assert( (1 + 2 == 3) && "C is broken" );
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
			let config = std::path::Path::new("cpkg.toml");
			if !config.exists() {
				anyhow::bail!("No cpkg.toml detected, this doesn't seem to be a valid project.");
			}

			let target = std::path::Path::new("target");
			if !target.exists() {
				std::fs::create_dir(&target)?;
			}

			let tests = std::path::Path::new("tests");

			let out = target.join("test");
			if !out.exists() {
				std::fs::create_dir(&out)?;
			}

			let backend = compiler::try_locate()?;

			let now = std::time::Instant::now();

			let mut compiled_tests = vec![];

			for entry in tests.read_dir()? {
				if let Ok(entry) = entry {
					let path = entry.path();
					let out = out.join(&path.file_stem().unwrap());

					backend.compile(&path, &[], &out)?;
					compiled_tests.push(out);
				}
			}

			for test in &compiled_tests {
				let out = std::process::Command::new(test)
					.output()?;

				if out.status.success() {
					println!("✅ {} {}", test.display(), "passed".green());
				} else {
					eprintln!("{} {}: {}", test.display(), "failed".red(), String::from_utf8_lossy(&out.stderr).trim_end());
				}
			}

			println!("Successfully ran {} tests in {}s.", compiled_tests.len(), now.elapsed().as_secs_f32());
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

			let now = std::time::Instant::now();

			let out = target.join("out");
			let backend = compiler::try_locate()?;
			backend.compile(main, &[], &out)?;

			println!("Successfully built program in {}s", now.elapsed().as_secs_f32());
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

			let b = compiler::try_locate()?;
			b.compile(main, &[], &out)?;

			std::process::Command::new(out)
				.spawn()?;
		},

		Some(Commands::Clean) => {
			let config = std::path::Path::new("cpkg.toml");
			if !config.exists() {
				anyhow::bail!("No cpkg.toml detected, this doesn't seem to be a valid project.");
			}

			let target = std::path::Path::new("target");
			if !target.exists() {
				anyhow::bail!("Failed to clean target directory. Doesn't seem to exist.");
			}

			std::fs::remove_dir_all(target)?;

			println!("Removed target directory.");
		},

		Some(Commands::Doc { open }) => {
			let config = std::path::Path::new("cpkg.toml");
			if !config.exists() {
				anyhow::bail!("No cpkg.toml detected, this doesn't seem to be a valid project.");
			}

			let backend = docgen::try_locate()?;

			let target = std::path::Path::new("target");
			if !target.exists() {
				std::fs::create_dir(target)?;
			}

			let doc = target.join("doc");
			if !doc.exists() {
				std::fs::create_dir(&doc)?;
			}

			let now = std::time::Instant::now();

			let proj = std::path::Path::new("src");
			backend.generate(proj, &doc)?;

			println!("Generated documentation in {}s", now.elapsed().as_secs_f32());

			if *open {
				backend.open(&doc)?;
			}
		},

		Some(Commands::Repl) => {
			anyhow::bail!("Repl is not implemented");
		},

		Some(Commands::Upgrade) => {
			self_update::backends::github::Update::configure()
				.repo_owner("DvvCz")
				.repo_name("cpkg")
				.bin_name("cpkg")
				.show_download_progress(true)
				.current_version(self_update::cargo_crate_version!())
				.build()?
				.update()?;
		}

		None => ()
	}

	Ok(())
}
