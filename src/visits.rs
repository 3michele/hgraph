use std::collections::VecDeque;

use ahash::AHashSet;

use super::{Hypergraph, Node};

/// `type Node = i64`
///
/// Breadth-First-Search of the hypergraph starting from a given node.   
///
/// # Parameters  
/// - `hg` : `&Hypergraph` - The hypergraph to search.
/// - `start` : `Node` - The node to start the search from.
/// - `max_depth` : `Option<usize>` - `Some` maximum depth for the search. If `None` the search is not limited.
/// - `order` : `Option<usize>` - `Some` order of the hyperedges to consider. If `None` all hyperedges are considered.
/// - `size` : `Option<usize>` - `Some` size of the hyperedges to consider. If `None` all hyperedges are considered.
///
/// # Returns
/// - `AHashSet<Node>` - The nodes visited during the search. If the length of the returned hashset is `0`, then it means 
/// that the node provided was not in the hypergraph.
///
/// # Performance
/// - `O(n*n*m)`, where `n` and `m` are the number of nodes and hyperedges of the hypergraph, respectively.
pub fn _bfs(
    hg: &Hypergraph,
    start: Node,
    max_depth: Option<usize>,
    order: Option<usize>,
    size: Option<usize>,
) -> AHashSet<Node> {
    let mut visited = AHashSet::new();

    if hg.check_node(start) { // Added this check
        let mut queue = VecDeque::new();
        queue.push_back((start, 0));
        visited.insert(start);

        // O(n)
        while let Some((now, depth)) = queue.pop_front() {
            if max_depth.map_or(true, |max| depth < max) {
                // O(n*m)
                if let Ok(Some(neighbors)) = hg.get_neighbors(now,order,size) {


                    for neighbor in neighbors.iter() {
                        if !visited.contains(neighbor) {
                            queue.push_back((*neighbor, depth + 1));
                            visited.insert(*neighbor);
                        }
                    }
                }
            }
        }
    }

    visited
}

/// `type Node = i64`
///
/// Depth-First-Search of the hypergraph starting from a given node.   
///
/// # Parameters  
/// - `hg` : `&Hypergraph` - The hypergraph to search.
/// - `start` : `Node` - The node to start the search from.
/// - `max_depth` : `Option<usize>` - `Some` maximum depth for the search. If `None` the search is not limited.
/// - `order` : `Option<usize>` - `Some` order of the hyperedges to consider. If `None` all hyperedges are considered.
/// - `size` : `Option<usize>` - `Some` size of the hyperedges to consider. If `None` all hyperedges are considered.
///
/// # Returns
/// - `AHashSet<Node>` - The nodes visited during the search. If the length of the returned hashset is `0`, then it means 
/// that the node provided was not in the hypergraph.
///
/// # Performance
/// - `O(n*n*m)`, where `n` and `m` are the number of nodes and the number of hyperedges of the hypergraph, respectively.
pub fn _dfs(
    hg: &Hypergraph,
    start: Node,
    max_depth: Option<usize>,
    order: Option<usize>,
    size: Option<usize>,
) -> AHashSet<Node> {
    let mut visited = AHashSet::new();
    
    if hg.check_node(start) {
        compute_dfs(hg, start, max_depth, 0, order, size, &mut visited);
    }
    visited
}

/// Effectively computes the dfs of the hypergraph.
fn compute_dfs(
    hg: &Hypergraph,
    node: Node,
    max_depth: Option<usize>,
    depth: usize,
    order: Option<usize>,
    size: Option<usize>,
    visited: &mut AHashSet<Node>,
) {
    if ! visited.contains(&node) {
        visited.insert(node);

        if max_depth.map_or(true, |max| depth < max) {
            if let Ok(Some(neighbors)) = hg.get_neighbors(node, order, size) {
                for neighbor in neighbors.iter() {
                    compute_dfs(hg, *neighbor, max_depth, depth + 1, order, size, visited);
                }
            }
        }
    }
}


#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_bfs_0() {
        let mut hg = Hypergraph::new(true);
        hg.add_nodes(&vec![1,2,3,4,5,6]);
        
        hg.add_edge(&vec![1, 2]);
        hg.add_edge_weighted(&vec![2, 3, 4], 27.7);
        hg.add_edge(&vec![2, 3, 5]);
        hg.add_edge(&vec![4, 6, 5]);

        let result = _bfs(&hg, 1, None, None, None);
        let expected: AHashSet<Node> = [1, 2, 3, 4, 5, 6].iter().cloned().collect();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_bfs_cycle() {
        let edges = vec![vec![1,3,7], vec![2, 4,3], vec![5,6,4], vec![7,6,9], vec![3,9]];

        let hg = Hypergraph::from(&edges);

        let result = _bfs(&hg, 1, None, None, None);
        let expected: AHashSet<Node> = [1, 2, 3, 4, 5, 6, 7, 9].iter().cloned().collect();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_bfs_depth_limit() {
        let mut hg = Hypergraph::new(false);

        hg.add_edge(&vec![1, 2]);
        hg.add_edge(&vec![2, 3]);
        hg.add_edge(&vec![3, 4]);

        let result = _bfs(&hg, 1, Some(2), None, None);
        let expected: AHashSet<Node> = [1, 2, 3].iter().cloned().collect(); 

        assert_eq!(result, expected);
    }

    #[test]
    fn test_bfs_with_size() {
        let mut hg = Hypergraph::new(true);

        hg.add_edge(&vec![1, 2, 3]); 
        hg.add_edge(&vec![2, 5]);
        hg.add_edge_weighted(&vec![5, 1], 45.9);
        hg.add_edge_weighted(&vec![3, 4], 100.1);    

        let result = _bfs(&hg, 1, None, None, Some(2));
        let expected: AHashSet<Node> = [1, 5, 2].iter().cloned().collect();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_dfs_0() {
        let mut hg = Hypergraph::new(true);
        hg.add_nodes(&vec![1,2,3,4,5,6]);
        
        hg.add_edge(&vec![1, 2]);
        hg.add_edge_weighted(&vec![2, 3, 4], 27.7);
        hg.add_edge(&vec![2, 3, 5]);
        hg.add_edge(&vec![4, 6, 5]);

        let result = _dfs(&hg, 1, None, None, None);
        let expected: AHashSet<Node> = [1, 2, 3, 4, 5, 6].iter().cloned().collect();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_dfs_cycle() {
        let edges = vec![vec![1,3,7], vec![2, 4,3], vec![5,6,4], vec![7,6,9], vec![3,9]];

        let hg = Hypergraph::from(&edges);

        let result = _dfs(&hg, 1, None, None, None);
        let expected: AHashSet<Node> = [1, 2, 3, 4, 5, 6, 7, 9].iter().cloned().collect();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_dfs_depth_limit() {
        let mut hg = Hypergraph::new(false);

        hg.add_edge(&vec![1, 2]);
        hg.add_edge(&vec![2, 3]);
        hg.add_edge(&vec![3, 4]);

        let result = _dfs(&hg, 1, Some(2), None, None);
        let expected: AHashSet<Node> = [1, 2, 3].iter().cloned().collect(); 

        assert_eq!(result, expected);
    }

    #[test]
    fn test_dfs_with_size() {
        let mut hg = Hypergraph::new(true);

        hg.add_edge(&vec![1, 2, 3]); 
        hg.add_edge(&vec![2,5]);
        hg.add_edge_weighted(&vec![5, 1], 45.9);
        hg.add_edge_weighted(&vec![3, 4], 100.1);    

        let result = _dfs(&hg, 1, None, None, Some(2)); 
        let expected: AHashSet<Node> = [1, 5, 2].iter().cloned().collect();

        assert_eq!(result, expected);
    }
}
