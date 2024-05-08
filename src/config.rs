use std::collections::HashMap;

nestify::nest! {
	#[derive(serde::Serialize, serde::Deserialize)]*
	pub struct Config {
		pub package: pub struct ConfigPackage {
			pub name: String,
			/// Optional location to output the target binary
			pub bin: Option<String>
		},

		#[serde(default)]
		pub dependencies: HashMap<String, #[serde(untagged)] pub enum ConfigDependency {
			Path {
				path: String,
			},
			Git {
				git: String
			}
		}>,

		#[serde(default)]
		pub scripts: HashMap<String, String>,

		pub compiler: Option<pub struct ConfigCompiler {
			pub default: Option<String>,
			pub flags: Option<Vec<String>>,

			pub gcc: Option<pub struct ConfigGcc {
				pub flags: Option<Vec<String>>,
			}>,

			pub clang: Option<pub struct ConfigClang {
				pub flags: Option<Vec<String>>,
			}>
		}>,

		pub formatter: Option<pub struct ConfigFormatter {
			pub clang_format: toml::Table,
		}>,

		pub docgen: Option<pub struct ConfigDocgen {
			pub doxygen: pub struct ConfigDoxygen {
				pub doxyfile: String
			},
		}>
	}
}
