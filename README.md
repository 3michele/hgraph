# hgraph

A flexible and extensible Rust-based hypergraph library.

## Features

- **Weighted Hypergraphs**: Supports the creation of weighted hypergraphs for more nuanced representations.
- **Flexible Subgraph Creation**: Easily create subgraphs induced by selected nodes.
- **Efficient Node and Edge Operations**: Add, remove, and clear nodes and hyperedges as needed.
  
## Installation

To use this library, add it to your `Cargo.toml` file:

```toml
[dependencies]
hgraph = { git = "https://github.com/3michele/hgraph.git" }
```

## Usage 
Here is a basic example of how to use the `hgraph` library:

```rust
use hgraph::Hypergraph;

// Creates a new weighted hypergraph
let mut hypergraph = Hypergraph::new(true);

// Adds nodes
for i in 0..8 {
    hypergraph.add_node(i);
}

// Adds weighted hyperedges
hypergraph.add_edge_weighted(&vec![0, 2, 3, 4], 27.7);    
hypergraph.add_edge_weighted(&vec![0, 6, 7], 12.3);    
hypergraph.add_edge_weighted(&vec![2, 5], 69.0);    

// Removes a node
hypergraph.remove_node(7);

// Creates a subhypergraph
let mut subhypergraph = hypergraph.subhypergraph(&vec![0, 2, 4, 3, 5]);

// Prints the hypergraphs 
println!("{:?}", hypergraph);
println!("{:?}", subhypergraph);

// Clears the hypergraphs 
subhypergraph.clear();
hypergraph.clear();
```

## Project Background  
This library was created as part of my thesis project at University of Trento, focusing on the development and analysis of hypergraphs.   

## Supervisors 
- **Alberto Montresor** (**Quintino Francesco Lotito**);
- **Marco Patrignani**. 
