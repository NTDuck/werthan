//! Interface contract registry C_contracts per METHODOLOGY.md Definition 1.

/// Interface contract registry mapping dependency edges to edge contracts.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::serde::Serialize, ::serde::Deserialize)]
pub struct ContractRegistry {
    /// Interface contracts indexed by parent and child edge pair.
    pub contracts: ::std::collections::BTreeMap<(::std::string::String, ::std::string::String), ::holon_blackboard::EdgeContract>,
}

#[::bon::bon]
impl self::ContractRegistry {
    /// Construct a contract registry using a bon builder.
    #[builder]
    pub fn builder(
        file_path: ::core::option::Option<::std::path::PathBuf>,
        output_dir: ::core::option::Option<::std::path::PathBuf>,
        #[builder(default = "default".to_string())]
        hash: ::std::string::String,
        contracts: ::core::option::Option<::std::collections::BTreeMap<(::std::string::String, ::std::string::String), ::holon_blackboard::EdgeContract>>,
        #[builder(default = false)]
        create_if_missing: ::core::primitive::bool,
    ) -> ::core::result::Result<Self, ::std::io::Error> {
        let resolved_path = crate::path_resolver::resolve_component_path(
            file_path.as_deref(),
            output_dir.as_deref(),
            &hash,
            "contracts.json",
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
        if let ::core::option::Option::Some(contract_map) = contracts {
            instance.contracts = contract_map;
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

    /// Save contracts to resolved component path.
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
            "contracts.json",
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
    /// Construct an empty contract registry.
    pub fn new() -> Self {
        Self {
            contracts: ::std::collections::BTreeMap::new(),
        }
    }

    /// Insert an edge contract for a parent and child edge pair.
    pub fn insert(
        &mut self,
        edge: (::std::string::String, ::std::string::String),
        contract: ::holon_blackboard::EdgeContract,
    ) -> ::core::option::Option<::holon_blackboard::EdgeContract> {
        self.contracts.insert(edge, contract)
    }

    /// Borrow an edge contract for a given edge pair.
    pub fn get(&self, edge: &(::std::string::String, ::std::string::String)) -> ::core::option::Option<&::holon_blackboard::EdgeContract> {
        self.contracts.get(edge)
    }

    /// Mutably borrow an edge contract for a given edge pair.
    pub fn get_mut(&mut self, edge: &(::std::string::String, ::std::string::String)) -> ::core::option::Option<&mut ::holon_blackboard::EdgeContract> {
        self.contracts.get_mut(edge)
    }

    /// Report whether the registry contains a contract for an edge pair.
    pub fn contains_edge(&self, edge: &(::std::string::String, ::std::string::String)) -> ::core::primitive::bool {
        self.contracts.contains_key(edge)
    }

    /// Return contracts where the given module is the downstream child.
    pub fn contracts_for_child(&self, child: &::core::primitive::str) -> ::std::vec::Vec<&::holon_blackboard::EdgeContract> {
        let mut results = ::std::vec::Vec::new();
        for (edge_key, contract) in &self.contracts {
            let child_matches = edge_key.1 == child;
            if child_matches {
                results.push(contract);
            }
        }
        results
    }

    /// Return contracts where the given module is the upstream parent.
    pub fn contracts_from_parent(&self, parent: &::core::primitive::str) -> ::std::vec::Vec<&::holon_blackboard::EdgeContract> {
        let mut results = ::std::vec::Vec::new();
        for (edge_key, contract) in &self.contracts {
            let parent_matches = edge_key.0 == parent;
            if parent_matches {
                results.push(contract);
            }
        }
        results
    }

    /// Return total registered contract count.
    pub fn len(&self) -> ::core::primitive::usize {
        self.contracts.len()
    }

    /// Report whether the contract registry is empty.
    pub fn is_empty(&self) -> ::core::primitive::bool {
        self.contracts.is_empty()
    }
}

impl ::core::ops::Deref for self::ContractRegistry {
    type Target = ::std::collections::BTreeMap<(::std::string::String, ::std::string::String), ::holon_blackboard::EdgeContract>;

    fn deref(&self) -> &Self::Target {
        &self.contracts
    }
}

impl ::core::ops::DerefMut for self::ContractRegistry {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.contracts
    }
}

impl ::core::convert::From<::std::collections::BTreeMap<(::std::string::String, ::std::string::String), ::holon_blackboard::EdgeContract>> for self::ContractRegistry {
    fn from(contracts: ::std::collections::BTreeMap<(::std::string::String, ::std::string::String), ::holon_blackboard::EdgeContract>) -> Self {
        Self { contracts }
    }
}
