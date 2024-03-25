pub trait Docgen {
	fn generate(&self, src: &std::path::Path, to: &std::path::Path) -> anyhow::Result<()>;
	fn open(&self, to: &std::path::Path) -> anyhow::Result<()>;
}

pub struct Doxygen;

impl Docgen for Doxygen {
	fn generate(&self, _src: &std::path::Path, to: &std::path::Path) -> anyhow::Result<()> {
		let config = to.join("Doxyfile");

		std::fs::write(&config, indoc::formatdoc! {"
			INPUT=../../src
			OUTPUT_DIRECTORY=.
		"})?;

		let out = std::process::Command::new("doxygen")
			.current_dir(to)
			.output()?;

		if !out.status.success() {
			anyhow::bail!("Failed to generate documentation: {}", String::from_utf8_lossy(&out.stderr));
		}

		Ok(())
	}

	fn open(&self, to: &std::path::Path) -> anyhow::Result<()> {
		let index = to.join("html/index.html");
		start_program(&index)
	}
}

pub struct Cldoc;

impl Docgen for Cldoc {
	fn generate(&self, src: &std::path::Path, to: &std::path::Path) -> anyhow::Result<()> {
		let out = std::process::Command::new("cldoc")
			.arg("generate")
			.arg("--")
			.arg("--output")
			.arg(to)
			.arg(src)
			.output()?;

		if !out.status.success() {
			anyhow::bail!("Failed to generate documentation: {}", String::from_utf8_lossy(&out.stderr));
		}

		Ok(())
	}

	fn open(&self, _to: &std::path::Path) -> anyhow::Result<()> {
		anyhow::bail!("Open is not yet implemented for cldoc. Sorry!")
	}
}

#[cfg(target_os = "linux")]
fn start_program(p: &std::path::Path) -> anyhow::Result<()> {
	std::process::Command::new("xdg-open")
		.arg(p)
		.output()?;

	Ok(())
}

#[cfg(target_os = "windows")]
fn start_program(p: &std::path::Path) -> anyhow::Result<()> {
	// TODO: Test on windows
	std::process::Command::new("cmd")
		.arg("-C")
		.arg("start")
		.arg(p)
		.spawn()?
		.wait()?;

	Ok(())
}

/// Tries to find an available C compiler backend.
/// Currently only supports gcc -> clang.
pub fn try_locate() -> anyhow::Result<Box<dyn Docgen>> {
	match which::which("doxygen") {
		Ok(_) => Ok(Box::new(Doxygen)),
		Err(_) => match which::which("cldoc") {
			Ok(_) => Ok(Box::new(Cldoc)),
			Err(_) => Err(anyhow::anyhow!("Couldn't find doxygen or cldoc."))
		}
	}
}