use hgraph::Hypergraph;

#[test]
pub fn test1() {
    let mut mat = Hypergraph::new(true);
    for i in 0..10 {
        mat.add_node(i);
    }

    mat.add_edge(&vec![0, 2, 5, 6]);
    mat.add_edge(&vec![3, 4, 5, 9]);
    mat.add_edge(&vec![10, 11, 5]);

    println!("mat = {:?}", mat);
}

#[test]
pub fn test2() {
    let mut edges = Vec::new();
    edges.push(vec![1, 3, 5]);
    edges.push(vec![1, 2, 4]);
    edges.push(vec![3, 4, 6]);

    let weights = vec![27.7_f64, 18.1, 2.7, 8.9];

    let mat = Hypergraph::from_weighted(&edges, &weights);

    println!("test2: {:?}", mat);
}
