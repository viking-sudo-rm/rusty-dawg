// Driver to build a CDAWG on a corpus.
// Eventually, this should probably be merged with main.

use anyhow::Result;

use std::cell::RefCell;
use std::cmp::min;

use std::convert::TryInto;

use std::rc::Rc;

use std::fs;
use std::mem::size_of;

use kdam::{tqdm, BarExt};

use super::Args;

use crate::build_stats::BuildStats;
use crate::cdawg::token_backing::TokenBacking;
use crate::cdawg::Cdawg;
use crate::cdawg::TopologicalCounter;
use crate::data_reader::{DataReader, JsonlReader, PileReader, TxtReader};
use crate::graph::avl_graph::edge::Edge;
use crate::graph::avl_graph::node::Node;
use crate::graph::indexing::DefaultIx;
use crate::io;
use crate::io::Save;
use crate::memory_backing::{DiskVec, MemoryBacking};
use crate::tokenize::{NullTokenIndex, PretrainedTokenizer, TokenIndex, Tokenize};

type N = super::N;

pub fn build_cdawg<Mb>(args: Args, mb: Mb) -> Result<()>
where
    Mb: MemoryBacking<N, (DefaultIx, DefaultIx), DefaultIx>,
    Cdawg<N, DefaultIx, Mb>: io::Save,
{
    // TODO: Support token types with more bits?
    let mut index: Box<dyn Tokenize<u16>> = if args.tokenizer == "whitespace" {
        Box::new(TokenIndex::new())
    } else if args.tokenizer == "null" {
        Box::new(NullTokenIndex::new())
    } else {
        let mut pt = PretrainedTokenizer::new(&args.tokenizer);
        pt.add_eos = true;
        Box::new(pt)
    };

    println!("==========");
    println!("Sizes");
    println!("==========");
    println!("  Ix: {}B", size_of::<DefaultIx>());
    println!("  N: {}B", size_of::<N>());
    println!("  E: {}B", size_of::<(DefaultIx, DefaultIx)>());
    println!("  Node: {}B", size_of::<Node<N, DefaultIx>>());
    println!(
        "  Edge: {}B",
        size_of::<Edge<(DefaultIx, DefaultIx), DefaultIx>>()
    );
    println!();

    println!("Opening train file...");
    let train_file = fs::File::open(args.train_path.as_str())?;
    let n_bytes = train_file.metadata().unwrap().len();
    let buf_size: usize = min(n_bytes.try_into().unwrap(), args.buf_size);
    println!("Buffer size: {}B", args.buf_size);

    let reader: Box<DataReader> = if args.data_reader == "pile" {
        Box::new(PileReader::new(args.train_path.clone()).unwrap())
    } else if args.data_reader == "jsonl" {
        Box::new(JsonlReader::new(args.train_path.clone(), "text".to_string(), None).unwrap())
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

    println!("Cache size: {}", args.cache_size);
    let n_nodes = (args.nodes_ratio * (args.n_tokens as f64)).ceil() as usize;
    let n_edges = (args.edges_ratio * (args.n_tokens as f64)).ceil() as usize;
    let cache_config = args.get_cache_config();
    let _max_length: Option<u64> = if !args.max_state_length.is_negative() {
        Some(args.max_state_length.try_into().unwrap())
    } else {
        None
    };

    // Maintain a DiskVec that we update incrementally (whenever we read a token, set it).
    println!("# tokens: {}", args.n_tokens);
    println!("Creating train vector...");
    let train_vec: Rc<RefCell<dyn TokenBacking<u16>>> = match &args.train_vec_path {
        Some(ref train_vec_path) => {
            let disk_vec = DiskVec::new(train_vec_path, args.n_tokens)?;
            Rc::new(RefCell::new(disk_vec))
        }
        None => {
            println!("Storing tokens vector in RAM!");
            let vec = Vec::with_capacity(args.n_tokens);
            Rc::new(RefCell::new(vec))
        }
    };

    println!("Allocating CDAWG...");
    let mut cdawg: Cdawg<N, DefaultIx, Mb> =
        Cdawg::with_capacity_mb(train_vec.clone(), mb, n_nodes, n_edges, cache_config);

    let mut idx: usize = 0;
    let mut pbar = tqdm!(total = args.n_tokens);
    let (mut state, mut start) = (cdawg.get_source(), 1);
    for (doc_id, doc) in reader {
        let tokens = index.tokenize(doc.as_str());
        for token in &tokens {
            idx += 1;
            train_vec.borrow_mut().push(*token);
            (state, start) = cdawg.update(state, start, idx);
            if *token == u16::MAX {
                (state, start) = cdawg.end_document(idx, doc_id);
            }
            let _ = pbar.update(1);

            if let Some(stats_threshold) = args.stats_threshold {
                if (idx + 1) % stats_threshold == 0 {
                    let stats = BuildStats::from_cdawg(&cdawg, idx, n_bytes, pbar.elapsed_time());
                    let npt = stats.get_nodes_per_token();
                    let ept = stats.get_edges_per_token();
                    pbar.set_description(format!("n/t: {:.2}, e/t: {:.2}", npt, ept));
                    if let Some(ref stats_path) = args.stats_path {
                        stats.append_to_jsonl(stats_path)?;
                    }
                }
            }
        }
    }
    eprintln!();

    println!("\nFilling counts...");
    if !args.no_counts {
        match args.count_path {
            Some(ref count_path) => {
                let mut counter = TopologicalCounter::new_disk(count_path, idx)?;
                counter.fill_counts(&mut cdawg);
            }
            None => {
                let mut counter = TopologicalCounter::new_ram();
                counter.fill_counts(&mut cdawg);
            }
        }
    }

    let stats = BuildStats::from_cdawg(&cdawg, idx, n_bytes, pbar.elapsed_time());
    if let Some(ref stats_path) = args.stats_path {
        stats.append_to_jsonl(stats_path)?;
    }
    println!();
    println!("==========");
    println!("Completed!");
    println!("==========");
    println!("  # tokens: {}", idx);
    println!("  # nodes: {}", stats.n_nodes);
    println!("  # edges: {}", stats.n_edges);
    println!("  tokens/byte: {:.2}", stats.get_tokens_per_byte());
    println!("  nodes/token: {:.2}", stats.get_nodes_per_token());
    println!("  edge/token: {:.2}", stats.get_edges_per_token());
    println!("  balance ratio: {:.2}", stats.balance_ratio);
    println!();

    // TODO: Simplify this logic and the associated flags.
    if !args.save_path.is_empty() {
        println!("Saving DAWG...");
        let _ = cdawg.save(&args.save_path);
        println!("Successfully saved DAWG to {}!", &args.save_path);
    } else if let Some(disk_path) = args.disk_path {
        let _ = cdawg.save(disk_path.as_str());
    }
    Ok(())
}
