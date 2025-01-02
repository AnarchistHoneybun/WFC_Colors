use petgraph::graph::{DiGraph, UnGraph};
use petgraph::Graph;
use std::hash::Hash;
use std::io::BufRead;
use std::path::Path;
use std::time::Instant;
use std::{fs, io};
use wfc_colors::{validate_coloring, wfc_color};

/// Reads a graph from a DIMACS-format file
fn read_graph_from_file(filename: &str) -> io::Result<(UnGraph<(), ()>, usize, usize)> {
    let file = fs::File::open(Path::new(filename))?;
    let reader = io::BufReader::new(file);
    let mut graph = Graph::new_undirected();
    let mut num_nodes = 0;
    let mut num_edges = 0;

    // First pass: create nodes
    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts[0] == "p" {
            num_nodes = parts[2].parse().unwrap();
            num_edges = parts[3].parse().unwrap();
            for _ in 0..num_nodes {
                graph.add_node(());
            }
            break;
        }
    }

    // Second pass: add edges
    let file = fs::File::open(Path::new(filename))?;
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts[0] == "e" {
            let v1: usize = parts[1].parse::<usize>().unwrap() - 1;
            let v2: usize = parts[2].parse::<usize>().unwrap() - 1;
            graph.add_edge(
                graph.node_indices().nth(v1).unwrap(),
                graph.node_indices().nth(v2).unwrap(),
                (),
            );
        }
    }

    Ok((graph, num_nodes, num_edges))
}

fn main() {
    let data_dir = "./data";
    let entries = fs::read_dir(data_dir).expect("Unable to read data directory");

    for entry in entries {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if path.is_file() && path.extension().map_or(false, |ext| ext == "col") {
            let file_name = path.file_name().unwrap().to_string_lossy();
            println!("Processing file: {}", file_name);

            // Read the graph
            match read_graph_from_file(path.to_str().unwrap()) {
                Ok((graph, num_nodes, num_edges)) => {
                    // Run the coloring algorithm and time it
                    let start_time = Instant::now();
                    match wfc_color(&graph) {
                        Ok(colors) => {
                            let duration = start_time.elapsed();

                            // Validate the coloring
                            let is_valid = validate_coloring(&graph, &colors);

                            // Calculate number of colors used
                            let k_value = colors.values().copied().max().unwrap_or(0);

                            // Print results
                            println!("Results for {}", file_name);
                            println!("Vertices: {}", num_nodes);
                            println!("Edges: {}", num_edges);
                            println!("k-value: {}", k_value);
                            println!("Runtime: {:?}", duration);
                            println!("Valid coloring: {}", is_valid);
                            println!("-------------------");
                        }
                        Err(e) => eprintln!("Error coloring graph {}: {}", file_name, e),
                    }
                }
                Err(e) => eprintln!("Error reading file {}: {}", file_name, e),
            }
        }
    }

    // Check rejection case for Directed Graph
    let mut graph = DiGraph::<(), ()>::new();
    let a = graph.add_node(());
    let b = graph.add_node(());
    let c = graph.add_node(());
    let d = graph.add_node(());

    graph.extend_with_edges([(a, b), (b, c), (c, d), (d, a), (a, c)]);

    // Try to color the graph
    match wfc_color(&graph) {
        Ok(colors) => {
            println!("Colors: {:?}", colors);
        }
        Err(e) => eprintln!("Error coloring graph: {}", e),
    }
}
