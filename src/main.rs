use colored::Colorize;

mod cli;

mod components;
use components::*;

mod project;
use project::*;

mod config;
use config::*;

fn main() -> anyhow::Result<()> {
	let args: cli::Cli = clap::Parser::parse();
	let cd = std::env::current_dir()?;

	match &args.command {
		cli::Commands::New { name } => {
			Project::create(name.as_ref())?;
		}

		cli::Commands::Init => {
			Project::init(&cd)?;
		}

		cli::Commands::Test { print } => {
			let proj = Project::open(&cd)?;

			let now = std::time::Instant::now();

			let results = proj.run_tests(compiler::try_locate(Some(&proj))?.as_ref(), *print)?;

			for (passed, path, err) in &results {
				if *passed {
					println!(
						"{} {}",
						" PASSED ".on_bright_green().white(),
						path.display()
					);
				} else {
					eprintln!(
						"{} {}: {}",
						" FAILED ".on_bright_red().white(),
						path.display(),
						err.clone().unwrap().trim_end()
					);
				}
			}

			println!(
				"Successfully ran {} tests in {}s.",
				results.len(),
				now.elapsed().as_secs_f32()
			);
		}

		cli::Commands::Build { bin } => {
			let proj = Project::open(&cd)?;

			let now = std::time::Instant::now();

			proj.build(compiler::try_locate(Some(&proj))?.as_ref(), bin)?;

			println!(
				"Successfully built program(s) in {}s",
				now.elapsed().as_secs_f32()
			);
		}

		cli::Commands::Run { path, bin } => {
			let proj = Project::open(&cd);

			if let Some(script) = path {
				if let Ok(proj) = proj {
					let c = proj.config();

					if let Some(script) = c.scripts.get(script) {
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

				let script = std::path::PathBuf::from(script);
				if script.exists() {
					let temp = tempfile::Builder::new()
						.prefix("cpkg-repl")
						.tempfile()?
						.into_temp_path();

					compiler::try_locate(None)?.compile(&[script], &[], &temp, &[])?;

					std::process::Command::new(&temp).spawn()?;

					return Ok(());
				} else {
					return Err(anyhow::anyhow!("Script not found: {}", script.display()));
				}
			}

			let proj = proj?;
			let out = proj.build(compiler::try_locate(Some(&proj))?.as_ref(), bin)?;

			std::process::Command::new(out).spawn()?;
		}

		cli::Commands::Clean => {
			let proj = Project::open(&cd)?;

			let target = proj.target();

			if !target.exists() {
				anyhow::bail!("Failed to clean target directory. Doesn't seem to exist.");
			}

			std::fs::remove_dir_all(target)?;

			println!("Removed target directory.");
		}

		cli::Commands::Doc { open } => {
			let proj = Project::open(&cd)?;
			let backend = docgen::try_locate(&proj)?;

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

			println!(
				"Generated documentation in {}s",
				now.elapsed().as_secs_f32()
			);

			if *open {
				backend.open(&doc)?;
			}
		}

		cli::Commands::Format => {
			let p = Project::open(&cd)?;

			let backend = format::try_locate(&p)?;

			let now = std::time::Instant::now();

			backend.format(&p)?;

			println!("Formatted code in {}s", now.elapsed().as_secs_f32());
		}

		cli::Commands::Generate { kind } => match kind {
			cli::GenerateCommand::Make => {
				let proj = Project::open(&cd)?;

				let backend = compiler::try_locate(Some(&proj))?;
				let make = backend.makefile(&proj);
				std::fs::write("Makefile", make)?;

				println!("Generated Makefile.");
			}
		},

		cli::Commands::Add { name, git, path } => {
			let mut project = Project::open(&cd)?;

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

			project.add_dep(name.to_owned(), dep)?;

			println!("Added dependency to {}.", "cpkg.toml".yellow())
		}

		cli::Commands::Remove { name } => {
			let mut proj = Project::open(&cd)?;

			proj.remove_dep(name)?;

			println!("Removed {} from {}.", name.yellow(), "cpkg.toml".yellow());
		}

		cli::Commands::Install => {
			let proj = Project::open(&cd)?;

			let now = std::time::Instant::now();

			proj.install_deps()?;

			println!(
				"Installed {} dependencies in {} seconds.",
				proj.config().dependencies.len().to_string().yellow(),
				now.elapsed().as_secs_f32().to_string().yellow()
			);
		}

		cli::Commands::Repl => {
			use std::io::Write;

			println!("{}", "Please note that the repl is very basic and experimental.\nYour code will run entirely each line.".yellow());

			let backend = compiler::try_locate(None)?;

			let temp_repl = tempfile::Builder::new()
				.prefix("cpkg-repl")
				.suffix(".c")
				.tempfile()?
				.into_temp_path();

			let temp_bin = tempfile::Builder::new()
				.tempfile()?
				.into_temp_path();

			let mut stdout = std::io::stdout().lock();
			let mut buffer = String::new();

			let mut editor = rustyline::DefaultEditor::new()?;

			loop {
				let temp = editor.readline("> ")?;
				editor.add_history_entry(&temp)?;

				let total = [buffer.clone(), temp].join("");

				#[rustfmt::skip]
				std::fs::write(
					&temp_repl,
					indoc::formatdoc!(r#"
						int main() {{
							{total}
							return 0;
						}}
					"#)
				)?;

				match backend.compile(&[temp_repl.to_path_buf()], &[], &temp_bin, &["-w".to_owned(), "-fdiagnostics-color=always".to_owned()]) {
					Ok(_) => {
						let mut out = std::process::Command::new(&temp_bin).output()?;

						if out.status.success() {
							buffer = total; // Only update entire code if ran successfully

							if out.stdout.ends_with(b"\n") {
								stdout.write(&out.stdout)?;
							} else {
								// If no newline present, add one to the end to avoid breaking rendering
								out.stdout.push(b'\n');
								stdout.write(&out.stdout)?;
							}
						} else {
							stdout.write(b"Failed to run: ")?;
							stdout.write(&out.stderr)?;
							stdout.write(b"\n")?;
						}

						stdout.flush()?;
					}
					Err(e) => {
						print!("{e}");
					},
				}
			}
		}

		cli::Commands::Upgrade => {
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
