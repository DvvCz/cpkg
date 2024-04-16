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

	#[command(about = "Runs the project's main file, a standalone c file or a cpkg.toml script.\x1b[31m")]
	Run {
		path: Option<String>
	},

	#[command(about = "Runs the project's test suite.\n\x1b[33m")]
	Test {
		#[arg(short, long)]
		print: bool
	},

	#[command(about = "Removes compiled programs from the project.\x1b[33m")]
	Clean,

	#[command(about = "Generates documentation for the project using doxygen, if available.\x1b[33m")]
	Doc {
		#[arg(short, long)]
		open: bool
	},

	#[command(about = "Formats the project's code using clang-format, if available.\x1b[33m")]
	Format,

	#[command(about = "Generates a project file for use with other build managers.\n\x1b[36m")]
	Generate {
		#[command(subcommand)]
		kind: GenerateCommand
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
	Remove {
		name: String
	},

	#[command(about = "Installs dependencies from cpkg project.\n\x1b[34m")]
	Install,

	#[command(about = "Creates a REPL with gcc or clang, if available.\x1b[34m")]
	Repl,

	#[command(about = "Updates to the latest version of cpkg.\n\x1b[35m")]
	Upgrade
}

#[derive(Subcommand)]
enum GenerateCommand {
	#[command(about = "Creates a Makefile in the project directory")]
	Make,
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

		[scripts]

		[dependencies]
	"#})?;

	if let Ok(git) = which::which("git") {
		let ignore = proj.join(".gitignore");
		std::fs::write(ignore, indoc!{r#"
			/target
		"#})?;

		std::process::Command::new(git)
			.arg("init")
			.output()?;
	}

	Ok(())
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Config {
	package: ConfigPackage,

	#[serde(default)]
	dependencies: std::collections::HashMap<String, ConfigDependency>,

	#[serde(default)]
	scripts: std::collections::HashMap<String, String>,

	compiler: Option<ConfigCompiler>,
	formatter: Option<ConfigFormatter>,
	docgen: Option<ConfigDocgen>,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct ConfigPackage {
	name: String,
	/// Optional location to output the target binary
	bin: Option<String>
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
enum ConfigDependency {
	Path {
		path: String,
	},
	Git {
		git: String
	}
}

#[derive(serde::Deserialize, serde::Serialize)]
struct ConfigCompiler {
	default: Option<String>,
	flags: Option<Vec<String>>,
	gcc: Option<ConfigGcc>,
	clang: Option<ConfigClang>
}

#[derive(serde::Deserialize, serde::Serialize)]
struct ConfigGcc {
	#[serde(default)]
	flags: Vec<String>
}

#[derive(serde::Deserialize, serde::Serialize)]
struct ConfigClang {
	#[serde(default)]
	flags: Vec<String>
}

#[derive(serde::Deserialize, serde::Serialize)]
struct ConfigFormatter {
	clang_format: toml::Table,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct ConfigClangFormat {
	style: String
}


#[derive(serde::Deserialize, serde::Serialize)]
struct ConfigDocgen {
	doxygen: ConfigDoxygen,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct ConfigDoxygen {
	doxyfile: String,
}

fn get_config() -> anyhow::Result<Config> {
	let config = std::path::Path::new("cpkg.toml");
	if !config.exists() {
		anyhow::bail!("No cpkg.toml detected, this doesn't seem to be a valid project.");
	}

	let config = std::fs::read_to_string(config)?;
	let config = toml::from_str::<Config>(&config)?;

	Ok(config)
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
			let cd = std::env::current_dir()?;
			let config = cd.join("cpkg.toml");

			if config.exists() {
				anyhow::bail!("Cannot initialize project at existing cpkg project.");
			} else {
				init_project(&cd)?;
			}
		},

		Commands::Test { print } => {
			let config = get_config()?;

			let flags = config
				.compiler
				.and_then(|c| c.flags)
				.unwrap_or(vec![]);

			let src = std::path::Path::new("src");

			let target = std::path::Path::new("target");
			if !target.exists() {
				std::fs::create_dir(&target)?;
			}

			let vendor = target.join("vendor");

			let out = target.join("test");
			if !out.exists() {
				std::fs::create_dir(&out)?;
			}

			let backend = compiler::try_locate()?;

			let now = std::time::Instant::now();

			let tests_path = std::path::Path::new("tests");

			let tests = walkdir::WalkDir::new(tests_path)
				.into_iter()
				.chain(walkdir::WalkDir::new(src).into_iter())
				.flat_map(std::convert::identity) // Remove walkdir fails
				.filter(|e| e.file_type().is_file()) // Remove directories
				.filter(|e| e.path().to_string_lossy().ends_with(".test.c")) // Only testing .test.c files
				.map(|e| e.path().to_owned());

			let mut compiled_tests = vec![];

			for path in tests { // Todo: Convert to iterator
				use std::hash::{Hash, Hasher};

				let mut hasher = std::hash::DefaultHasher::new();
				path.hash(&mut hasher);
				let hash = hasher.finish().to_string();

				let out = out.join(hash);
				backend.compile(&path, &[src, tests_path, &vendor], &out, &flags)?;
				compiled_tests.push((path, out));
			}

			for (src, compiled) in &compiled_tests {
				let mut out = std::process::Command::new(compiled);

				let out = if *print {
					out.spawn()?.wait_with_output()?
				} else {
					out.output()?
				};

				if out.status.success() {
					println!("{} {}", " PASSED ".on_bright_green().white(), src.display());
				} else {
					eprintln!("{} {}: {}", " FAILED ".on_bright_red().white(), src.display(), String::from_utf8_lossy(&out.stderr).trim_end());
				}
			}

			println!("Successfully ran {} tests in {}s.", compiled_tests.len(), now.elapsed().as_secs_f32());
		},

		Commands::Build => {
			let config = get_config()?;

			let flags = config
				.compiler
				.and_then(|c| c.flags)
				.unwrap_or(vec![]);

			let src = std::path::Path::new("src");

			let main = src.join("main.c");
			if !main.exists() {
				anyhow::bail!("No entrypoint found (create src/main.c)");
			}

			let target = std::path::Path::new("target");
			if !target.exists() {
				std::fs::create_dir(target)?;
			}

			let vendor = target.join("vendor");

			let now = std::time::Instant::now();

			let out = if let Some(p) = config.package.bin {
				std::path::PathBuf::from(p)
			} else {
				target.join(config.package.name)
			};

			let backend = compiler::try_locate()?;
			backend.compile(&main, &[src, &vendor], &out, &flags)?;

			println!("Successfully built program in {}s", now.elapsed().as_secs_f32());
		},

		Commands::Run { path } => {
			let config = get_config();

			if let Some(path) = path {
				if let Ok(c) = config {
					if let Some(script) = c.scripts.get(path) {
						#[cfg(target_os = "linux")]
						std::process::Command::new("sh")
							.arg("-c")
							.arg(script)
							.spawn()?
							.wait()?;

						#[cfg(target_os = "windows")]
						std::process::Command::new("cmd.exe")
							.arg("/c")
							.arg(script)
							.spawn()?
							.wait()?;

						return Ok(());
					}
				}

				let path = std::path::Path::new(path);

				if !path.is_file() {
					anyhow::bail!("Script not found: {}", path.display());
				}

				let temp = std::env::temp_dir();
				let temp_bin = temp.join("cpkg_run");

				let b = compiler::try_locate()?;
				b.compile(&path, &[ path.parent().unwrap() ], &temp_bin, &[])?;

				std::process::Command::new(&temp_bin)
					.spawn()?;
				
				return Ok(());
			}

			let config = config?;

			let flags = config
				.compiler
				.and_then(|c| c.flags)
				.unwrap_or(vec![]);

			let src = std::path::Path::new("src");

			let main = src.join("main.c");
			if !main.exists() {
				anyhow::bail!("No entrypoint found (create src/main.c)");
			}

			let target = std::path::Path::new("target");
			if !target.exists() {
				std::fs::create_dir(target)?;
			}

			let vendor = target.join("vendor");

			let out = if let Some(p) = config.package.bin {
				std::path::PathBuf::from(p)
			} else {
				target.join(config.package.name)
			};

			let b = compiler::try_locate()?;
			b.compile(&main, &[src, &vendor], &out, &flags)?;

			std::process::Command::new(out)
				.spawn()?;
		},

		Commands::Clean => {
			let _ = get_config()?;

			let target = std::path::Path::new("target");
			if !target.exists() {
				anyhow::bail!("Failed to clean target directory. Doesn't seem to exist.");
			}

			std::fs::remove_dir_all(target)?;

			println!("Removed target directory.");
		},

		Commands::Doc { open } => {
			let _ = get_config()?;
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
			let _ = get_config()?;

			let backend = format::try_locate()?;

			let now = std::time::Instant::now();

			backend.format(std::path::Path::new("src"))?;
			backend.format(std::path::Path::new("tests"))?;

			println!("Formatted code in {}s", now.elapsed().as_secs_f32());
		},


		Commands::Generate { kind } => {
			match kind {
				GenerateCommand::Make => {
					let config = get_config()?;

					let flags = config
						.compiler
						.and_then(|c| c.flags)
						.unwrap_or(vec![]);

					let src = std::path::Path::new("src");

					let main = src.join("main.c");
					if !main.exists() {
						anyhow::bail!("No entrypoint found (create src/main.c)");
					}
		
					let target = std::path::Path::new("target");
					if !target.exists() {
						std::fs::create_dir(target)?;
					}
		
					let now = std::time::Instant::now();

					let out = if let Some(p) = config.package.bin {
						std::path::PathBuf::from(p)
					} else {
						target.join(config.package.name)
					};
		
					let backend = compiler::try_locate()?;

					let cmd = backend.get_compile_command(&main, &[], &out, &flags);

					let cc = cmd.get_program();

					let make = indoc::formatdoc! {"
						CC = {cc:?}

						main:
							{cmd:?}
					"};

					std::fs::write("Makefile", make)?;

					println!("Generated Makefile in {}s", now.elapsed().as_secs_f32());
				}
			}
		},

		Commands::Add { name, git, path } => {
			let mut config = get_config()?;

			if git.is_some() && path.is_some() {
				anyhow::bail!("Cannot be both git and path dependencies");
			}

			let dep = if let Some(git) = git {
				ConfigDependency::Git { git: git.clone() }
			} else if let Some(path) = path {
				ConfigDependency::Path { path: path.clone() }
			} else {
				anyhow::bail!("Must provide either --git or --path, for now.");
			};

			config.dependencies.insert(name.clone(), dep);

			std::fs::write("cpkg.toml", toml::to_string(&config)?)?;

			println!("Added dependency to {}.", "cpkg.toml".yellow())
		},

		Commands::Remove { name } => {
			let mut config = get_config()?;

			if config.dependencies.remove(name).is_none() {
				anyhow::bail!("Could not find dependency {} to remove.", name.yellow());
			}

			std::fs::write("cpkg.toml", toml::to_string(&config)?)?;
			println!("Removed {} from {}.", name.yellow(), "cpkg.toml".yellow());
		},

		Commands::Install => {
			let config = get_config()?;

			let target = std::path::Path::new("target");

			if !target.exists() {
				std::fs::create_dir(target)?;
			}

			let build = target.join("vendor");

			if !build.exists() {
				std::fs::create_dir(&build)?;
			}

			// Create include path for clangd, if present
			// TODO: Replace with more robust compile_commands.json
			if which::which("clangd").is_ok() {
				let clangd = std::path::Path::new("compile_flags.txt");
				if !clangd.exists() {
					std::fs::write(clangd, "-I./target/vendor")?;
				}
			}

			let git_cmd = which::which("git")
				.map_err(|_| anyhow::anyhow!("You need git installed to use cpkg install, for now."))?;

			let now = std::time::Instant::now();

			for (name, dep) in &config.dependencies {
				let dep_path = build.join(name);

				if dep_path.exists() {
					continue;
				}

				match dep {
					ConfigDependency::Git { git } => {
						std::process::Command::new(&git_cmd)
							.arg("clone")
							.arg(git)
							.arg(dep_path)
							.spawn()?;
					},

					ConfigDependency::Path { path } => {
						std::fs::hard_link(path, dep_path)?;
					}
				}
			}

			println!("Installed {} dependencies in {} seconds.", config.dependencies.len().to_string().yellow(), now.elapsed().as_secs_f32().to_string().yellow());
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
