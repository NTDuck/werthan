//! Condensed dependency directed acyclic graph G = (V, E) per METHODOLOGY.md Definition 1.

/// Condensed dependency directed acyclic graph G = (V, E) from METHODOLOGY.md Definition 1.
/// Each directed edge is a parent to child pair. The child depends on symbols of the parent.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::core::cmp::PartialEq, ::core::cmp::Eq, ::serde::Serialize, ::serde::Deserialize)]
pub struct CondensedDag {
    /// Vertex identifiers in deterministic discovery order.
    pub vertices: ::std::vec::Vec<::std::string::String>,
    /// Directed dependency edges as parent to child pairs.
    pub edges: ::std::vec::Vec<(::std::string::String, ::std::string::String)>,
}

#[::bon::bon]
impl self::CondensedDag {
    /// Construct a condensed DAG using a bon builder.
    #[builder]
    pub fn builder(
        file_path: ::core::option::Option<::std::path::PathBuf>,
        output_dir: ::core::option::Option<::std::path::PathBuf>,
        #[builder(default = "default".to_string())]
        hash: ::std::string::String,
        vertices: ::core::option::Option<::std::vec::Vec<::std::string::String>>,
        edges: ::core::option::Option<::std::vec::Vec<(::std::string::String, ::std::string::String)>>,
        #[builder(default = false)]
        create_if_missing: ::core::primitive::bool,
    ) -> ::core::result::Result<Self, ::std::io::Error> {
        let resolved_path = crate::path_resolver::resolve_component_path(
            file_path.as_deref(),
            output_dir.as_deref(),
            &hash,
            "dag.json",
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
        if let ::core::option::Option::Some(vertex_list) = vertices {
            instance.vertices = vertex_list;
        }
        if let ::core::option::Option::Some(edge_list) = edges {
            instance.edges = edge_list;
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

    /// Save DAG to resolved component path.
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
            "dag.json",
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
    /// Construct an empty condensed DAG.
    pub fn new() -> Self {
        Self::default()
    }

    /// Return parent identifiers that the given child depends on.
    pub fn parents_of(&self, child: &::core::primitive::str) -> ::std::vec::Vec<::std::string::String> {
        let mut parents = ::std::vec::Vec::new();
        for edge in &self.edges {
            let child_matches = edge.1 == child;
            if child_matches {
                let parent_id = edge.0.clone();
                parents.push(parent_id);
            }
        }
        parents
    }

    /// Return child identifiers that depend on the given parent.
    pub fn children_of(&self, parent: &::core::primitive::str) -> ::std::vec::Vec<::std::string::String> {
        let mut children = ::std::vec::Vec::new();
        for edge in &self.edges {
            let parent_matches = edge.0 == parent;
            if parent_matches {
                let child_id = edge.1.clone();
                children.push(child_id);
            }
        }
        children
    }

    /// Report whether the graph is strictly acyclic via Kahn in-degree elimination.
    pub fn is_acyclic(&self) -> ::core::primitive::bool {
        let mut in_degree = ::std::collections::BTreeMap::new();
        for vertex in &self.vertices {
            let vertex_key = vertex.clone();
            in_degree.insert(vertex_key, 0);
        }
        for edge in &self.edges {
            let target_key = edge.1.clone();
            let entry = in_degree.entry(target_key).or_insert(0);
            *entry += 1;
        }
        let mut queue = ::std::collections::VecDeque::new();
        for (vertex, degree) in &in_degree {
            let degree_is_zero = *degree == 0;
            if degree_is_zero {
                let zero_vertex = vertex.clone();
                queue.push_back(zero_vertex);
            }
        }
        let mut visited = 0;
        while let ::core::option::Option::Some(current) = queue.pop_front() {
            visited += 1;
            for edge in &self.edges {
                let edge_leaves_current = edge.0 == current;
                if edge_leaves_current {
                    if let ::core::option::Option::Some(degree) = in_degree.get_mut(&edge.1) {
                        *degree -= 1;
                        let degree_now_zero = *degree == 0;
                        if degree_now_zero {
                            let next_vertex = edge.1.clone();
                            queue.push_back(next_vertex);
                        }
                    }
                }
            }
        }
        visited == self.vertices.len()
    }
}
