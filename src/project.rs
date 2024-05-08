use crate::ConfigDependency;

/// A `cpkg` project.
/// This is defined as a directory containing a cpkg.toml file inside of it.
pub struct Project<'a> {
	path: &'a std::path::Path,
	config: crate::Config,
}

impl<'a> Project<'a> {
	/// Folder containing source files
	const SRC: &'static str = "src";

	/// Folder containing output
	const TARGET: &'static str = "target";

	/// Folder containing libraries inside of output/target folder
	const VENDOR: &'static str = "vendor";

	/// Folder containing test files
	const TESTS: &'static str = "tests";

	/*
		Paths
	*/

	pub fn get_or_mkdir(path: std::path::PathBuf) -> anyhow::Result<std::path::PathBuf> {
		if !path.is_dir() {
			std::fs::create_dir(&path)?;
		}

		Ok(path)
	}

	pub fn src(&self) -> std::path::PathBuf {
		self.path.join(Self::SRC)
	}

	pub fn target(&self) -> std::path::PathBuf {
		self.path.join(Self::TARGET)
	}

	pub fn vendor(&self) -> std::path::PathBuf {
		self.target().join(Self::VENDOR)
	}

	pub fn tests(&self) -> std::path::PathBuf {
		self.path.join(Self::TESTS)
	}

	/*
		Instantiation
	*/

	pub fn create(path: &'a std::path::Path) -> anyhow::Result<Self> {
		if path.exists() {
			anyhow::bail!(
				"Failed to create project at {}: path already exists",
				path.display()
			);
		}

		std::fs::create_dir(path)?;

		Self::init(path)
	}

	pub fn init(path: &'a std::path::Path) -> anyhow::Result<Self> {
		if !path.is_dir() {
			anyhow::bail!(
				"Failed to initialize project at {}: not a directory.",
				path.display()
			);
		}

		if path.join("cpkg.toml").exists() {
			anyhow::bail!("Cannot initialize project at existing cpkg project.");
		}

		let src = Self::get_or_mkdir(path.join(Self::SRC))?;

		std::fs::write(
			src.join("main.c"),
			indoc::indoc! {r#"
				#include <stdio.h>

				int main() {
					printf("Hello, world!\n");
					return 0;
				}
			"#},
		)?;

		std::fs::write(
			src.join("main.test.c"),
			indoc::indoc! {r#"
				#include <assert.h>

				int main() {
					assert( (1 + 2 == 3) && "C is broken" );
				}
			"#},
		)?;

		let config = crate::Config {
			package: crate::ConfigPackage {
				name: String::from(path.file_name().unwrap().to_string_lossy()),
				bin: None,
			},

			dependencies: Default::default(),
			scripts: Default::default(),

			compiler: None,
			formatter: None,
			docgen: None,
		};

		std::fs::write(path.join("cpkg.toml"), toml::to_string(&config)?)?;

		if let Ok(git) = which::which("git") {
			std::fs::write(
				path.join(".gitignore"),
				indoc::indoc! {r#"
					/target
				"#},
			)?;

			std::process::Command::new(git)
				.arg("init")
				.current_dir(path)
				.output()?;
		}

		let p = Project { path, config };

		Ok(p)
	}

	pub fn open(path: &'a std::path::Path) -> anyhow::Result<Self> {
		if !path.is_dir() {
			anyhow::bail!(
				"Failed to open project {}: not a directory.",
				path.display()
			);
		}

		let config = path.join("cpkg.toml");
		if !config.is_file() {
			anyhow::bail!("No cpkg.toml detected, this doesn't seem to be a valid project.");
		}

		let config = std::fs::read_to_string(config)?;
		let config = toml::from_str::<crate::Config>(&config)?;

		Ok(Project { path, config })
	}

	/*
		Configuration
	*/

	pub fn config(&self) -> &crate::Config {
		&self.config
	}

	pub fn name(&self) -> &String {
		&self.config.package.name
	}

	/// Saves the config to cpkg.toml  
	/// Shouldn't need to use this, as [Self::with_config] calls this for you.
	pub fn save_config(&self) -> anyhow::Result<()> {
		std::fs::write(
			self.path.join("cpkg.toml"),
			toml::to_string_pretty(&self.config)?,
		)?;
		Ok(())
	}

	/// Allows you to mutate the config inside of a callback  
	/// The config will be saved to the file afterwards, ensuring no desync.
	pub fn with_config<T>(
		&mut self,
		cb: impl FnOnce(&mut crate::Config) -> T,
	) -> anyhow::Result<T> {
		let r = cb(&mut self.config);
		self.save_config()?;
		Ok(r)
	}

	/*
		Dependencies
	*/

	#[must_use = "Ensure successfully added dependency"]
	pub fn add_dep(&mut self, name: String, dep: crate::ConfigDependency) -> anyhow::Result<()> {
		self.with_config(|conf| {
			conf.dependencies.insert(name, dep);
		})
	}

	#[must_use = "Ensure successfully removed dependency"]
	pub fn remove_dep(&mut self, name: impl AsRef<str>) -> anyhow::Result<crate::ConfigDependency> {
		let name = name.as_ref();

		// Convert Result<Option<T>> to Result<T> for case that the dependency didn't exist.
		// Might change this to just return Result<Option<T>> in the future.
		let r = self.with_config(|conf| conf.dependencies.remove(name));

		r.and_then(|o| {
			o.ok_or(anyhow::anyhow!(
				"Could not find dependency {} to remove",
				name
			))
		})
	}

	pub fn install_deps(&self) -> anyhow::Result<()> {
		let target = Self::get_or_mkdir(self.target())?;
		let build = Self::get_or_mkdir(target.join("vendor"))?;

		/*
			Create compile_flags.txt for intellisense
			TODO: Generate more robust compile_commands.json instead
		*/
		if which::which("clangd").is_ok() {
			let clangd = self.path.join("compile_flags.txt");
			if !clangd.exists() {
				std::fs::write(clangd, "-I./target/vendor")?;
			}
		}

		let has_git = which::which("git").is_ok();

		let needs_git = self
			.config
			.dependencies
			.iter()
			.find(|dep| matches!(dep.1, ConfigDependency::Git { .. }))
			.map(|dep| dep.0);

		if let Some(dep) = needs_git {
			anyhow::ensure!(has_git, "Cannot install dependency '{dep}' without git.");
		}

		for (name, dep) in &self.config.dependencies {
			let install_dir = build.join(name);

			/* Already installed */
			if install_dir.exists() {
				continue;
			}

			match dep {
				ConfigDependency::Path { path } => {
					std::fs::hard_link(path, install_dir)?;
				}
				ConfigDependency::Git { git } => {
					std::process::Command::new("git")
						.arg("clone")
						.arg(git)
						.arg(install_dir)
						.spawn()?;
				}
			}
		}

		Ok(())
	}

	/*
		File Iterators
	*/

	pub fn test_files(&self) -> impl std::iter::Iterator<Item = std::path::PathBuf> {
		let inline_tests = walkdir::WalkDir::new(self.src())
			.into_iter()
			.flat_map(std::convert::identity)
			.filter(|e| e.path().is_file())
			.filter(|e| e.path().to_string_lossy().ends_with(".test.c"))
			.map(|e| e.path().to_owned());

		let explicit_tests = walkdir::WalkDir::new(self.tests())
			.into_iter()
			.flat_map(std::convert::identity)
			.filter(|e| e.path().is_file())
			.filter(|e| e.path().to_string_lossy().ends_with(".c"))
			.map(|e| e.path().to_owned());

		inline_tests.chain(explicit_tests)
	}

	fn source_files(
		&self,
		dir: &std::path::Path,
	) -> impl std::iter::Iterator<Item = std::path::PathBuf> {
		walkdir::WalkDir::new(dir)
			.into_iter()
			.flat_map(std::convert::identity)
			.filter(|e| e.path().is_file())
			.filter(|e| e.path().to_string_lossy().ends_with(".c"))
			.filter(|e| !e.path().to_string_lossy().ends_with(".test.c"))
			.map(|e| e.path().to_owned())
	}

	/*
		Building
	*/

	pub fn build_flags(
		&self,
		_backend: &dyn crate::compiler::Compiler,
	) -> std::borrow::Cow<Vec<String>> {
		/* TODO: Support backend-specific flags */
		if let Some(provided) = self.config.compiler.as_ref() {
			if let Some(ref flags) = provided.flags {
				return std::borrow::Cow::Borrowed(&flags);
			}
		}

		std::borrow::Cow::Owned(vec![])
	}

	/// Returns PathBuf to desired executable location
	pub fn build_out(&self) -> std::path::PathBuf {
		if let Some(ref bin) = self.config.package.bin {
			std::path::PathBuf::from(bin)
		} else {
			self.target().join(&self.config.package.name)
		}
	}

	/// Builds the project at provided entrypoint, returning executable path.
	#[must_use = "Ensure actually built correctly"]
	pub fn build(
		&self,
		backend: &dyn crate::compiler::Compiler,
		main: Option<&String>,
	) -> anyhow::Result<std::path::PathBuf> {
		let src = self.src();

		/* Provided a specific entrypoint */
		if let Some(main) = main {
			todo!();
		} else {
			/* Traditional main entrypoint */

			let main = src.join("main.c");
			if main.exists() {
				let source_files = self.source_files(&src).collect::<Vec<_>>();
				let out = self.build_out();
				let flags = self.build_flags(backend);

				backend.compile(&source_files, &[&self.vendor(), &src], &out, &flags)?;

				Ok(out)
			} else {
				anyhow::bail!("Couldn't find main.c to build!");
			}
		}
	}

	/*
		Tests
	*/

	pub fn compile_tests(
		&self,
		backend: &dyn crate::compiler::Compiler,
	) -> anyhow::Result<Vec<(std::path::PathBuf, std::path::PathBuf)>> {
		let src = self.src();

		let mut c_files = self
			.source_files(&src)
			.take_while(|f| f.file_name().unwrap() != "main.c")
			.collect::<Vec<_>>();

		let out_dir = Self::get_or_mkdir(Self::get_or_mkdir(self.target())?.join("test"))?;
		let flags = self.build_flags(backend);

		let mut compiled = vec![];

		for test in self.test_files() {
			let hash = {
				use std::hash::{Hash, Hasher};

				let mut hasher = std::hash::DefaultHasher::new();
				test.hash(&mut hasher);
				hasher.finish().to_string()
			};

			let out_path = out_dir.join(&hash);

			c_files.push(test);
			backend.compile(&c_files, &[&src], &out_path, &flags)?;
			let test = c_files.pop().unwrap();

			compiled.push((test, out_path));
		}

		Ok(compiled)
	}

	pub fn run_tests(
		&self,
		backend: &dyn crate::compiler::Compiler,
		print: bool,
	) -> anyhow::Result<Vec<(bool, std::path::PathBuf, Option<String>)>> {
		let compiled = self.compile_tests(backend)?;

		let mut results = Vec::with_capacity(compiled.len());

		for (src, compiled) in compiled {
			let mut out = std::process::Command::new(&compiled);

			let out = if print {
				out.spawn()?.wait_with_output()?
			} else {
				out.output()?
			};

			if out.status.success() {
				results.push((true, src, None));
			} else {
				results.push((false, src, Some(String::from_utf8(out.stderr)?)))
			}
		}

		Ok(results)
	}
}