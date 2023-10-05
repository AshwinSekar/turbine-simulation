use std::{collections::HashSet, convert::identity};

use clap::{App, crate_name, crate_description, value_t_or_exit, Arg};
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
    let matches = App::new(crate_name!())
        .about(crate_description!())
        .arg(
            Arg::with_name("online_percent")
            .long("online-percent")
            .takes_value(true)
            .default_value("66")
            .help("Percentage of nodes that are online for the simulation")
        )
        .arg(
            Arg::with_name("malicious_percent")
            .long("malicious-percent")
            .takes_value(true)
            .default_value("0")
            .help("Percentage of nodes that are malicious, will only be sampled from online nodes")
        )
        .arg(
            Arg::with_name("threads")
            .long("threads")
            .takes_value(true)
            .default_value("10")
            .help("Number of threads")
        )
        .arg(
            Arg::with_name("trials")
            .long("trials")
            .takes_value(true)
            .default_value("1000")
            .help("Number of trials to simulate")
        ).get_matches();
    let online_p = value_t_or_exit!(matches.value_of("online_percent"), f64) / 100.0;
    let _threads = value_t_or_exit!(matches.value_of("threads"), usize);
    let trials = value_t_or_exit!(matches.value_of("trials"), usize);
    let malicious_p = value_t_or_exit!(matches.value_of("malicious_percent"), f64) / 100.0;
    turbine_recovery(trials, online_p, malicious_p);
}

fn turbine_recovery(trials: usize, online_p: f64, malicious_p: f64) {
    let online_nodes: usize = (online_p * (NUM_NODES as f64)) as usize;
    let malicious_nodes: usize = (malicious_p * (NUM_NODES as f64)) as usize;
    println!(
        "Running simulation with {} / {} nodes online of which {} are malicious,
        {L1_SIZE} l1 nodes,
        {L2_NODES} l2 nodes in neighborhoods of {L2_SIZE}",
        online_nodes,
        NUM_NODES,
        malicious_nodes,
    );
    let online = |node: usize| node < online_nodes;
    // Malicious nodes are always online
    let malicious = |node: usize| node < malicious_nodes;
    let mut results: Vec<f64> = vec![];
    let mut non_malicious_results: Vec<f64> = vec![];
    for block in 1..trials {
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
                    // If it's online and has the shred or it's malicious
                    if (online(l1_node) && nodes[l1_node].shreds[shred]) || malicious(l1_node) {
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
            // println!(
            //     "    round {} potentially recovering {}",
            //     rounds,
            //     might_recover.len()
            // );
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
            // println!(
            //     "    round {} recovered {recovered} median shreds received {}",
            //     rounds,
            //     median(&sizes)
            // );
        }

        // How many nodes recovered the block
        let mut recovered = 0;
        let mut non_malicious_recovered = 0;
        for node in 0..NUM_NODES {
            if nodes[node]
                .shreds
                .into_iter()
                .take(DATA_SHREDS)
                .all(identity) || malicious(node)
            {
                recovered += 1;
                if !malicious(node) {
                    non_malicious_recovered += 1;
                }
            }
        }
        results.push((recovered as f64) / (NUM_NODES as f64));
        non_malicious_results.push((non_malicious_recovered as f64) / (NUM_NODES as f64));
    }
    println!(
        "Median recovered: {}, Median non malicious recovered: {} Mean recovered: {} Mean non malicious recovered: {}",
        median(&results),
        median(&non_malicious_results),
        mean(&results),
        mean(&non_malicious_results),
    );
}
