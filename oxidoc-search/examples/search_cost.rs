//! Deterministic cost probe for the browser search engine.
//!
//! Each phase runs in its own process, so one `perf stat` run measures one phase.
//! The phases call the same library functions as the wasm exports in `src/wasm.rs`.
//! Measure instructions, never wall-clock time. The output holds no timings.
//!
//! ```text
//! cargo build --release --example search_cost --features semantic-build
//! BIN=target/release/examples/search_cost
//! $BIN build --out DIR [--records N] [--model GGUF]
//! perf stat -e instructions:u $BIN init --out DIR
//! perf stat -e instructions:u $BIN query --out DIR
//! perf stat -e instructions:u $BIN model-load --out DIR --model GGUF
//! perf stat -e instructions:u $BIN query-ai --out DIR --model GGUF
//! $BIN query --out DIR --dump FILE
//! ```
//!
//! `--dump FILE` applies to `query` and `query-ai`. It writes one line per query run:
//! the query text, a tab, then the JSON string of the results. The file is written
//! after the query loop.
//!
//! Every count includes process start-up. Subtract an `init` run from a `query` run
//! to get the cost of the queries alone.

use std::error::Error;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use boostr::format::Gguf;
use boostr::{CpuClient, CpuDevice, CpuRuntime, EmbeddingPipeline};
use flate2::Compression;
use flate2::write::GzEncoder;
use oxidoc_search::builder::vectors::{MODEL_FILE, VECTORS_FILE};
use oxidoc_search::builder::write::{META_FILE, chunk_file_name};
use oxidoc_search::builder::{
    EmbeddingModel, build_lexical, build_records, build_vectors, write_lexical, write_model,
    write_vectors,
};
use oxidoc_search::records::RECORDS_FILE;
use oxidoc_search::vectors::decode_vectors;
use oxidoc_search::{
    Field, LexicalIndex, LexicalSearcher, Record, SearchEngine, SearchQuery, SemanticSearcher,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
type Engine = SearchEngine<CpuRuntime>;

const USAGE: &str = "usage: search_cost <build|init|query|model-load|query-ai> --out DIR \
                     [--records N] [--model PATH] [--dump FILE]\n\
                     --records applies to build only. --model is required by model-load and query-ai.\n\
                     --dump applies to query and query-ai only.";

const DEFAULT_RECORDS: usize = 1000;
const QUERY_REPEATS: usize = 10;
const MAX_RESULTS: usize = 10;

/// Fixed query list: 20 single words, 12 word pairs, 9 prefixes and 9 typos.
const QUERIES: [&str; 50] = [
    "tensor",
    "gradient",
    "transformer",
    "optimizer",
    "embedding",
    "quantization",
    "tokenizer",
    "inference",
    "kernel",
    "attention",
    "scheduler",
    "checkpoint",
    "dataset",
    "compiler",
    "allocator",
    "benchmark",
    "mamba",
    "diffusion",
    "regression",
    "async",
    "tensor kernel",
    "gradient descent",
    "attention mask",
    "batch normalization",
    "model checkpoint",
    "memory allocator",
    "sparse matrix",
    "distributed training",
    "zero copy",
    "beam search",
    "vector index",
    "token stream",
    "tens",
    "grad",
    "trans",
    "optim",
    "embed",
    "quant",
    "token",
    "infer",
    "sched",
    "tensro",
    "gradeint",
    "trasformer",
    "optimzer",
    "embeding",
    "quantizaton",
    "tokenzier",
    "infrence",
    "kernal",
];

const KINDS: [&str; 5] = ["library", "tool", "model", "dataset", "framework"];
const TOPICS: [&str; 8] = [
    "nlp", "vision", "audio", "tabular", "systems", "science", "graphics", "robotics",
];
const LICENSES: [&str; 4] = ["MIT", "Apache-2.0", "GPL-3.0", "BSD-3-Clause"];
const BOOSTS: [f32; 6] = [1.0, 1.0, 1.0, 1.25, 1.5, 0.75];

/// Weights of the four record fields: name, keywords, description, readme.
const FIELD_WEIGHTS: [f32; 4] = [3.0, 2.0, 1.5, 1.0];

/// Word pool for the generated text. A draw favours the front of the list.
const VOCAB: &[&str] = &[
    "tensor",
    "gradient",
    "transformer",
    "optimizer",
    "embedding",
    "quantization",
    "tokenizer",
    "inference",
    "kernel",
    "attention",
    "scheduler",
    "checkpoint",
    "dataset",
    "compiler",
    "allocator",
    "benchmark",
    "mamba",
    "diffusion",
    "regression",
    "async",
    "descent",
    "mask",
    "batch",
    "normalization",
    "model",
    "memory",
    "sparse",
    "matrix",
    "distributed",
    "training",
    "zero",
    "copy",
    "beam",
    "search",
    "vector",
    "index",
    "token",
    "stream",
    "layer",
    "weight",
    "bias",
    "loss",
    "activation",
    "softmax",
    "dropout",
    "residual",
    "encoder",
    "decoder",
    "context",
    "window",
    "prompt",
    "sampling",
    "temperature",
    "logits",
    "vocabulary",
    "corpus",
    "pipeline",
    "parser",
    "lexer",
    "serializer",
    "codec",
    "buffer",
    "cache",
    "thread",
    "channel",
    "mutex",
    "atomic",
    "future",
    "runtime",
    "executor",
    "reactor",
    "socket",
    "protocol",
    "packet",
    "latency",
    "throughput",
    "profile",
    "trace",
    "metric",
    "counter",
    "histogram",
    "logger",
    "config",
    "schema",
    "migration",
    "query",
    "planner",
    "storage",
    "block",
    "page",
    "segment",
    "shard",
    "replica",
    "cluster",
    "node",
    "gateway",
    "router",
    "proxy",
    "filter",
    "reducer",
    "mapper",
    "iterator",
    "closure",
    "trait",
    "generic",
    "lifetime",
    "borrow",
    "ownership",
    "macro",
    "crate",
    "module",
    "workspace",
    "feature",
    "target",
    "linker",
    "binary",
    "library",
    "framework",
    "toolkit",
    "engine",
    "server",
    "client",
    "daemon",
    "service",
    "worker",
    "queue",
    "stack",
    "heap",
    "arena",
    "pool",
    "slab",
    "ring",
    "tree",
    "graph",
    "forest",
    "boosting",
    "bagging",
    "clustering",
    "centroid",
    "distance",
    "cosine",
    "norm",
    "eigen",
    "decomposition",
    "factorization",
    "convolution",
    "pooling",
    "stride",
    "padding",
    "kernelized",
    "fourier",
    "wavelet",
    "spectrogram",
    "waveform",
    "phoneme",
    "speech",
    "audio",
    "image",
    "video",
    "frame",
    "pixel",
    "texture",
    "shader",
    "render",
    "mesh",
    "camera",
    "lidar",
    "sensor",
    "planner",
    "controller",
    "policy",
    "reward",
    "agent",
    "environment",
    "simulation",
    "physics",
    "solver",
    "integrator",
    "ode",
    "stiff",
    "adaptive",
    "tolerance",
    "precision",
    "float",
    "half",
    "bfloat",
    "integer",
    "fixed",
    "calibration",
    "pruning",
    "distillation",
    "adapter",
    "lora",
    "finetune",
    "pretrain",
    "evaluation",
    "accuracy",
    "recall",
    "perplexity",
    "bleu",
    "annotation",
    "label",
    "split",
    "shuffle",
    "augment",
    "loader",
    "reader",
    "writer",
    "format",
    "gguf",
    "safetensors",
    "onnx",
    "wasm",
    "browser",
    "island",
    "component",
    "template",
    "theme",
    "style",
    "markdown",
    "docs",
    "reference",
    "guide",
    "tutorial",
    "example",
    "release",
    "changelog",
    "license",
    "contribute",
    "review",
    "testing",
    "fuzz",
    "property",
    "snapshot",
    "golden",
    "coverage",
    "lint",
    "format",
];

/// Linear congruential generator (Knuth MMIX constants). Same seed, same sequence.
struct Lcg(u64);

impl Lcg {
    fn next_bits(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 33
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next_bits() % n as u64) as usize
    }

    fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len())]
    }

    /// Draws a vocabulary word. The lower of two draws skews toward common words.
    fn word(&mut self) -> &'static str {
        let a = self.below(VOCAB.len());
        let b = self.below(VOCAB.len());
        VOCAB[a.min(b)]
    }

    fn words(&mut self, count: usize) -> String {
        let words: Vec<&str> = (0..count).map(|_| self.word()).collect();
        words.join(" ")
    }

    /// Builds sentences until the text reaches a target of 1 to 3 KB.
    fn readme(&mut self) -> String {
        let target = 1024 + self.below(2049);
        let mut text = String::new();
        while text.len() < target {
            let count = 6 + self.below(9);
            text.push_str(&self.words(count));
            text.push_str(". ");
        }
        text
    }
}

fn make_records(count: usize) -> Vec<Record> {
    let mut rng = Lcg(0x5EED_C057);
    let mut records = Vec::with_capacity(count);
    for i in 0..count {
        let name = format!("{}-{}-{i}", rng.word(), rng.word());
        let keywords_len = 3 + rng.below(4);
        let description_len = 12 + rng.below(14);
        let texts = [
            name.clone(),
            rng.words(keywords_len),
            rng.words(description_len),
            rng.readme(),
        ];
        let mut record = Record::new(format!("rec-{i}"), format!("/catalogue/{i}"), name);
        let names = ["name", "keywords", "description", "readme"];
        for ((field_name, text), weight) in names.iter().zip(texts).zip(FIELD_WEIGHTS) {
            let mut field = Field::new(*field_name, text);
            field.weight = weight;
            record.fields.push(field);
        }
        record.tags = vec![
            ("kind".to_string(), rng.pick(&KINDS).to_string()),
            ("topic".to_string(), rng.pick(&TOPICS).to_string()),
            ("license".to_string(), rng.pick(&LICENSES).to_string()),
        ];
        record.boost = *rng.pick(&BOOSTS);
        records.push(record);
    }
    records
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Build,
    Init,
    Query,
    ModelLoad,
    QueryAi,
}

impl Phase {
    fn parse(name: &str) -> Option<Self> {
        match name {
            "build" => Some(Self::Build),
            "init" => Some(Self::Init),
            "query" => Some(Self::Query),
            "model-load" => Some(Self::ModelLoad),
            "query-ai" => Some(Self::QueryAi),
            _ => None,
        }
    }

    fn needs_model(self) -> bool {
        matches!(self, Self::ModelLoad | Self::QueryAi)
    }
}

struct Args {
    phase: Phase,
    out: PathBuf,
    records: usize,
    model: Option<PathBuf>,
    dump: Option<PathBuf>,
}

fn parse_args(mut argv: impl Iterator<Item = String>) -> std::result::Result<Args, String> {
    let phase_name = argv.next().ok_or("missing phase")?;
    let phase = Phase::parse(&phase_name).ok_or(format!("unknown phase '{phase_name}'"))?;
    let mut out = None;
    let mut records = DEFAULT_RECORDS;
    let mut model = None;
    let mut dump = None;
    while let Some(flag) = argv.next() {
        let value = argv.next().ok_or(format!("{flag} needs a value"))?;
        match flag.as_str() {
            "--out" => out = Some(PathBuf::from(value)),
            "--model" => model = Some(PathBuf::from(value)),
            "--dump" => dump = Some(PathBuf::from(value)),
            "--records" => {
                records = value
                    .parse::<usize>()
                    .ok()
                    .filter(|n| *n > 0)
                    .ok_or(format!("--records needs a positive integer, got '{value}'"))?;
            }
            _ => return Err(format!("unknown flag '{flag}'")),
        }
    }
    let out = out.ok_or("missing --out DIR")?;
    if phase.needs_model() && model.is_none() {
        return Err("this phase needs --model PATH".to_string());
    }
    if dump.is_some() && !matches!(phase, Phase::Query | Phase::QueryAi) {
        return Err("--dump applies to query and query-ai only".to_string());
    }
    Ok(Args {
        phase,
        out,
        records,
        model,
        dump,
    })
}

fn read_file(dir: &Path, name: &str) -> Result<Vec<u8>> {
    let path = dir.join(name);
    fs::read(&path).map_err(|e| format!("cannot read {}: {e}", path.display()).into())
}

fn is_output_name(name: &str) -> bool {
    name == META_FILE
        || name == RECORDS_FILE
        || name == VECTORS_FILE
        || name == MODEL_FILE
        || (name.starts_with("search-chunk-") && name.ends_with(".bin"))
}

/// Lists the output files of `dir` by name.
fn output_files(dir: &Path) -> Result<Vec<(String, PathBuf)>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| format!("cannot read {}: {e}", dir.display()))? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if is_output_name(&name) {
            files.push((name, entry.path()));
        }
    }
    files.sort();
    Ok(files)
}

/// Counts bytes written to it and drops them.
struct CountingSink(u64);

impl Write for CountingSink {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0 += buf.len() as u64;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Returns the size of `path` after gzip at the default level.
fn gzip_len(path: &Path) -> Result<u64> {
    let mut encoder = GzEncoder::new(CountingSink(0), Compression::default());
    io::copy(&mut File::open(path)?, &mut encoder)?;
    Ok(encoder.finish()?.0)
}

fn print_sizes(dir: &Path) -> Result<()> {
    println!("{:<24} {:>12} {:>12}", "file", "raw_bytes", "gzip_bytes");
    let (mut raw_total, mut gzip_total) = (0u64, 0u64);
    for (name, path) in output_files(dir)? {
        let raw = fs::metadata(&path)?.len();
        let gzip = gzip_len(&path)?;
        println!("{name:<24} {raw:>12} {gzip:>12}");
        raw_total += raw;
        gzip_total += gzip;
    }
    println!("{:<24} {raw_total:>12} {gzip_total:>12}", "total");
    Ok(())
}

fn build(args: &Args) -> Result<()> {
    fs::create_dir_all(&args.out)?;
    for (_, path) in output_files(&args.out)? {
        fs::remove_file(path)?;
    }
    let records = make_records(args.records);
    let index = build_lexical(&records);
    let side = build_records(&records);
    write_lexical(&index, Some(&side), &args.out)?;
    if let Some(path) = &args.model {
        let model = EmbeddingModel::File(path);
        write_vectors(&build_vectors(&records, model)?, &args.out)?;
        write_model(model, &args.out)?;
    }
    println!("records {}", args.records);
    print_sizes(&args.out)
}

/// Mirrors `oxidoc_search_init`, `oxidoc_search_load_chunk` and `oxidoc_search_load_records`.
fn init(dir: &Path) -> Result<(Engine, CpuClient, u32)> {
    let meta = read_file(dir, META_FILE)?;
    let mut engine = Engine::new(LexicalSearcher::from_metadata(&meta)?, None);
    let client = CpuClient::new(CpuDevice::new());
    let mut chunks = 0u32;
    while dir.join(chunk_file_name(chunks)).exists() {
        engine.load_chunk(&read_file(dir, &chunk_file_name(chunks))?)?;
        chunks += 1;
    }
    engine.load_records(&read_file(dir, RECORDS_FILE)?)?;
    Ok((engine, client, chunks))
}

/// Mirrors `oxidoc_search_load_semantic`. The browser passes the vectors as bytes.
fn load_semantic(engine: &mut Engine, dir: &Path, model_path: &Path) -> Result<usize> {
    let model_data =
        fs::read(model_path).map_err(|e| format!("cannot read {}: {e}", model_path.display()))?;
    let vectors = decode_vectors(&read_file(dir, VECTORS_FILE)?)?;
    let dimension = vectors.dimension();
    let mut gguf =
        Gguf::from_bytes(model_data).map_err(|e| format!("Failed to parse GGUF model: {e}"))?;
    let pipeline = EmbeddingPipeline::<CpuRuntime>::from_gguf(&mut gguf, CpuDevice::new())
        .map_err(|e| format!("Failed to load embedding pipeline: {e}"))?;
    engine.set_semantic(SemanticSearcher::new(pipeline, vectors));
    Ok(dimension)
}

fn query_of(text: &str) -> SearchQuery {
    SearchQuery {
        text: text.to_string(),
        max_results: MAX_RESULTS,
        filters: Vec::new(),
    }
}

/// Appends `text`, a tab and `json` as one line to the dump buffer, when dumping.
fn dump_line(dump: &mut Option<String>, text: &str, json: &str) {
    if let Some(buffer) = dump {
        buffer.push_str(text);
        buffer.push('\t');
        buffer.push_str(json);
        buffer.push('\n');
    }
}

/// Writes the dump buffer to the `--dump` path. Does nothing when `--dump` is absent.
fn write_dump(path: Option<&Path>, dump: Option<String>) -> Result<()> {
    if let (Some(path), Some(buffer)) = (path, dump) {
        fs::write(path, buffer).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    }
    Ok(())
}

/// Mirrors `oxidoc_search_query`: lexical search, then JSON encoding of the results.
fn run_lexical(
    engine: &Engine,
    repeats: usize,
    dump: &mut Option<String>,
) -> Result<(usize, usize)> {
    let (mut hits, mut json_bytes) = (0, 0);
    for _ in 0..repeats {
        for text in QUERIES {
            let results = engine.search_lexical(&query_of(text));
            hits += results.len();
            let json = serde_json::to_string(&results)?;
            json_bytes += json.len();
            dump_line(dump, text, &json);
        }
    }
    Ok((hits, json_bytes))
}

/// Mirrors `oxidoc_search_query_ai`: hybrid search, then JSON encoding of the results.
fn run_hybrid(
    engine: &Engine,
    client: &CpuClient,
    repeats: usize,
    dump: &mut Option<String>,
) -> Result<(usize, usize)> {
    let (mut hits, mut json_bytes) = (0, 0);
    for _ in 0..repeats {
        for text in QUERIES {
            let results = engine.search(client, &query_of(text))?;
            hits += results.len();
            let json = serde_json::to_string(&results)?;
            json_bytes += json.len();
            dump_line(dump, text, &json);
        }
    }
    Ok((hits, json_bytes))
}

fn run(args: &Args) -> Result<()> {
    let model = args.model.as_deref();
    match args.phase {
        Phase::Build => build(args),
        Phase::Init => {
            let (engine, _client, chunks) = init(&args.out)?;
            println!("documents {} chunks {chunks}", engine.document_count());
            Ok(())
        }
        Phase::Query => {
            let (engine, _client, _chunks) = init(&args.out)?;
            let mut dump = args.dump.is_some().then(String::new);
            let (hits, json_bytes) = run_lexical(&engine, QUERY_REPEATS, &mut dump)?;
            write_dump(args.dump.as_deref(), dump)?;
            println!(
                "documents {} queries {} hits {hits} json_bytes {json_bytes}",
                engine.document_count(),
                QUERIES.len() * QUERY_REPEATS
            );
            Ok(())
        }
        Phase::ModelLoad => {
            let model = model.ok_or("this phase needs --model PATH")?;
            let empty = LexicalIndex {
                postings: Default::default(),
                documents: Vec::new(),
            };
            let mut engine = Engine::new(LexicalSearcher::new(empty), None);
            let dimension = load_semantic(&mut engine, &args.out, model)?;
            println!("semantic {} dimension {dimension}", engine.has_semantic());
            Ok(())
        }
        Phase::QueryAi => {
            let model = model.ok_or("this phase needs --model PATH")?;
            let (mut engine, client, _chunks) = init(&args.out)?;
            load_semantic(&mut engine, &args.out, model)?;
            let mut dump = args.dump.is_some().then(String::new);
            let (hits, json_bytes) = run_hybrid(&engine, &client, 1, &mut dump)?;
            write_dump(args.dump.as_deref(), dump)?;
            println!(
                "documents {} queries {} hits {hits} json_bytes {json_bytes}",
                engine.document_count(),
                QUERIES.len()
            );
            Ok(())
        }
    }
}

fn main() -> ExitCode {
    let args = match parse_args(std::env::args().skip(1)) {
        Ok(args) => args,
        Err(message) => {
            eprintln!("search_cost: {message}\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("search_cost: {error}");
            ExitCode::FAILURE
        }
    }
}
