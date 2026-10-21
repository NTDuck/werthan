//! Repository inventory and dialect configuration per METHODOLOGY.md Definition 1.

/// Dialect profile for the source and target language toolchains.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::core::cmp::PartialEq, ::core::cmp::Eq, ::serde::Serialize, ::serde::Deserialize)]
pub struct DialectProfile {
    /// Source language name.
    pub source_language: ::std::string::String,
    /// Target language name.
    pub target_language: ::std::string::String,
    /// Target edition or version specification.
    pub target_edition: ::std::string::String,
}

/// Verified build and test harness command configuration.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::core::cmp::PartialEq, ::core::cmp::Eq, ::serde::Serialize, ::serde::Deserialize)]
pub struct HarnessConfig {
    /// Target build command.
    pub build_cmd: ::std::string::String,
    /// Target test command.
    pub test_cmd: ::std::string::String,
    /// Legacy harness execution command.
    pub legacy_harness_cmd: ::std::string::String,
}

/// Repository inventory holding file paths, dialect, and harness configuration.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::core::cmp::PartialEq, ::core::cmp::Eq, ::serde::Serialize, ::serde::Deserialize)]
pub struct RepositoryInventory {
    /// Repository file inventory entries.
    pub files: ::std::vec::Vec<::std::string::String>,
    /// Dialect configuration profile.
    pub dialect: self::DialectProfile,
    /// Verified harness configuration.
    pub harness: self::HarnessConfig,
}

#[::bon::bon]
impl self::RepositoryInventory {
    /// Construct a repository inventory using a bon builder.
    #[builder]
    pub fn builder(
        file_path: ::core::option::Option<::std::path::PathBuf>,
        output_dir: ::core::option::Option<::std::path::PathBuf>,
        #[builder(default = "default".to_string())]
        hash: ::std::string::String,
        files: ::core::option::Option<::std::vec::Vec<::std::string::String>>,
        dialect: ::core::option::Option<self::DialectProfile>,
        harness: ::core::option::Option<self::HarnessConfig>,
        #[builder(default = false)]
        create_if_missing: ::core::primitive::bool,
    ) -> ::core::result::Result<Self, ::std::io::Error> {
        let resolved_path = crate::path_resolver::resolve_component_path(
            file_path.as_deref(),
            output_dir.as_deref(),
            &hash,
            "inventory.json",
        );
        let mut instance = if resolved_path.is_file() {
            let file_contents = ::std::fs::read_to_string(&resolved_path)?;
            let parsed_result = ::serde_json::from_str::<Self>(&file_contents);
            match parsed_result {
                ::core::result::Result::Ok(data) => data,
                ::core::result::Result::Err(err) => {
                    let io_error = ::std::io::Error::new(::std::io::ErrorKind::InvalidData, err);
                    return ::core::result::Result::Err(io_error);
                }
            }
        } else {
            Self::new()
        };
        if let ::core::option::Option::Some(files_list) = files {
            instance.files = files_list;
        }
        if let ::core::option::Option::Some(dialect_cfg) = dialect {
            instance.dialect = dialect_cfg;
        }
        if let ::core::option::Option::Some(harness_cfg) = harness {
            instance.harness = harness_cfg;
        }
        if create_if_missing && !resolved_path.is_file() {
            if let ::core::option::Option::Some(parent_dir) = resolved_path.parent() {
                let _ = ::std::fs::create_dir_all(parent_dir);
            }
            let serialized_res = ::serde_json::to_string_pretty(&instance);
            let serialized = match serialized_res {
                ::core::result::Result::Ok(content) => content,
                ::core::result::Result::Err(err) => {
                    let io_error = ::std::io::Error::new(::std::io::ErrorKind::InvalidData, err);
                    return ::core::result::Result::Err(io_error);
                }
            };
            let _ = ::std::fs::write(&resolved_path, serialized);
        }
        ::core::result::Result::Ok(instance)
    }

    /// Save inventory to resolved component path.
    pub fn save(
        &self,
        file_path: ::core::option::Option<&::std::path::Path>,
        output_dir: ::core::option::Option<&::std::path::Path>,
        hash: &::core::primitive::str,
    ) -> ::core::result::Result<::std::path::PathBuf, ::std::io::Error> {
        let target_path = crate::path_resolver::resolve_component_path(
            file_path,
            output_dir,
            hash,
            "inventory.json",
        );
        if let ::core::option::Option::Some(parent_dir) = target_path.parent() {
            let _ = ::std::fs::create_dir_all(parent_dir);
        }
        let serialized_res = ::serde_json::to_string_pretty(self);
        let serialized = match serialized_res {
            ::core::result::Result::Ok(content) => content,
            ::core::result::Result::Err(err) => {
                let io_error = ::std::io::Error::new(::std::io::ErrorKind::InvalidData, err);
                return ::core::result::Result::Err(io_error);
            }
        };
        ::std::fs::write(&target_path, serialized)?;
        ::core::result::Result::Ok(target_path)
    }
    /// Construct an empty repository inventory.
    pub fn new() -> Self {
        Self {
            files: ::std::vec::Vec::new(),
            dialect: self::DialectProfile::default(),
            harness: self::HarnessConfig::default(),
        }
    }

    /// Add a file into the inventory.
    pub fn add_file(&mut self, file: impl ::core::convert::Into<::std::string::String>) {
        let path = file.into();
        self.files.push(path);
    }

    /// Push a file into the inventory.
    pub fn push(&mut self, file: impl ::core::convert::Into<::std::string::String>) {
        let path = file.into();
        self.files.push(path);
    }

    /// Return a slice view over the inventory files.
    pub fn files(&self) -> &[::std::string::String] {
        &self.files
    }

    /// Return total file count in the inventory.
    pub fn len(&self) -> ::core::primitive::usize {
        self.files.len()
    }

    /// Report whether the file inventory is empty.
    pub fn is_empty(&self) -> ::core::primitive::bool {
        self.files.is_empty()
    }

    /// Report whether the inventory contains a file path.
    pub fn contains(&self, path: &::core::primitive::str) -> ::core::primitive::bool {
        self.files.iter().any(|item| item == path)
    }

    /// Return an iterator over file path references.
    pub fn iter(&self) -> impl ::core::iter::Iterator<Item = &::std::string::String> {
        self.files.iter()
    }
}

impl ::core::ops::Deref for self::RepositoryInventory {
    type Target = [::std::string::String];

    fn deref(&self) -> &Self::Target {
        &self.files
    }
}

impl ::core::convert::From<::std::vec::Vec<::std::string::String>> for self::RepositoryInventory {
    fn from(files: ::std::vec::Vec<::std::string::String>) -> Self {
        Self {
            files,
            dialect: self::DialectProfile::default(),
            harness: self::HarnessConfig::default(),
        }
    }
}

impl<'a> ::core::iter::IntoIterator for &'a self::RepositoryInventory {
    type Item = &'a ::std::string::String;
    type IntoIter = ::core::slice::Iter<'a, ::std::string::String>;

    fn into_iter(self) -> Self::IntoIter {
        self.files.iter()
    }
}

impl ::core::iter::IntoIterator for self::RepositoryInventory {
    type Item = ::std::string::String;
    type IntoIter = ::std::vec::IntoIter<::std::string::String>;

    fn into_iter(self) -> Self::IntoIter {
        self.files.into_iter()
    }
}
