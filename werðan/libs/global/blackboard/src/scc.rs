//! Tarjan strongly-connected-component condensation for the global dependency DAG.
//! Implements METHODOLOGY.md section 3 S2 Tarjan condensation feeding the Tier C macro-regression.
//! Output vertices keep singleton ids verbatim. Collapsed components join member ids with a plus.

use crate::dag::CondensedDag;

/// Result of one Tarjan condensation over the input DAG.
/// Vertex order follows the original input vertex order.
pub fn tarjan_condense(dag: &self::CondensedDag) -> self::CondensedDag {
    let mut index_of = ::std::collections::BTreeMap::new();
    for (position, vertex) in dag.vertices.iter().enumerate() {
        index_of.insert(vertex.clone(), position);
    }
    let mut adjacency = ::std::collections::BTreeMap::new();
    for vertex in &dag.vertices {
        let empty_neighbors = ::std::vec::Vec::new();
        adjacency.insert(vertex.clone(), empty_neighbors);
    }
    for edge in &dag.edges {
        if let ::core::option::Option::Some(targets) = adjacency.get_mut(&edge.0) {
            targets.push(edge.1.clone());
        }
    }

    let vertex_count = dag.vertices.len();
    let mut indices = ::std::vec![::core::primitive::usize::MAX; vertex_count];
    let mut low_links = ::std::vec![::core::primitive::usize::MAX; vertex_count];
    let mut on_stack = ::std::vec![false; vertex_count];
    let mut stack = ::std::vec::Vec::new();
    let mut next_index = 0;
    let mut components = ::std::vec::Vec::new();

    for start in 0..vertex_count {
        let already_indexed = indices[start] != ::core::primitive::usize::MAX;
        if already_indexed {
            continue;
        }
        // Iterative Tarjan with an explicit frame stack of (vertex, next-child-cursor).
        let mut work = ::std::vec![(start, 0)];
        while let ::core::option::Option::Some(frame) = work.last_mut() {
            let vertex = frame.0;
            let cursor = frame.1;
            if cursor == 0 {
                indices[vertex] = next_index;
                low_links[vertex] = next_index;
                next_index += 1;
                stack.push(vertex);
                on_stack[vertex] = true;
            }
            let neighbors = adjacency.get(&dag.vertices[vertex]).cloned().unwrap_or_default();
            let mut advanced = false;
            while frame.1 < neighbors.len() {
                let target_name = &neighbors[frame.1];
                frame.1 += 1;
                let target = index_of.get(target_name).copied().unwrap_or(::core::primitive::usize::MAX);
                let target_known = target == ::core::primitive::usize::MAX;
                if target_known {
                    continue;
                }
                let target_unvisited = indices[target] == ::core::primitive::usize::MAX;
                if target_unvisited {
                    work.push((target, 0));
                    advanced = true;
                    break;
                } else if on_stack[target] {
                    low_links[vertex] = low_links[vertex].min(indices[target]);
                }
            }
            if advanced {
                continue;
            }
            // All children processed. Fold the low link into the parent frame if present.
            work.pop();
            if let ::core::option::Option::Some(parent) = work.last_mut() {
                low_links[parent.0] = low_links[parent.0].min(low_links[vertex]);
            }
            let is_root = low_links[vertex] == indices[vertex];
            if is_root {
                let mut component = ::std::vec::Vec::new();
                while let ::core::option::Option::Some(top) = stack.pop() {
                    on_stack[top] = false;
                    component.push(top);
                    let top_is_vertex = top == vertex;
                    if top_is_vertex {
                        break;
                    }
                }
                components.push(component);
            }
        }
    }

    // Deterministic merge ids: singletons keep their id, nontrivial components join with plus.
    // The loop orders components and members by the original input vertex order.
    let mut ordered_components = components;
    for component in ordered_components.iter_mut() {
        component.sort_unstable();
    }
    ordered_components.sort_unstable();
    let mut merged_of = ::std::collections::BTreeMap::new();
    let mut condensed = self::CondensedDag::new();
    for component in &ordered_components {
        let mut members = ::std::vec::Vec::new();
        for vertex_index in component {
            members.push(dag.vertices[*vertex_index].clone());
        }
        let merged_id = if members.len() == 1 {
            members[0].clone()
        } else {
            members.join("+")
        };
        condensed.vertices.push(merged_id.clone());
        for member in members {
            merged_of.insert(member, merged_id.clone());
        }
    }
    let mut seen_edges = ::std::collections::BTreeSet::new();
    for edge in &dag.edges {
        let source = merged_of.get(&edge.0).cloned().unwrap_or_else(|| edge.0.clone());
        let target = merged_of.get(&edge.1).cloned().unwrap_or_else(|| edge.1.clone());
        let internal = source == target;
        if internal {
            continue;
        }
        let fresh = seen_edges.insert((source.clone(), target.clone()));
        if fresh {
            condensed.edges.push((source, target));
        }
    }
    condensed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acyclic_passthrough() {
        let mut dag = self::CondensedDag::new();
        dag.vertices = ::std::vec!["a".to_string(), "b".to_string()];
        dag.edges = ::std::vec![("a".to_string(), "b".to_string())];
        let condensed = self::tarjan_condense(&dag);
        ::std::assert_eq!(condensed.vertices, ::std::vec!["a".to_string(), "b".to_string()]);
        ::std::assert_eq!(condensed.edges, dag.edges);
    }

    #[test]
    fn test_two_cycle_collapse() {
        let mut dag = self::CondensedDag::new();
        dag.vertices = ::std::vec!["a".to_string(), "b".to_string(), "c".to_string()];
        dag.edges = ::std::vec![
            ("a".to_string(), "b".to_string()),
            ("b".to_string(), "a".to_string()),
            ("b".to_string(), "c".to_string()),
        ];
        let condensed = self::tarjan_condense(&dag);
        ::std::assert_eq!(condensed.vertices, ::std::vec!["a+b".to_string(), "c".to_string()]);
        ::std::assert_eq!(condensed.edges.len(), 1);
        ::std::assert_eq!(condensed.edges[0].0, "a+b");
        ::std::assert_eq!(condensed.edges[0].1, "c");
        ::std::assert!(condensed.is_acyclic());
    }

    #[test]
    fn test_cross_scc_edge_preservation() {
        let mut dag = self::CondensedDag::new();
        dag.vertices = ::std::vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string()];
        dag.edges = ::std::vec![
            ("a".to_string(), "b".to_string()),
            ("b".to_string(), "c".to_string()),
            ("c".to_string(), "a".to_string()),
            ("c".to_string(), "d".to_string()),
        ];
        let condensed = self::tarjan_condense(&dag);
        ::std::assert!(condensed.vertices.contains(&"a+b+c".to_string()));
        ::std::assert!(condensed.vertices.contains(&"d".to_string()));
        ::std::assert_eq!(condensed.edges, ::std::vec![("a+b+c".to_string(), "d".to_string())]);
    }

    #[test]
    fn test_diamond_graph() {
        let mut dag = self::CondensedDag::new();
        dag.vertices = ::std::vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string()];
        dag.edges = ::std::vec![
            ("a".to_string(), "b".to_string()),
            ("a".to_string(), "c".to_string()),
            ("b".to_string(), "d".to_string()),
            ("c".to_string(), "d".to_string()),
        ];
        let condensed = self::tarjan_condense(&dag);
        ::std::assert_eq!(condensed.vertices, ::std::vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string()]);
        ::std::assert_eq!(condensed.edges, dag.edges);
    }

    #[test]
    fn test_duplicate_cross_edges_deduplicated() {
        let mut dag = self::CondensedDag::new();
        dag.vertices = ::std::vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string()];
        dag.edges = ::std::vec![
            ("a".to_string(), "b".to_string()),
            ("b".to_string(), "a".to_string()),
            ("c".to_string(), "d".to_string()),
            ("d".to_string(), "c".to_string()),
            ("a".to_string(), "c".to_string()),
            ("b".to_string(), "d".to_string()),
        ];
        let condensed = self::tarjan_condense(&dag);
        ::std::assert_eq!(condensed.vertices, ::std::vec!["a+b".to_string(), "c+d".to_string()]);
        ::std::assert_eq!(condensed.edges.len(), 1);
        ::std::assert_eq!(condensed.edges[0], ("a+b".to_string(), "c+d".to_string()));
    }
}
