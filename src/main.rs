extern crate anyhow;
extern crate bincode;
extern crate bitvec;
extern crate clap;
extern crate comparator;
extern crate flate2;
extern crate kdam;
extern crate lru;
extern crate memmap2;
extern crate rusty_dawg;
extern crate serde;
extern crate serde_json;
extern crate substring;
extern crate tempfile;
extern crate tokenizers;
extern crate unicode_segmentation;

mod build_cdawg;
mod build_stats;
mod cdawg;
mod data_reader;
mod dawg;
mod evaluator;
mod graph;
mod io;
mod memory_backing;
mod stat_utils;
mod tokenize;
mod weight;

use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::cmp::Ord;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::fmt::Debug;

use io::Save;

use clap::Parser;
use std::fs;
use std::mem::size_of;

use kdam::{tqdm, BarExt};

use crate::build_cdawg::build_cdawg;
use crate::dawg::Dawg;
use crate::evaluator::Evaluator;

use crate::graph::avl_graph::edge::AvlEdge;
use crate::graph::avl_graph::node::AvlNode;
use crate::graph::indexing::DefaultIx;
use crate::memory_backing::{CacheConfig, DiskBacking, MemoryBacking, RamBacking};

use crate::data_reader::{DataReader, PileReader, TxtReader};
use crate::tokenize::{NullTokenIndex, PretrainedTokenizer, TokenIndex, Tokenize};
use crate::weight::DefaultWeight;

// Node and edge weight types.
type N = DefaultWeight;

#[derive(Parser, Debug)]
#[command(
author = "William Merrill <willm@nyu.edu>",
version, about, long_about = None,
)]
pub struct Args {
    /// Path to corpus DAWG is built on.
    #[arg(long)]
    train_path: String,

    /// Path to evaluation data. Optional.
    #[arg(long, default_value = "")]
    test_path: String,

    /// Where DAWG is saved. If saving to disk, will be treated as a directory; if
    /// serializing a RAM data structure, will be treated as a file.
    #[arg(long)]
    save_path: String,

    /// Path to save evaluation results.
    #[arg(long, default_value = "")]
    results_path: String,

    /// Tokenizer to use. This can be `whitespace` or any huggingface tokenizer, e.g.,
    /// `gpt2`, `bert-base-uncased`, etc.
    #[arg(long, default_value = "gpt2")]
    tokenizer: String,

    /// Specifies how to read data from `train_path`. This can be `txt`, `pile`, or
    /// `jsonl`.
    #[arg(long, default_value = "txt")]
    data_reader: String,

    /// Datatype used to represent tokens in a DAWG (no effect for CDAWG). Can be
    /// `u16`, `u32`, or `usize`.
    #[arg(long, default_value = "u16")]
    utype: String,

    /// Truncate evaluation data to this many tokens.
    #[arg(long, default_value_t = 0)]
    truncate_test: usize,

    /// Number of tokens to wait before evaluating.
    #[arg(long, default_value_t = 0)]
    n_eval: usize,

    /// Maximum suffix length to track when computing evaluation metrics.
    #[arg(long, default_value_t = 10)]
    max_length: u64,

    /// Max length of a state in the DAWG.
    #[arg(long, default_value_t = -1)]
    max_state_length: i64,

    /// Token used to split documents when `data_reader` is `txt`.
    #[arg(long)]
    split_token: Option<String>,

    /// Estimate of the number of nodes to allocate, expressed as a ratio of the
    /// estimated total number of tokens (`n_tokens`).
    #[arg(long, default_value_t = 2.)]
    nodes_ratio: f64,

    /// Estimate of the number of edges to allocate, expressed as a ratio of the
    /// estimated total number of tokens (`n_tokens`).
    #[arg(long, default_value_t = 3.)]
    edges_ratio: f64,

    /// Estimate of the number of tokens, used to allocate DAWG.
    #[arg(long, default_value_t = 200000000)]
    n_tokens: usize,

    /// Number of states cached in RAM if building a DAWG on disk.
    #[arg(long, default_value_t = 0)]
    cache_size: usize,

    /// Amount of input to read, in bytes, at a time while consuming file.
    /// Defaults to 10 GB.
    #[arg(long, default_value_t = 10_000_000_000)]
    buf_size: usize,

    /// Don't add document boundaries between adjacent documents.
    #[arg(long, short, action)]
    single_string: bool,

    // CDAWG args.
    /// Build CDAWG instead of DAWG.
    #[arg(long, short, action)]
    cdawg: bool,

    /// Make CDAWG immutable after building. Only works with CDAWG, not DAWG
    #[arg(long, short, action)]
    immutable: bool,

    /// Path to store a vector of all tokens in training corpus.
    #[arg(long)]
    train_vec_path: Option<String>,

    /// Number of tokens to wait before computing CDAWG statistics.
    #[arg(long)]
    stats_threshold: Option<usize>,

    /// Path to save CDAWG computed statistics.
    #[arg(long)]
    stats_path: Option<String>,

    /// DiskVec path to use while traversing graph.
    #[arg(long)]
    count_path: Option<String>,

    /// Don't add counts.
    #[arg(long)]
    no_counts: bool,

    /// Build DAWG in RAM instead of on disk.
    #[arg(long)]
    ram: bool,
    // FIXME: Below is causing issues, for whatever reason.
    // Special arguments for JsonReader (not used for Pile).
    // #[arg(long, default_value = "text")]
    // jsonl_text_key: String,
    // #[arg(long, default_value = "split")]
    // jsonl_domain_key: String,
}

impl Args {
    pub fn get_cache_config(&self) -> CacheConfig {
        // TODO: Generalize CacheConfig to store size info as well?
        let nodes_ratio = self.nodes_ratio / (self.nodes_ratio + self.edges_ratio);
        let edges_ratio = self.edges_ratio / (self.nodes_ratio + self.edges_ratio);
        CacheConfig {
            node_cache_size: (nodes_ratio * (self.cache_size as f64)).ceil() as usize,
            edge_cache_size: (edges_ratio * (self.cache_size as f64)).ceil() as usize,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.immutable && !args.cdawg {
        panic!("--immutable can only be used with --cdawg");
    }

    if args.cdawg {
        if args.ram {
            println!("Building CDAWG in RAM but saving on disk...");
            type Mb = RamBacking<N, (DefaultIx, DefaultIx), DefaultIx>;
            let mb = Mb::default();
            return Ok(build_cdawg::<Mb>(args, mb)?);
        }
        println!("Building CDAWG on disk...");
        type Mb = DiskBacking<N, (DefaultIx, DefaultIx), DefaultIx>;
        let mb = Mb::new(args.save_path.clone());
        return Ok(build_cdawg::<Mb>(args, mb)?);
    }

    // Messy, but it works.
    if args.utype == "u16" {
        type E = u16;
        if args.ram {
            type Mb = RamBacking<N, E, DefaultIx>;
            let mb = Mb::default();
            run_rusty_dawg::<E, Mb>(args, mb)
        } else {
            type Mb = DiskBacking<N, E, DefaultIx>;
            let mb = Mb::new(args.save_path.clone());
            run_rusty_dawg::<E, Mb>(args, mb)
        }
    } else if args.utype == "u32" {
        type E = u32;
        if args.ram {
            type Mb = RamBacking<N, E, DefaultIx>;
            let mb = Mb::default();
            run_rusty_dawg::<E, Mb>(args, mb)
        } else {
            type Mb = DiskBacking<N, E, DefaultIx>;
            let mb = Mb::new(args.save_path.clone());
            run_rusty_dawg::<E, Mb>(args, mb)
        }
    } else if args.utype == "usize" {
        type E = usize;
        if args.ram {
            type Mb = RamBacking<N, E, DefaultIx>;
            let mb = Mb::default();
            run_rusty_dawg::<E, Mb>(args, mb)
        } else {
            type Mb = DiskBacking<N, E, DefaultIx>;
            let mb = Mb::new(args.save_path.clone());
            run_rusty_dawg::<E, Mb>(args, mb)
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
        + TryFrom<u32>
        + tokenize::end::End,
    usize: TryFrom<E>,
    u64: TryFrom<E>,
    Mb: MemoryBacking<N, E, DefaultIx>,
    <E as TryFrom<usize>>::Error: Debug,
    Dawg<E, N, DefaultIx, Mb>: io::Save,
{
    println!("sizeof(Ix) {}B", size_of::<DefaultIx>());
    println!("sizeof(N) {}B", size_of::<N>());
    println!("sizeof(E) {}B", size_of::<E>());
    println!("sizeof(Node): {}B", size_of::<AvlNode<N, DefaultIx>>());
    println!("sizeof(Edge): {}B", size_of::<AvlEdge<E, DefaultIx>>());

    let mut index: Box<dyn Tokenize<E>> = if args.tokenizer == "whitespace" {
        Box::new(TokenIndex::new())
    } else if args.tokenizer == "null" {
        Box::new(NullTokenIndex::new())
    } else {
        Box::new(PretrainedTokenizer::new(&args.tokenizer))
    };

    let train_file = fs::File::open(args.train_path.as_str())?;
    let n_bytes = train_file.metadata().unwrap().len();
    let eval_threshold = if args.n_eval == 0 {
        0
    } else {
        args.n_tokens / args.n_eval
    };
    let buf_size: usize = min(n_bytes.try_into().unwrap(), args.buf_size);
    let reader: Box<DataReader> = if args.data_reader == "pile" {
        Box::new(PileReader::new(args.train_path.clone()).unwrap())
    } else {
        Box::new(TxtReader::new(
            train_file,
            buf_size,
            args.split_token.clone(),
        ))
    };

    let test_raw: String = if args.test_path.is_empty() {
        "".to_string()
    } else {
        let path = args.test_path.as_str();
        fs::read_to_string(path).unwrap_or_else(|_| panic!("Could not load test from {}", path))
    };
    index.build(&test_raw); // Either the tokenizer must be pretrained or test must contain all tokens!
    let doc_id_token = E::try_from(index.get_count()).unwrap(); // The token used to store document IDs.
    let mut test: Vec<E> = index.tokenize(&test_raw);
    let old_test_len = test.len();
    if args.truncate_test > 0 {
        test = test[0..args.truncate_test].to_vec();
    }
    let mut evaluator = Evaluator::new(&test, args.max_length);
    println!("#(test): {}/{}", test.len(), old_test_len);

    let n_nodes = (args.nodes_ratio * (args.n_tokens as f64)).ceil() as usize;
    let n_edges = (args.edges_ratio * (args.n_tokens as f64)).ceil() as usize;
    let cache_config = args.get_cache_config();
    let max_length: Option<u64> = if !args.max_state_length.is_negative() {
        Some(args.max_state_length.try_into().unwrap())
    } else {
        None
    };

    let mut dawg: Dawg<E, N, DefaultIx, Mb> =
        Dawg::with_capacity_mb(mb, max_length, n_nodes, n_edges, cache_config);

    let mut idx = 0;
    let mut last = dawg.get_initial();
    let mut length = 0;
    let mut pbar = tqdm!(total = args.n_tokens);
    for (doc_id, doc) in reader {
        let tokens = index.tokenize(doc.as_str());
        for token in &tokens {
            (last, length) = dawg.extend(*token, last, length);
            if eval_threshold != 0 && idx % eval_threshold == 0 && idx != 0 {
                println!("Evaluating...");
                evaluator.evaluate(&dawg, idx);
                if !args.results_path.is_empty() {
                    evaluator.to_json(&args.results_path)?;
                }
            }
            idx += 1;
            let _ = pbar.update(1);
        }
        (last, length) = dawg.end_document(last, doc_id_token, doc_id.try_into().unwrap());
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

    println!("Saving DAWG...");
    dawg.save(&args.save_path)?;
    println!("Successfully saved DAWG to {}!", &args.save_path);
    Ok(())
}
