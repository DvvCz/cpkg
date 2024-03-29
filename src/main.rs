use clap::{Parser, Subcommand};
use indoc::indoc;
use colored::Colorize;

mod compiler;
mod docgen;
mod format;

/// Dead simple C package manager
#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(arg_required_else_help = true)]
struct Cli {
	#[command(subcommand)]
	command: Commands,
}

#[derive(Subcommand)]
enum Commands {
	#[command(about = "Creates a template project at a given directory.")]
	New {
		/// Name of folder to create new project inside of.
		name: String
	},
	#[command(about = "Initializes a template project at the cwd.\n\x1b[31m")]
	Init,

	#[command(about = "Builds the project to the target directory using gcc or clang, if available.\x1b[31m")]
	Build,

	#[command(about = "Runs the project's main file, or a standalone c file.\x1b[31m")]
	Run {
		path: Option<String>
	},

	#[command(about = "Runs the project's test suite.\n\x1b[33m")]
	Test,

	#[command(about = "Removes compiled programs from the project.\x1b[33m")]
	Clean,

	#[command(about = "Generates documentation for the project using doxygen, if available.\x1b[33m")]
	Doc {
		#[arg(short, long)]
		open: bool
	},

	#[command(about = "Formats the project's code using clang-format, if available.\n\x1b[36m")]
	Format,

	#[command(about = "Creates a REPL with gcc or clang, if available.\x1b[36m")]
	Repl,

	#[command(about = "Updates to the latest version of cpkg.\n\x1b[35m")]
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

#[derive(serde::Deserialize)]
struct Config {
	package: ConfigPackage,

	compiler: Option<ConfigCompiler>,
	formatter: Option<ConfigFormatter>,
	docgen: Option<ConfigDocgen>,
}

#[derive(serde::Deserialize)]
struct ConfigPackage {
	name: String,
}

#[derive(serde::Deserialize)]
struct ConfigCompiler {
	default: Option<String>,
	flags: Option<Vec<String>>,

	gcc: Option<ConfigGcc>,
	clang: Option<ConfigClang>
}

#[derive(serde::Deserialize)]
struct ConfigGcc {
	flags: Option<Vec<String>>
}

#[derive(serde::Deserialize)]
struct ConfigClang {
	flags: Option<Vec<String>>
}

#[derive(serde::Deserialize)]
struct ConfigFormatter {
	clang_format: toml::Table,
}

#[derive(serde::Deserialize)]
struct ConfigClangFormat {
	style: String
}


#[derive(serde::Deserialize)]
struct ConfigDocgen {
	doxygen: ConfigDoxygen,
}

#[derive(serde::Deserialize)]
struct ConfigDoxygen {
	doxyfile: String,
}

fn main() -> anyhow::Result<()> {
	let args = Cli::parse();

	match &args.command {
		Commands::New { name } => {
			let p = std::path::Path::new(name);

			if p.exists() {
				anyhow::bail!("Cannot create new project at already existing '{name}'");
			} else {
				std::fs::create_dir(&p)?;
				init_project(&p)?;
			}
		},

		Commands::Init => {
			let p = std::env::current_dir()?;
			if p.read_dir()?.next().is_none() {
				init_project(&p)?;
			} else {
				anyhow::bail!("Cannot initialize project at non-empty directory");
			}
		},

		Commands::Test => {
			let config = std::path::Path::new("cpkg.toml");
			if !config.exists() {
				anyhow::bail!("No cpkg.toml detected, this doesn't seem to be a valid project.");
			}

			let config = std::fs::read_to_string(config)?;
			let config = toml::from_str::<Config>(&config)?;

			let flags = config
				.compiler
				.and_then(|c| c.flags)
				.unwrap_or(vec![]);

			let target = std::path::Path::new("target");
			if !target.exists() {
				std::fs::create_dir(&target)?;
			}

			let out = target.join("test");
			if !out.exists() {
				std::fs::create_dir(&out)?;
			}

			let backend = compiler::try_locate()?;

			let now = std::time::Instant::now();

			let src = std::path::Path::new("src");

			let tests = std::path::Path::new("tests");
			let tests = walkdir::WalkDir::new(tests)
				.into_iter()
				.chain(walkdir::WalkDir::new(src).into_iter())
				.flat_map(std::convert::identity) // Remove walkdir fails
				.filter(|e| e.file_type().is_file()) // Remove directories
				.filter(|e| e.path().to_string_lossy().ends_with(".test.c")) // Only testing .test.c files
				.map(|e| e.path().to_owned());

			let mut compiled_tests = vec![];

			for test_path in tests { // Todo: Convert to iterator
				use std::hash::{Hash, Hasher};

				let mut hasher = std::hash::DefaultHasher::new();
				test_path.hash(&mut hasher);
				let hash = hasher.finish().to_string();

				let out = out.join(hash);
				backend.compile(&test_path, &[], &out, &flags)?;
				compiled_tests.push((test_path, out));
			}

			for (src, compiled) in &compiled_tests {
				let out = std::process::Command::new(compiled)
					.output()?;

				if out.status.success() {
					println!("{} {}", " PASSED ".on_bright_green().white(), src.display());
				} else {
					eprintln!("{} {}: {}", " FAILED ".on_bright_red().white(), src.display(), String::from_utf8_lossy(&out.stderr).trim_end());
				}
			}

			println!("Successfully ran {} tests in {}s.", compiled_tests.len(), now.elapsed().as_secs_f32());
		},

		Commands::Build => {
			let config = std::path::Path::new("cpkg.toml");
			if !config.exists() {
				anyhow::bail!("No cpkg.toml detected, this doesn't seem to be a valid project.");
			}

			let config = std::fs::read_to_string(config)?;
			let config = toml::from_str::<Config>(&config)?;

			let flags = config
				.compiler
				.and_then(|c| c.flags)
				.unwrap_or(vec![]);

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
			backend.compile(main, &[], &out, &flags)?;

			println!("Successfully built program in {}s", now.elapsed().as_secs_f32());
		},

		Commands::Run { path } => {
			if let Some(path) = path {
				let path = std::path::Path::new(path);

				if !path.exists() {
					anyhow::bail!("File does not exist.");
				}

				let temp = std::env::temp_dir();
				let temp_bin = temp.join("cpkg_run");

				let b = compiler::try_locate()?;
				b.compile(&path, &[], &temp_bin, &[])?;

				std::process::Command::new(&temp_bin)
					.spawn()?;
				
				return Ok(());
			}

			let config = std::path::Path::new("cpkg.toml");
			if !config.exists() {
				anyhow::bail!("No cpkg.toml detected, this doesn't seem to be a valid project.");
			}

			let config = std::fs::read_to_string(config)?;
			let config = toml::from_str::<Config>(&config)?;

			let flags = config
				.compiler
				.and_then(|c| c.flags)
				.unwrap_or(vec![]);

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
			b.compile(main, &[], &out, &flags)?;

			std::process::Command::new(out)
				.spawn()?;
		},

		Commands::Clean => {
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

		Commands::Doc { open } => {
			let config = std::path::Path::new("cpkg.toml");
			if !config.exists() {
				anyhow::bail!("No cpkg.toml detected, this doesn't seem to be a valid project.");
			}

			let config = std::fs::read_to_string(config)?;
			let config = toml::from_str::<Config>(&config)?;

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

		Commands::Format => {
			let config = std::path::Path::new("cpkg.toml");
			if !config.exists() {
				anyhow::bail!("No cpkg.toml detected, this doesn't seem to be a valid project.");
			}

			let config = std::fs::read_to_string(config)?;
			let config = toml::from_str::<Config>(&config)?;

			let backend = format::try_locate()?;

			let now = std::time::Instant::now();

			backend.format(std::path::Path::new("src"))?;
			backend.format(std::path::Path::new("tests"))?;

			println!("Formatted code in {}s", now.elapsed().as_secs_f32());
		},

		Commands::Repl => {
			use std::io::Write;

			println!("{}", "Please note that the repl is very basic and experimental.\nYour code will run entirely each line.".yellow());

			let backend = compiler::try_locate()?;

			let temp = std::env::temp_dir();
			let temp_repl = temp.join("cpkg_repl.c");
			let temp_bin = temp.join("cpkg_repl");

			let mut stdout = std::io::stdout().lock();
			let mut buffer = String::new();

			let mut editor = rustyline::DefaultEditor::new()?;

			loop {
				let temp = editor.readline("> ")?;
				editor.add_history_entry(&temp)?;

				let total = [buffer.clone(), temp].join("");

				std::fs::write(&temp_repl, indoc::formatdoc!(r#"
					int main() {{
						{total}
						return 0;
					}}
				"#))?;

				match backend.compile(&temp_repl, &[], &temp_bin, &["-w".to_owned()]) {
					Ok(_) => {
						let mut out = std::process::Command::new(&temp_bin)
							.output()?;

						if out.status.success() {
							buffer = total; // Only update entire code if ran successfully

							if out.stdout.ends_with(b"\n") {
								stdout.write(&out.stdout)?;
							} else { // If no newline present, add one to the end to avoid breaking rendering
								out.stdout.push(b'\n');
								stdout.write(&out.stdout)?;
							}
						} else {
							stdout.write(b"Failed to run: ")?;
							stdout.write(&out.stderr)?;
							stdout.write(b"\n")?;
						}

						stdout.flush()?;
					},
					Err(_) => ()
				}
			}
		},

		Commands::Upgrade => {
			self_update::backends::github::Update::configure()
				.repo_owner("DvvCz")
				.repo_name("cpkg")
				.bin_name("cpkg")
				.show_download_progress(true)
				.current_version(self_update::cargo_crate_version!())
				.build()?
				.update()?;
		}
	}

	Ok(())
}
