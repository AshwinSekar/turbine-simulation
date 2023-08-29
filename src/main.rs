use std::{collections::HashSet, convert::identity};

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use statistical::{mean, median};

const BATCH_SIZE: usize = 64;
const RECOVER_SIZE: usize = 32;
const DATA_SHREDS: usize = RECOVER_SIZE;
const NUM_NODES: usize = 10_000;
const L1_SIZE: usize = 200;
const L2_NODES: usize = NUM_NODES - L1_SIZE - 1;
const L2_SIZE: usize = L2_NODES / L1_SIZE;

#[derive(Clone, Copy)]
struct Node {
    shreds: [bool; BATCH_SIZE],
}

impl Default for Node {
    fn default() -> Self {
        Node {
            shreds: [false; BATCH_SIZE],
        }
    }
}

fn main() {
    // turbine_recovery(0.25);
    // turbine_recovery(0.33);
    // turbine_recovery(0.5);
    turbine_recovery(0.66);
    // turbine_recovery(0.75);
    // turbine_recovery(1.0);
}

fn turbine_recovery(p: f64) {
    let online_nodes: usize = (p * (NUM_NODES as f64)) as usize;
    println!("Running simulation with {} / {} nodes online, {L1_SIZE} l1 nodes, {L2_NODES} l2 nodes in neighborhoods of {L2_SIZE}", online_nodes, NUM_NODES);
    let online = |node: usize| node < online_nodes;
    let mut results: Vec<f64> = vec![];
    for block in 1..1_000_000 {
        let mut nodes = [Node::default(); NUM_NODES];
        let mut rounds = 0;
        loop {
            let mut might_recover: HashSet<usize> = HashSet::default();
            // Only need to transmit coding shreds once as they are not recovered
            let shreds = if rounds == 0 { BATCH_SIZE } else { DATA_SHREDS };
            for shred in 0..shreds {
                let mut rng = ChaCha8Rng::seed_from_u64(shred as u64 * block as u64);
                let mut index: Vec<usize> = (0..NUM_NODES).into_iter().collect();
                index.shuffle(&mut rng);

                // If root is online, root + online L1 should get the shred
                let root = index[0];
                if online(root) {
                    for &node in &index[0..(1 + L1_SIZE)] {
                        if online(node) && !nodes[node].shreds[shred] {
                            nodes[node].shreds[shred] = true;
                            might_recover.insert(node);
                        }
                    }
                }

                // L1 transmits to L2
                for i in 1..(1 + L1_SIZE) {
                    let l1_node = index[i];
                    // If it's online and has the shred
                    if online(l1_node) && nodes[l1_node].shreds[shred] {
                        let l2_start = (1 + L1_SIZE) + i * L2_SIZE;
                        for &l2_node in &index[l2_start..(l2_start + L2_SIZE)] {
                            if online(l2_node) && !nodes[l2_node].shreds[shred] {
                                nodes[l2_node].shreds[shred] = true;
                                might_recover.insert(l2_node);
                            }
                        }
                    }
                }
            }

            rounds += 1;

            if might_recover.is_empty() {
                break;
            }
            println!(
                "    round {} potentially recovering {}",
                rounds,
                might_recover.len()
            );
            let mut sizes = vec![];
            let mut recovered = 0;
            for node in might_recover {
                assert!(online(node));
                sizes.push(nodes[node].shreds.into_iter().filter(|s| *s).count());
                if nodes[node].shreds.into_iter().filter(|s| *s).count() >= RECOVER_SIZE {
                    recovered += 1;
                    for i in 0..DATA_SHREDS {
                        nodes[node].shreds[i] = true;
                    }
                }
            }
            println!(
                "    round {} recovered {recovered} median shreds received {}",
                rounds,
                median(&sizes)
            );
        }

        // How many L2 nodes recovered the block
        let mut l2_recovered = 0;
        for node in (1 + L1_SIZE)..NUM_NODES {
            if nodes[node]
                .shreds
                .into_iter()
                .take(DATA_SHREDS)
                .all(identity)
            {
                l2_recovered += 1;
            }
        }
        results.push((l2_recovered as f64) / (L2_NODES as f64));
        println!(
            "{p}: Median {} Mean {} Rounds {rounds}",
            median(&results),
            mean(&results)
        );
    }
}
