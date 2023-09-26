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
mod tokenize;
mod weight;
mod stat_utils;

use serde::{Deserialize, Serialize};
use std::cmp::Ord;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::fmt::Debug;

use io::Save;

use clap::Parser;
use std::fs;
use std::mem::size_of;

use kdam::tqdm;

use dawg::Dawg;
use evaluator::Evaluator;

use graph::indexing::DefaultIx;
use graph::memory_backing::{MemoryBacking, DiskBacking, RamBacking};

use tokenize::{NullTokenIndex, PretrainedTokenizer, TokenIndex, Tokenize};
use weight::weight40::DefaultWeight;

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
    #[arg(long)]
    gen_path: Option<String>,
    #[arg(long)]
    gen_results_path: Option<String>,
    // #[arg(long, default_value_t = false)]
    // tokenize: bool,

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

    #[arg(long)]
    disk_path: Option<String>,

    #[arg(long, default_value_t = 2.)]
    nodes_ratio: f64,
    #[arg(long, default_value_t = 3.)]
    edges_ratio: f64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Messy, but it works.
    if args.utype == "u16" {
        type E = u16;
        match args.disk_path {
            Some(path) => {
                type Mb = DiskBacking<N, E, DefaultIx>;
                let mb = Mb::new(&path);
                run_rusty_dawg::<E, Mb>(args, mb)
            },
            None => {
                type Mb = RamBacking<N, E, DefaultIx>;
                let mb = Mb::default();
                run_rusty_dawg::<E, Mb>(args, mb)
            },
        }
    } else if args.utype == "u32" {
        type E = u32;
        match args.disk_path {
            Some(path) => {
                type Mb = DiskBacking<N, E, DefaultIx>;
                let mb = Mb::new(&path);
                run_rusty_dawg::<E, Mb>(args, mb)
            },
            None => {
                type Mb = RamBacking<N, E, DefaultIx>;
                let mb = Mb::default();
                run_rusty_dawg::<E, Mb>(args, mb)
            },
        }
    } else if args.utype == "usize" {
        type E = usize;
        match args.disk_path {
            Some(path) => {
                type Mb = DiskBacking<N, E, DefaultIx>;
                let mb = Mb::new(&path);
                run_rusty_dawg::<E, Mb>(args, mb)
            },
            None => {
                type Mb = RamBacking<N, E, DefaultIx>;
                let mb = Mb::default();
                run_rusty_dawg::<E, Mb>(args, mb)
            },
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
{
    println!("sizeof(edge): {}B", size_of::<E>());
    println!("sizeof(node): {}B", size_of::<N>());

    let mut index: Box<dyn Tokenize<E>> = if args.tokenizer == "whitespace" {
        Box::new(TokenIndex::new())
    } else if args.tokenizer == "null" {
        Box::new(NullTokenIndex::new())
    } else {
        Box::new(PretrainedTokenizer::new(&args.tokenizer))
    };

    let train_raw: String =
        fs::read_to_string(args.train_path.as_str()).expect("Error loading train");
    index.build(&train_raw);
    let train: Vec<E> = index.tokenize(&train_raw);
    let eval_threshold = if args.n_eval != 0 {
        train.len() / args.n_eval
    } else {
        0
    };
    println!("#(train): {}", train.len());

    let test_raw: String = fs::read_to_string(args.test_path.as_str()).expect("Error loading test");
    let mut test: Vec<E> = index.tokenize(&test_raw);
    // let mut test: Vec<usize> = test_raw.split_whitespace().map(|x| index.add(x)).collect();
    let old_test_len = test.len();
    if args.truncate_test > 0 {
        test = test[0..args.truncate_test].to_vec();
    }
    println!("#(test): {}/{}", test.len(), old_test_len);

    let gen_raw: String = match &args.gen_path {
        Some(path) => {
            fs::read_to_string(path).unwrap_or_else(|_| panic!("Error loading gen path: {}", path))
        }
        None => "".to_string(),
    };
    let gen: Vec<E> = index.tokenize(&gen_raw);
    println!("#(gen): {}", gen.len());
    println!("#(vocab): {}", index.get_count());

    let mut evaluator = Evaluator::new(&test, args.max_length);
    let mut gen_evaluator = Evaluator::new(&gen, args.max_length);

    let n_nodes = (args.nodes_ratio * (train.len() as f64)).ceil() as usize;
    let n_edges = (args.edges_ratio * (train.len() as f64)).ceil() as usize;
    let mut dawg: Dawg<E, N, DefaultIx, Mb> = Dawg::with_capacity_mb(mb, n_nodes, n_edges);

    let mut last = dawg.get_initial();
    for (idx, token) in tqdm!(train.iter()).enumerate() {
        last = dawg.extend(*token, last);
        if eval_threshold != 0 && idx % eval_threshold == 0 && idx != 0 {
            evaluator.evaluate(&dawg, idx);
            if !args.results_path.is_empty() {
                evaluator.to_json(&args.results_path)?;
            }
            match &args.gen_results_path {
                Some(gen_path) => {
                    gen_evaluator.evaluate(&dawg, idx);
                    gen_evaluator.to_json(gen_path)?;
                }
                None => {}
            }
        }
    }
    println!("Completed!");
    println!(
        "  Node ratio: {:.2} (total={})",
        (dawg.node_count() as f64) / (train.len() as f64),
        dawg.node_count()
    );
    println!(
        "  Edge ratio: {:.2} (total={})",
        (dawg.edge_count() as f64) / (train.len() as f64),
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
