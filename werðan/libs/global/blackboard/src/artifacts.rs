//! Hardened target module artifacts A_artifacts per METHODOLOGY.md Definition 1.

/// Hardened target artifact committed to A_artifacts(v) on the global blackboard.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::core::cmp::PartialEq, ::core::cmp::Eq, ::serde::Serialize, ::serde::Deserialize)]
pub struct ModuleArtifact {
    /// Verified target code string.
    pub target_code: ::std::string::String,
    /// Characterization test suite items.
    pub test_suite: ::std::vec::Vec<::std::string::String>,
}

/// Hardened target artifact registry A_artifacts per METHODOLOGY.md Definition 1.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::core::cmp::PartialEq, ::core::cmp::Eq, ::serde::Serialize, ::serde::Deserialize)]
pub struct ArtifactRegistry {
    /// Hardened target artifacts indexed by vertex identifier.
    pub artifacts: ::std::collections::BTreeMap<::std::string::String, ::core::option::Option<self::ModuleArtifact>>,
}

#[::bon::bon]
impl self::ArtifactRegistry {
    /// Construct an artifact registry using a bon builder.
    #[builder]
    pub fn builder(
        file_path: ::core::option::Option<::std::path::PathBuf>,
        output_dir: ::core::option::Option<::std::path::PathBuf>,
        #[builder(default = "default".to_string())]
        hash: ::std::string::String,
        artifacts: ::core::option::Option<::std::collections::BTreeMap<::std::string::String, ::core::option::Option<self::ModuleArtifact>>>,
        #[builder(default = false)]
        create_if_missing: ::core::primitive::bool,
    ) -> ::core::result::Result<Self, ::std::io::Error> {
        let resolved_path = crate::path_resolver::resolve_component_path(
            file_path.as_deref(),
            output_dir.as_deref(),
            &hash,
            "artifacts.json",
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
        if let ::core::option::Option::Some(artifact_map) = artifacts {
            instance.artifacts = artifact_map;
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

    /// Save artifacts to resolved component path.
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
            "artifacts.json",
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
    /// Construct an empty artifact registry.
    pub fn new() -> Self {
        Self {
            artifacts: ::std::collections::BTreeMap::new(),
        }
    }

    /// Commit a hardened artifact for a vertex.
    pub fn commit(
        &mut self,
        vertex: impl ::core::convert::Into<::std::string::String>,
        artifact: self::ModuleArtifact,
    ) {
        let key = vertex.into();
        let value = ::core::option::Option::Some(artifact);
        self.artifacts.insert(key, value);
    }

    /// Insert an optional artifact for a vertex.
    pub fn insert(
        &mut self,
        vertex: impl ::core::convert::Into<::std::string::String>,
        artifact: ::core::option::Option<self::ModuleArtifact>,
    ) -> ::core::option::Option<::core::option::Option<self::ModuleArtifact>> {
        let key = vertex.into();
        self.artifacts.insert(key, artifact)
    }

    /// Borrow an artifact slot for a vertex.
    pub fn get(&self, vertex: &::core::primitive::str) -> ::core::option::Option<&::core::option::Option<self::ModuleArtifact>> {
        self.artifacts.get(vertex)
    }

    /// Report whether a vertex holds a hardened artifact.
    pub fn is_hardened(&self, vertex: &::core::primitive::str) -> ::core::primitive::bool {
        ::core::matches!(self.artifacts.get(vertex), ::core::option::Option::Some(::core::option::Option::Some(_)))
    }

    /// Report whether all specified vertices hold hardened artifacts.
    pub fn all_hardened(&self, vertices: &[::std::string::String]) -> ::core::primitive::bool {
        for vertex in vertices {
            let hardened = ::core::matches!(
                self.artifacts.get(vertex.as_str()),
                ::core::option::Option::Some(::core::option::Option::Some(_))
            );
            if !hardened {
                return false;
            }
        }
        true
    }

    /// Return total registered artifact slots.
    pub fn len(&self) -> ::core::primitive::usize {
        self.artifacts.len()
    }

    /// Report whether the artifact registry is empty.
    pub fn is_empty(&self) -> ::core::primitive::bool {
        self.artifacts.is_empty()
    }
}

impl ::core::ops::Deref for self::ArtifactRegistry {
    type Target = ::std::collections::BTreeMap<::std::string::String, ::core::option::Option<self::ModuleArtifact>>;

    fn deref(&self) -> &Self::Target {
        &self.artifacts
    }
}

impl ::core::ops::DerefMut for self::ArtifactRegistry {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.artifacts
    }
}

impl ::core::convert::From<::std::collections::BTreeMap<::std::string::String, ::core::option::Option<self::ModuleArtifact>>> for self::ArtifactRegistry {
    fn from(artifacts: ::std::collections::BTreeMap<::std::string::String, ::core::option::Option<self::ModuleArtifact>>) -> Self {
        Self { artifacts }
    }
}
