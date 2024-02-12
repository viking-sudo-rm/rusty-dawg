// Driver to build a CDAWG on a corpus.
// Eventually, this should probably be merged with main.

use anyhow::Result;

use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::cmp::Ord;
use std::convert::TryInto;
use std::io::{BufReader, Read};
use std::rc::Rc;
use std::cell::RefCell;

use std::fs;
use std::mem::size_of;

use kdam::{tqdm, BarExt};

use super::Args;

use crate::io;
use crate::io::Save;
use crate::data_reader::{DataReader, PileReader, TxtReader};
use crate::cdawg::Cdawg;
use crate::cdawg::cdawg_edge_weight::CdawgEdgeWeight;
use crate::graph::avl_graph::edge::Edge;
use crate::graph::avl_graph::node::Node;
use crate::graph::indexing::DefaultIx;
use crate::graph::memory_backing::{DiskBacking, MemoryBacking, RamBacking};
use crate::graph::memory_backing::disk_backing::disk_vec::DiskVec;
use crate::build_stats::BuildStats;
use crate::tokenize::{NullTokenIndex, PretrainedTokenizer, TokenIndex, Tokenize};

type N = super::N;
type E = CdawgEdgeWeight<DefaultIx>;

// Confusingly, E here is the token type.
pub fn build_cdawg<Mb>(args: Args, mb: Mb) -> Result<()>
where
    Mb: MemoryBacking<N, CdawgEdgeWeight<DefaultIx>, DefaultIx>,
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
    println!("  E: {}B", size_of::<E>());
    println!("  Node: {}B", size_of::<Node<N, DefaultIx>>());
    println!("  Edge: {}B", size_of::<Edge<E, DefaultIx>>());
    println!("");

    println!("Opening train file...");
    let train_file = fs::File::open(args.train_path.as_str())?;
    let n_bytes = train_file.metadata().unwrap().len();
    let buf_size: usize = min(n_bytes.try_into().unwrap(), args.buf_size);
    println!("Buffer size: {}B", args.buf_size);

    let reader: Box<DataReader> = if args.data_reader == "pile" {
        Box::new(PileReader::new(args.train_path).unwrap())
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

    let n_nodes = (args.nodes_ratio * (args.n_tokens as f64)).ceil() as usize;
    let n_edges = (args.edges_ratio * (args.n_tokens as f64)).ceil() as usize;
    let max_length: Option<u64> = if !args.max_state_length.is_negative() {
        Some(args.max_state_length.try_into().unwrap())
    } else {
        None
    };

    // Maintain a DiskVec that we update incrementally (whenever we read a token, set it).
    println!("# tokens: {}", args.n_tokens);
    println!("Creating train vector...");
    // let train_vec: Vec<u16> = Vec::with_capacity(args.n_tokens);
    let train_vec: DiskVec<u16> = DiskVec::new(&args.train_vec_path.unwrap(), args.n_tokens)?;
    let train_vec_rc = Rc::new(RefCell::new(train_vec));

    println!("Allocating CDAWG...");
    let mut cdawg: Cdawg<N, DefaultIx, Mb> =
        Cdawg::with_capacity_mb(train_vec_rc.clone(), mb, n_nodes, n_edges);

    println!("Starting build...");
    let mut idx: usize = 0;
    let mut pbar = tqdm!(total = args.n_tokens);
    let (mut state, mut start) = (cdawg.get_source(), 1);
    for (doc_id, doc) in reader {
        let tokens = index.tokenize(doc.as_str());
        for token in &tokens {
            // *token for Vec, token for DiskVec
            let _ = train_vec_rc.borrow_mut().push(token);
            // let _ = train_vec_rc.borrow_mut().push(*token);
            idx += 1;
            (state, start) = cdawg.update(state, start, idx);
            if *token == u16::MAX {
                (state, start) = cdawg.end_document(idx, doc_id);
            }
            pbar.update(1);

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

    // All this does is generate the metadata file.
    if let Some(disk_path) = args.disk_path {
        let _ = cdawg.save(disk_path.as_str());
    }

    let stats = BuildStats::from_cdawg(&cdawg, idx, n_bytes, pbar.elapsed_time());
    eprintln!();
    println!("");
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
    println!("");

    if !args.save_path.is_empty() {
        println!("Saving DAWG...");
        cdawg.save(&args.save_path).unwrap();  // FIXME
        println!("Successfully saved DAWG to {}!", &args.save_path);
    }
    Ok(())
}
