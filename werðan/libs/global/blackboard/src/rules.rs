//! Business rule card registry R_rules per METHODOLOGY.md Definition 1.

/// Registry R_rules mapping vertices to verified business rule cards.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::serde::Serialize, ::serde::Deserialize)]
pub struct RuleRegistry {
    /// Rule cards indexed by vertex identifier.
    pub rules: ::std::collections::BTreeMap<::std::string::String, ::std::vec::Vec<::holon_blackboard::BusinessRuleCard>>,
}

#[::bon::bon]
impl self::RuleRegistry {
    /// Construct a rule registry using a bon builder.
    #[builder]
    pub fn builder(
        file_path: ::core::option::Option<::std::path::PathBuf>,
        output_dir: ::core::option::Option<::std::path::PathBuf>,
        #[builder(default = "default".to_string())]
        hash: ::std::string::String,
        rules: ::core::option::Option<::std::collections::BTreeMap<::std::string::String, ::std::vec::Vec<::holon_blackboard::BusinessRuleCard>>>,
        #[builder(default = false)]
        create_if_missing: ::core::primitive::bool,
    ) -> ::core::result::Result<Self, ::std::io::Error> {
        let resolved_path = crate::path_resolver::resolve_component_path(
            file_path.as_deref(),
            output_dir.as_deref(),
            &hash,
            "rules.json",
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
        if let ::core::option::Option::Some(rule_map) = rules {
            instance.rules = rule_map;
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

    /// Save rules to resolved component path.
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
            "rules.json",
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
    /// Construct an empty rule registry.
    pub fn new() -> Self {
        Self {
            rules: ::std::collections::BTreeMap::new(),
        }
    }

    /// Insert rule cards for a vertex.
    pub fn insert(
        &mut self,
        vertex: impl ::core::convert::Into<::std::string::String>,
        cards: ::std::vec::Vec<::holon_blackboard::BusinessRuleCard>,
    ) -> ::core::option::Option<::std::vec::Vec<::holon_blackboard::BusinessRuleCard>> {
        let key = vertex.into();
        self.rules.insert(key, cards)
    }

    /// Borrow rule cards for a vertex.
    pub fn get(&self, vertex: &::core::primitive::str) -> ::core::option::Option<&::std::vec::Vec<::holon_blackboard::BusinessRuleCard>> {
        self.rules.get(vertex)
    }

    /// Mutably borrow rule cards for a vertex.
    pub fn get_mut(&mut self, vertex: &::core::primitive::str) -> ::core::option::Option<&mut ::std::vec::Vec<::holon_blackboard::BusinessRuleCard>> {
        self.rules.get_mut(vertex)
    }

    /// Report whether the registry contains rule cards for a vertex.
    pub fn contains(&self, vertex: &::core::primitive::str) -> ::core::primitive::bool {
        self.rules.contains_key(vertex)
    }

    /// Return total registered vertex count.
    pub fn len(&self) -> ::core::primitive::usize {
        self.rules.len()
    }

    /// Report whether the rule registry is empty.
    pub fn is_empty(&self) -> ::core::primitive::bool {
        self.rules.is_empty()
    }

    /// Return an iterator over all verified rule cards across all vertices.
    pub fn all_rules(&self) -> impl ::core::iter::Iterator<Item = &::holon_blackboard::BusinessRuleCard> {
        self.rules.values().flatten()
    }
}

impl ::core::ops::Deref for self::RuleRegistry {
    type Target = ::std::collections::BTreeMap<::std::string::String, ::std::vec::Vec<::holon_blackboard::BusinessRuleCard>>;

    fn deref(&self) -> &Self::Target {
        &self.rules
    }
}

impl ::core::ops::DerefMut for self::RuleRegistry {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.rules
    }
}

impl ::core::convert::From<::std::collections::BTreeMap<::std::string::String, ::std::vec::Vec<::holon_blackboard::BusinessRuleCard>>> for self::RuleRegistry {
    fn from(rules: ::std::collections::BTreeMap<::std::string::String, ::std::vec::Vec<::holon_blackboard::BusinessRuleCard>>) -> Self {
        Self { rules }
    }
}
