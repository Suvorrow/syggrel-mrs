use syggrel_mrs::{GenomicTarget, mrs_algorithms::compute_topological_risk};

fn main() {
    let target = GenomicTarget {
        chromosome: "chr5".to_string(),
        position: 1_234_567,
        sequence: "ATCG".to_string(),
    };

    let risk = compute_topological_risk(&target);
    println!("Topological risk score: {}", risk);
}