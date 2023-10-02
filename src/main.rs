// Implementation of Suffix DFA in Rust.
//
// See here for Graph info:
// https://docs.rs/petgraph/latest/petgraph/graph/struct.Graph.html
//
// See here for Suffix Automaton algorithm in Python:
// https://github.com/viking-sudo-rm/knn-transformers/blob/master/src/suffix_dfa_builder.py
//

extern crate anyhow;
extern crate bincode;
extern crate bitvec;
extern crate clap;
extern crate kdam;
extern crate memmap2;
extern crate rusty_dawg;
extern crate serde;
extern crate serde_json;
extern crate substring;
extern crate tempfile;
extern crate tokenizers;
extern crate unicode_segmentation;

mod dawg;
mod evaluator;
mod graph;
mod io;
mod stat_utils;
mod tokenize;
mod weight;

use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::cmp::Ord;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::fmt::Debug;
use std::io::{BufReader, Read};

use io::Save;

use clap::Parser;
use std::fs;
use std::mem::size_of;

use kdam::{tqdm, BarExt};

use dawg::Dawg;
use evaluator::Evaluator;

use graph::avl_graph::edge::Edge;
use graph::avl_graph::node::Node;
use graph::indexing::DefaultIx;
use graph::memory_backing::{DiskBacking, MemoryBacking, RamBacking};

use tokenize::{NullTokenIndex, PretrainedTokenizer, TokenIndex, Tokenize};
use weight::DefaultWeight;

// Node and edge weight types.
type N = DefaultWeight;

#[derive(Parser, Debug)]
#[command(
author = "William Merrill <willm@nyu.edu>",
version, about, long_about = None,
)]
struct Args {
    #[arg(long)]
    train_path: String,
    #[arg(long)]
    test_path: String,
    #[arg(long)]
    save_path: String,
    #[arg(long)]
    results_path: String,

    // This value can take on the following values:
    // `whitespace`, and every huggingface tokenizer, e.g. `gpt2`, `bert-base-uncased`, etc.
    #[arg(long)]
    tokenizer: String,

    #[arg(long, default_value = "u32")]
    utype: String,

    #[arg(long, default_value_t = 10000)]
    truncate_test: usize,
    #[arg(long, default_value_t = 20)]
    n_eval: usize,
    #[arg(long, default_value_t = 10)]
    max_length: u64,
    // Max length of a state in the Dawg.
    #[arg(long, default_value_t = -1)]
    max_state_length: i64,

    #[arg(long)]
    disk_path: Option<String>,

    #[arg(long, default_value_t = 2.)]
    nodes_ratio: f64,
    #[arg(long, default_value_t = 3.)]
    edges_ratio: f64,
    #[arg(long, default_value_t = 0.33)]
    tokens_per_byte: f64,

    // Amount of input to read at a time while consuming file. Defaults to 10 GB.
    #[arg(long, default_value_t = 10000000000)]
    buf_size: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match args.disk_path.clone() {
        Some(path) => println!("DAWG on disk: {}", path),
        None => println!("DAWG in RAM"),
    };

    // Messy, but it works.
    if args.utype == "u16" {
        type E = u16;
        match args.disk_path.clone() {
            Some(path) => {
                type Mb = DiskBacking<N, E, DefaultIx>;
                let mb = Mb::new(path);
                run_rusty_dawg::<E, Mb>(args, mb)
            }
            None => {
                type Mb = RamBacking<N, E, DefaultIx>;
                let mb = Mb::default();
                run_rusty_dawg::<E, Mb>(args, mb)
            }
        }
    } else if args.utype == "u32" {
        type E = u32;
        match args.disk_path.clone() {
            Some(path) => {
                type Mb = DiskBacking<N, E, DefaultIx>;
                let mb = Mb::new(path);
                run_rusty_dawg::<E, Mb>(args, mb)
            }
            None => {
                type Mb = RamBacking<N, E, DefaultIx>;
                let mb = Mb::default();
                run_rusty_dawg::<E, Mb>(args, mb)
            }
        }
    } else if args.utype == "usize" {
        type E = usize;
        match args.disk_path.clone() {
            Some(path) => {
                type Mb = DiskBacking<N, E, DefaultIx>;
                let mb = Mb::new(path);
                run_rusty_dawg::<E, Mb>(args, mb)
            }
            None => {
                type Mb = RamBacking<N, E, DefaultIx>;
                let mb = Mb::default();
                run_rusty_dawg::<E, Mb>(args, mb)
            }
        }
    } else {
        panic!("Invalid usize type: {}", args.utype);
    }
}

fn run_rusty_dawg<E, Mb>(args: Args, mb: Mb) -> Result<(), Box<dyn std::error::Error>>
where
    E: Eq
        + Ord
        + Serialize
        + for<'a> Deserialize<'a>
        + Copy
        + Debug
        + TryInto<usize>
        + TryFrom<usize>
        + 'static
        + TryInto<u32>
        + TryFrom<u32>,
    usize: TryFrom<E>,
    Mb: MemoryBacking<N, E, DefaultIx>,
    Dawg<E, N, DefaultIx, Mb>: io::Save,
{
    println!("sizeof(Ix) {}B", size_of::<DefaultIx>());
    println!("sizeof(N) {}B", size_of::<N>());
    println!("sizeof(E) {}B", size_of::<E>());
    println!("sizeof(Node): {}B", size_of::<Node<N, DefaultIx>>());
    println!("sizeof(Edge): {}B", size_of::<Edge<E, DefaultIx>>());

    let mut index: Box<dyn Tokenize<E>> = if args.tokenizer == "whitespace" {
        Box::new(TokenIndex::new())
    } else if args.tokenizer == "null" {
        Box::new(NullTokenIndex::new())
    } else {
        Box::new(PretrainedTokenizer::new(&args.tokenizer))
    };

    let train_file = fs::File::open(args.train_path.as_str())?;
    let n_bytes = train_file.metadata().unwrap().len();
    let est_n_tokens = (args.tokens_per_byte * (n_bytes as f64)).round() as usize;
    let eval_threshold = if args.n_eval == 0 {
        0
    } else {
        est_n_tokens / args.n_eval
    };
    let buf_size: usize = min(n_bytes.try_into().unwrap(), args.buf_size);

    let test_raw: String = fs::read_to_string(args.test_path.as_str()).expect("Error loading test");
    index.build(&test_raw); // Either the tokenizer must be pretrained or test must contain all tokens!
    let mut test: Vec<E> = index.tokenize(&test_raw);
    // let mut test: Vec<usize> = test_raw.split_whitespace().map(|x| index.add(x)).collect();
    let old_test_len = test.len();
    if args.truncate_test > 0 {
        test = test[0..args.truncate_test].to_vec();
    }
    let mut evaluator = Evaluator::new(&test, args.max_length);
    println!("#(test): {}/{}", test.len(), old_test_len);

    let n_nodes = (args.nodes_ratio * (est_n_tokens as f64)).ceil() as usize;
    let n_edges = (args.edges_ratio * (est_n_tokens as f64)).ceil() as usize;
    let max_length: Option<u64> = if !args.max_state_length.is_negative() {
        Some(args.max_state_length.try_into().unwrap())
    } else {
        None
    };

    let mut dawg: Dawg<E, N, DefaultIx, Mb> =
        Dawg::with_capacity_mb(mb, max_length, n_nodes, n_edges);

    let mut idx = 0;
    let mut last = dawg.get_initial();
    let mut length = 0;
    let mut pbar = tqdm!(total = est_n_tokens);
    let mut train_reader = BufReader::with_capacity(buf_size, train_file);
    let mut buffer = vec![0; buf_size];
    loop {
        let n_bytes_read = train_reader.read(&mut buffer).unwrap();
        if n_bytes_read == 0 {
            break;
        }
        let text = std::str::from_utf8(&buffer);
        let tokens = index.tokenize(text.unwrap());
        for token in &tokens {
            (last, length) = dawg.extend(*token, last, length);
            if eval_threshold != 0 && idx % eval_threshold == 0 && idx != 0 {
                evaluator.evaluate(&dawg, idx);
                if !args.results_path.is_empty() {
                    evaluator.to_json(&args.results_path)?;
                }
            }
            idx += 1;
            pbar.update(1);
        }
    }

    eprintln!();
    println!("Completed!");
    println!(
        "  token/byte: {:.2} (tokens={})",
        (idx as f64) / (n_bytes as f64),
        idx
    );
    println!(
        "  node/token: {:.2} (nodes={})",
        (dawg.node_count() as f64) / (idx as f64),
        dawg.node_count()
    );
    println!(
        "  edge/token: {:.2} (edges={})",
        (dawg.edge_count() as f64) / (idx as f64),
        dawg.edge_count()
    );
    println!("  Balance ratio: {}", dawg.balance_ratio(1));

    if !args.save_path.is_empty() {
        println!("Saving DAWG...");
        dawg.save(&args.save_path)?;
        println!("Successfully saved DAWG to {}!", &args.save_path);
    }
    Ok(())
}
