// Implementation of Suffix DFA in Rust.
//
// See here for Graph info:
// https://docs.rs/petgraph/latest/petgraph/graph/struct.Graph.html
//
// See here for Suffix Automaton algorithm in Python:
// https://github.com/viking-sudo-rm/knn-transformers/blob/master/src/suffix_dfa_builder.py
//

extern crate bincode;
extern crate bitvec;
extern crate clap;
extern crate kdam;
extern crate petgraph;
extern crate serde;
extern crate serde_json;
extern crate substring;
extern crate tempfile;
extern crate tokenizers;
extern crate anyhow;
extern crate unicode_segmentation;

mod dawg;
mod evaluator;
mod graph;
mod lms;
mod stat_utils;
mod tokenize;
mod tokenize2;
mod weight;

use lms::induction_lm::InductionLM;
use lms::kn_lm::KNLM;
use lms::LM;

use bincode::serialize_into;
use clap::Parser;
use std::fs;
use std::mem::size_of;

use kdam::tqdm;

use dawg::Dawg;
use evaluator::Evaluator;
use stat_utils::*;
use tokenize::{NullTokenIndex, TokenIndex, Tokenize};
use weight::Weight40;

// Node and edge weight types.
type N = Weight40; // FIXME: Actually pass this as a generic argument to Dawg.
type E = usize;

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
    #[arg(long, default_value_t = false)]
    tokenize: bool,
    #[arg(long, default_value_t = 10000)]
    truncate_test: usize,
    #[arg(long, default_value_t = 20)]
    n_eval: usize,
    #[arg(long, default_value_t = 10)]
    max_length: u64,

    #[arg(long, short = 'f')]
    min_freq: Vec<u64>,
    #[arg(long, short = 'd')]
    delta: Vec<f64>,
    #[arg(long, short = 'n')]
    n_gram: Vec<i64>,
    #[arg(long, short = 'i')]
    induct_delta: Vec<f64>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("sizeof(edge): {}B", size_of::<E>());
    println!("sizeof(node): {}B", size_of::<N>());

    let args = Args::parse();
    let mut index: Box<dyn Tokenize> = if args.tokenize {
        Box::new(TokenIndex::<usize>::new())
    } else {
        Box::new(NullTokenIndex::new())
    };

    let train_raw: String =
        fs::read_to_string(args.train_path.as_str()).expect("Error loading train");
    let train: Vec<usize> = train_raw.split_whitespace().map(|x| index.add(x)).collect();
    let eval_threshold = if args.n_eval != 0 {
        train.len() / args.n_eval
    } else {
        0
    };
    println!("#(train): {}", train.len());

    let test_raw: String = fs::read_to_string(args.test_path.as_str()).expect("Error loading test");
    let mut test: Vec<usize> = test_raw.split_whitespace().map(|x| index.add(x)).collect();
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
    let gen: Vec<E> = gen_raw.split_whitespace().map(|x| index.add(x)).collect();
    println!("#(gen): {}", gen.len());
    println!("#(vocab): {}", index.get_count());

    let mut lms: Vec<Box<dyn LM>> = Vec::new();
    create_lms(&args, &mut lms);
    let mut evaluator = Evaluator::new(&mut lms, &test, args.max_length);
    let mut gen_lms: Vec<Box<dyn LM>> = Vec::new();
    create_lms(&args, &mut gen_lms);
    let mut gen_evaluator = Evaluator::new(&mut gen_lms, &gen, args.max_length);

    let mut dawg: Dawg<E> = Dawg::with_capacity(2 * train.len());
    let mut last = dawg.get_initial();
    for (idx, token) in tqdm!(train.iter()).enumerate() {
        last = dawg.extend(*token, last);
        if eval_threshold != 0 && idx % eval_threshold == 0 && idx != 0 {
            let good_turing = good_turing_estimate(&dawg, train.len());
            evaluator.evaluate(&dawg, idx, good_turing);
            if !args.results_path.is_empty() {
                evaluator.to_json(&args.results_path)?;
            }
            match &args.gen_results_path {
                Some(gen_path) => {
                    gen_evaluator.evaluate(&dawg, idx, good_turing);
                    gen_evaluator.to_json(gen_path)?;
                }
                None => {}
            }
        }
    }
    println!("Completed!");
    println!("  Node count: {}", dawg.node_count());
    println!("  Edge count: {}", dawg.edge_count());

    if !args.save_path.is_empty() {
        println!("Saving DAWG...");
        checkpoint(&dawg, &args.save_path)?;
        println!("Successfully saved DAWG to {}!", &args.save_path);
    }
    Ok(())
}

fn create_lms(args: &Args, lms: &mut Vec<Box<dyn LM>>) {
    for min_freq in args.min_freq.iter() {
        for delta in args.delta.iter() {
            let maxgram = KNLM::new(
                format!("maxgram-kn{}-#{}", delta, min_freq),
                *delta,
                -1,
                *min_freq,
            );
            lms.push(Box::new(maxgram));
            for n in args.n_gram.iter() {
                let ngram = KNLM::new(
                    format!("{}gram-kn{}-#{}", n, delta, min_freq),
                    *delta,
                    *n,
                    *min_freq,
                );
                lms.push(Box::new(ngram));
                for induct_delta in args.induct_delta.iter() {
                    let induct_backoff = KNLM::new(
                        format!("sub-{}gram-kn{}-#{}", n, delta, min_freq),
                        *delta,
                        *n,
                        *min_freq,
                    );
                    let induct = InductionLM::new(
                        format!("{}gram-kn{}-#{}-induct{}", n, delta, min_freq, induct_delta),
                        Box::new(induct_backoff),
                        *induct_delta,
                    );
                    lms.push(Box::new(induct))
                }
            }
        }
    }
}

fn checkpoint(dawg: &Dawg<usize>, save_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let save_file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(save_path)?;
    serialize_into(&save_file, &dawg)?;

    // HOWTO: Deserialize
    // let mut load_file = fs::OpenOptions::new()
    //     .read(true)
    //     .open(save_path)?;
    // let decoded: Dawg<usize> = deserialize_from(&load_file).expect("Failed to deserialize");
    // println!("decoded DAWG");
    Ok(())
}
