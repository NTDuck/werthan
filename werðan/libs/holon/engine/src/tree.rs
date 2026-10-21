//! Search tree representation and Upper Confidence Bound traversal.
//! Implements Softmax-UCB selection and backpropagation.

/// Single search node within the MCTS tree.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
pub struct SearchNode {
    /// Unique node identifier index in the arena.
    pub id: ::core::primitive::usize,
    /// Parent node index in the arena.
    pub parent_id: ::core::option::Option<::core::primitive::usize>,
    /// Child node indices.
    pub children: ::std::vec::Vec<::core::primitive::usize>,
    /// Candidate target code draft.
    pub candidate_code: ::core::option::Option<::std::string::String>,
    /// Strategy hypothesis identifier if spawned from strategy.
    pub strategy_id: ::core::option::Option<::std::string::String>,
    /// Strategy prior utility score.
    pub prior: ::core::primitive::f64,
    /// Visit count of this node across search rollouts.
    pub visit_count: ::core::primitive::usize,
    /// Cumulative backpropagated reward sum.
    pub total_reward: ::core::primitive::f64,
    /// Mean backpropagated reward score.
    pub mean_reward: ::core::primitive::f64,
    /// Evaluation outcome cache.
    pub eval_result: ::core::option::Option<::holon_evaluator::EvaluationResult>,
}

impl self::SearchNode {
    /// Construct a new unvisited node.
    pub fn new(
        id: ::core::primitive::usize,
        parent_id: ::core::option::Option<::core::primitive::usize>,
        candidate_code: ::core::option::Option<::std::string::String>,
        strategy_id: ::core::option::Option<::std::string::String>,
        prior: ::core::primitive::f64,
    ) -> Self {
        Self {
            id,
            parent_id,
            children: ::std::vec::Vec::new(),
            candidate_code,
            strategy_id,
            prior,
            visit_count: 0,
            total_reward: 0.0,
            mean_reward: 0.0,
            eval_result: ::core::option::Option::None,
        }
    }

    /// Inspect child node list emptiness.
    pub fn is_leaf(&self) -> ::core::primitive::bool {
        self.children.is_empty()
    }
}

/// MCTS search tree arena holding all rollout nodes.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
pub struct SearchTree {
    nodes: ::std::vec::Vec<self::SearchNode>,
    root_id: ::core::primitive::usize,
}

impl self::SearchTree {
    /// Create a new search tree with a root node.
    pub fn new(initial_candidate: ::core::option::Option<::std::string::String>) -> Self {
        let root = self::SearchNode::new(
            0,
            ::core::option::Option::None,
            initial_candidate,
            ::core::option::Option::None,
            0.5,
        );
        let nodes = ::std::vec![root];

        Self {
            nodes,
            root_id: 0,
        }
    }

    /// Return active root node index.
    pub fn root_id(&self) -> ::core::primitive::usize {
        self.root_id
    }

    /// Set root node index for warm start regression while preserving tree.
    pub fn set_warm_root(&mut self, new_root_id: ::core::primitive::usize) {
        if new_root_id < self.nodes.len() {
            self.root_id = new_root_id;
        }
    }

    /// Return reference to node by index.
    pub fn get(&self, id: ::core::primitive::usize) -> ::core::option::Option<&self::SearchNode> {
        let node = self.nodes.get(id);
        node
    }

    /// Return mutable reference to node by index.
    pub fn get_mut(&mut self, id: ::core::primitive::usize) -> ::core::option::Option<&mut self::SearchNode> {
        let node = self.nodes.get_mut(id);
        node
    }

    /// Total number of nodes in tree arena.
    pub fn len(&self) -> ::core::primitive::usize {
        self.nodes.len()
    }

    /// Return true when tree is empty.
    pub fn is_empty(&self) -> ::core::primitive::bool {
        self.nodes.is_empty()
    }

    /// Add a child node to an existing parent node.
    pub fn add_child(
        &mut self,
        parent_id: ::core::primitive::usize,
        candidate_code: ::core::option::Option<::std::string::String>,
        strategy_id: ::core::option::Option<::std::string::String>,
        prior: ::core::primitive::f64,
    ) -> ::core::primitive::usize {
        let next_id = self.nodes.len();
        let child = self::SearchNode::new(
            next_id,
            ::core::option::Option::Some(parent_id),
            candidate_code,
            strategy_id,
            prior,
        );
        self.nodes.push(child);

        if let ::core::option::Option::Some(parent) = self.nodes.get_mut(parent_id) {
            parent.children.push(next_id);
        }

        next_id
    }

    /// Traverse search tree from root using Softmax-UCB to an expandable node.
    pub fn select_expandable(
        &self,
        c_exp: ::core::primitive::f64,
        gamma_prior: ::core::primitive::f64,
        temperature: ::core::primitive::f64,
    ) -> ::core::primitive::usize {
        let mut current_id = self.root_id;

        loop {
            let node = match self.nodes.get(current_id) {
                ::core::option::Option::Some(n) => n,
                ::core::option::Option::None => return self.root_id,
            };

            if node.children.is_empty() {
                return current_id;
            }

            let next_opt = self.select_softmax_child(current_id, c_exp, gamma_prior, temperature);
            match next_opt {
                ::core::option::Option::Some(next_id) => {
                    let next_node = match self.nodes.get(next_id) {
                        ::core::option::Option::Some(n) => n,
                        ::core::option::Option::None => return current_id,
                    };
                    if next_node.visit_count == 0 {
                        return next_id;
                    }
                    current_id = next_id;
                }
                ::core::option::Option::None => return current_id,
            }
        }
    }

    /// Select child using softmax distribution over UCB scores.
    pub fn select_softmax_child(
        &self,
        parent_id: ::core::primitive::usize,
        c_exp: ::core::primitive::f64,
        gamma_prior: ::core::primitive::f64,
        temperature: ::core::primitive::f64,
    ) -> ::core::option::Option<::core::primitive::usize> {
        let parent = self.nodes.get(parent_id)?;
        if parent.children.is_empty() {
            return ::core::option::Option::None;
        }

        let mut ucb_scores = ::std::vec::Vec::new();
        let mut max_ucb = ::core::primitive::f64::NEG_INFINITY;

        for &child_id in &parent.children {
            let ucb = self.compute_ucb(child_id, parent_id, c_exp, gamma_prior);
            if ucb > max_ucb {
                max_ucb = ucb;
            }
            ucb_scores.push((child_id, ucb));
        }

        let tau = if temperature <= 0.001 { 0.001 } else { temperature };
        let mut exp_weights = ::std::vec::Vec::new();
        let mut sum_exp = 0.0;

        for (child_id, ucb) in ucb_scores {
            let delta = (ucb - max_ucb) / tau;
            let weight = delta.exp();
            sum_exp += weight;
            exp_weights.push((child_id, weight));
        }

        if sum_exp <= 0.0 {
            return parent.children.first().copied();
        }

        let mut best_id = parent.children[0];
        let mut best_w = -1.0;
        for (child_id, w) in exp_weights {
            if w > best_w {
                best_w = w;
                best_id = child_id;
            }
        }

        ::core::option::Option::Some(best_id)
    }

    /// Calculate Upper Confidence Bound score for a child node.
    pub fn compute_ucb(
        &self,
        child_id: ::core::primitive::usize,
        parent_id: ::core::primitive::usize,
        c_exp: ::core::primitive::f64,
        gamma_prior: ::core::primitive::f64,
    ) -> ::core::primitive::f64 {
        let child = match self.nodes.get(child_id) {
            ::core::option::Option::Some(c) => c,
            ::core::option::Option::None => return 0.0,
        };
        let parent = match self.nodes.get(parent_id) {
            ::core::option::Option::Some(p) => p,
            ::core::option::Option::None => return 0.0,
        };

        if child.visit_count == 0 {
            let prior_bonus = gamma_prior * child.prior;
            return 1000.0 + prior_bonus;
        }

        let n_parent = parent.visit_count as ::core::primitive::f64;
        let n_child = child.visit_count as ::core::primitive::f64;

        let log_val = (n_parent + 1.0).ln();
        let ratio = log_val / n_child;
        let sqrt_ratio = ratio.sqrt();
        let explore_term = c_exp * sqrt_ratio;
        let prior_term = gamma_prior * child.prior;

        child.mean_reward + explore_term + prior_term
    }

    /// Backpropagate evaluation reward along path to root.
    pub fn backpropagate(&mut self, leaf_id: ::core::primitive::usize, reward: ::core::primitive::f64) {
        let mut curr_opt = ::core::option::Option::Some(leaf_id);

        while let ::core::option::Option::Some(curr_id) = curr_opt {
            let node = match self.nodes.get_mut(curr_id) {
                ::core::option::Option::Some(n) => n,
                ::core::option::Option::None => break,
            };

            node.visit_count += 1;
            node.total_reward += reward;
            let n_f = node.visit_count as ::core::primitive::f64;
            node.mean_reward = node.total_reward / n_f;

            if curr_id == self.root_id {
                break;
            }
            curr_opt = node.parent_id;
        }
    }

    /// Locate node with highest mean reward across the tree.
    pub fn best_node(&self) -> ::core::primitive::usize {
        let mut best_id = self.root_id;
        let mut best_score = ::core::primitive::f64::NEG_INFINITY;

        for node in &self.nodes {
            if node.id != self.root_id && node.visit_count > 0 && node.mean_reward >= best_score {
                best_score = node.mean_reward;
                best_id = node.id;
            }
        }

        best_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_construction_and_expansion() {
        let mut tree = self::SearchTree::new(::core::option::Option::None);
        ::std::assert_eq!(tree.len(), 1);
        ::std::assert_eq!(tree.root_id(), 0);

        let child1 = tree.add_child(
            0,
            ::core::option::Option::Some("code1".to_string()),
            ::core::option::Option::Some("strat1".to_string()),
            0.8,
        );
        let _child2 = tree.add_child(
            0,
            ::core::option::Option::Some("code2".to_string()),
            ::core::option::Option::Some("strat2".to_string()),
            0.4,
        );
        ::std::assert_eq!(tree.len(), 3);

        tree.backpropagate(child1, 0.9);
        let node1 = tree.get(child1).unwrap();
        ::std::assert_eq!(node1.visit_count, 1);
        ::std::assert!((node1.mean_reward - 0.9).abs() < 1e-6);

        let root = tree.get(0).unwrap();
        ::std::assert_eq!(root.visit_count, 1);
        ::std::assert!((root.mean_reward - 0.9).abs() < 1e-6);

        let selected = tree.select_softmax_child(0, 1.414, 0.5, 0.5);
        ::std::assert!(selected.is_some());

        let best = tree.best_node();
        ::std::assert_eq!(best, child1);
    }

    #[test]
    fn test_warm_root_setting() {
        let mut tree = self::SearchTree::new(::core::option::Option::None);
        let child = tree.add_child(
            0,
            ::core::option::Option::Some("hardened_code".to_string()),
            ::core::option::Option::None,
            1.0,
        );
        tree.set_warm_root(child);
        ::std::assert_eq!(tree.root_id(), child);
    }
}
