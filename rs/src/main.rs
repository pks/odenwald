use std::fs;
use std::io::{self, BufRead};

use clap::Parser;

use odenwald::chart_to_hg::chart_to_hg;
use odenwald::grammar::Grammar;
use odenwald::hypergraph_algos::{derive, viterbi_path};
use odenwald::parse::{self, Chart};
use odenwald::semiring::ViterbiSemiring;
use odenwald::sparse_vector::SparseVector;

#[derive(Parser)]
#[command(name = "odenwald")]
struct Cli {
    /// Grammar file
    #[arg(short = 'g', long)]
    grammar: String,

    /// Weights file
    #[arg(short = 'w', long)]
    weights: String,

    /// Input file (default: stdin)
    #[arg(short = 'i', long, default_value = "-")]
    input: String,

    /// Add glue rules
    #[arg(short = 'l', long)]
    add_glue: bool,

    /// Add pass-through rules
    #[arg(short = 'p', long)]
    add_pass_through: bool,
}

fn main() {
    let cli = Cli::parse();

    eprintln!("> reading grammar '{}'", cli.grammar);
    let mut grammar = Grammar::from_file(&cli.grammar).expect("Failed to read grammar");

    if cli.add_glue {
        eprintln!(">> adding glue rules");
        grammar.add_glue_rules();
    }

    let weights_content = fs::read_to_string(&cli.weights).expect("Failed to read weights");
    let weights = SparseVector::from_kv(&weights_content, ' ', '\n');

    let input_lines: Vec<String> = if cli.input == "-" {
        let stdin = io::stdin();
        stdin.lock().lines().map(|l| l.unwrap()).collect()
    } else {
        let content = fs::read_to_string(&cli.input).expect("Failed to read input");
        content.lines().map(|l| l.to_string()).collect()
    };

    for line in &input_lines {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let input: Vec<String> = line.split_whitespace().map(|s| s.to_string()).collect();
        let n = input.len();

        if cli.add_pass_through {
            eprintln!(">> adding pass-through rules");
            grammar.add_pass_through_rules(&input);
        }

        eprintln!("> initializing charts");
        let mut passive_chart = Chart::new(n);
        let mut active_chart = Chart::new(n);
        parse::init(&input, n, &mut active_chart, &mut passive_chart, &grammar);

        eprintln!("> parsing");
        parse::parse(&input, n, &mut active_chart, &mut passive_chart, &grammar);

        // Count passive chart items
        let mut chart_items = 0;
        let mut chart_cells = 0;
        parse::visit(1, 0, n, 0, |i, j| {
            let items = passive_chart.at(i, j).len();
            if items > 0 {
                chart_cells += 1;
                chart_items += items;
            }
        });
        eprintln!("> chart: {} items in {} cells", chart_items, chart_cells);

        let mut hg = chart_to_hg(&passive_chart, n, &weights, &grammar);
        eprintln!("> hg: {} nodes, {} edges", hg.nodes.len(), hg.edges.len());

        eprintln!("> viterbi");
        let (path, score) = viterbi_path::<ViterbiSemiring>(&mut hg);

        if path.is_empty() {
            eprintln!("WARNING: no parse found");
            println!("NO_PARSE ||| 0.0");
            continue;
        }

        let last_head = hg.edges[path.last().unwrap().0].head;
        let mut carry = Vec::new();
        derive(&hg, &path, last_head, &mut carry);

        println!("{} ||| {}", carry.join(" "), score.ln());
    }
}
