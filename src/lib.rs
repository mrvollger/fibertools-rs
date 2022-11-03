/// Command line interface for fibertools-rs.
pub mod bamlift;
pub mod center;
pub mod cli;
#[cfg(feature = "cnn")]
pub mod cnn;
pub mod extract;
pub mod ml_models;
pub mod predict_m6a;

use anyhow::Result;
use rust_htslib::{bam, bam::Read};
use std::fs::File;
use std::io::{self, stdout, BufWriter, Write};
use std::path::PathBuf;
use std::process::exit;
const BUFFER_SIZE: usize = 32 * 1024;
const PROGRESS_STYLE: &str =
    "[{elapsed_precise:.yellow}] {bar:50.cyan/blue} {human_pos:>5.cyan}/{human_len:.blue} {percent:>3.green}% {per_sec:<10.cyan}";

/// Get a buffered output writer from stdout or a file
fn get_output(path: Option<PathBuf>) -> Result<Box<dyn Write + Send + 'static>> {
    let writer: Box<dyn Write + Send + 'static> = match path {
        Some(path) => {
            if path.as_os_str() == "-" {
                Box::new(BufWriter::with_capacity(BUFFER_SIZE, stdout()))
            } else {
                Box::new(BufWriter::with_capacity(BUFFER_SIZE, File::create(path)?))
            }
        }
        None => Box::new(BufWriter::with_capacity(BUFFER_SIZE, stdout())),
    };
    Ok(writer)
}

/// unzip a vector of tupples
pub fn unzip_to_vectors<T, U>(vec: Vec<(T, U)>) -> (Vec<T>, Vec<U>) {
    vec.into_iter().unzip()
}

/// join a vector with commas
pub fn join_by_str<'a, I, Z>(vals: I, sep: &str) -> String
where
    I: IntoIterator<Item = Z>,
    Z: ToString + 'a,
{
    vals.into_iter().map(|v| v.to_string() + sep).collect()
}

/// Write to stdout if - or the file specified by a path
pub fn writer(filename: &str) -> Result<Box<dyn Write>> {
    //let ext = Path::new(filename).extension();
    let path = PathBuf::from(filename);
    let buffer = get_output(Some(path))?; //.expect("Error: cannot create output file");
    Ok(buffer)
}

/// Open bam file
pub fn bam_reader(bam: &str, threads: usize) -> bam::Reader {
    let mut bam = if bam == "-" {
        bam::Reader::from_stdin().unwrap_or_else(|_| panic!("Failed to open bam from stdin"))
    } else {
        bam::Reader::from_path(bam).unwrap_or_else(|_| panic!("Failed to open {}", bam))
    };
    bam.set_threads(threads).unwrap();
    bam
}

pub struct FiberOut {
    pub m6a: Option<Box<dyn Write>>,
    pub cpg: Option<Box<dyn Write>>,
    pub msp: Option<Box<dyn Write>>,
    pub nuc: Option<Box<dyn Write>>,
    pub all: Option<Box<dyn Write>>,
    pub reference: bool,
    pub simplify: bool,
    pub quality: bool,
    pub min_ml_score: u8,
}

impl FiberOut {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        m6a: &Option<String>,
        cpg: &Option<String>,
        msp: &Option<String>,
        nuc: &Option<String>,
        all: &Option<String>,
        reference: bool,
        simplify: bool,
        quality: bool,
        min_ml_score: u8,
    ) -> Result<Self> {
        let m6a = match m6a {
            Some(m6a) => Some(writer(m6a)?),
            None => None,
        };
        let cpg = match cpg {
            Some(cpg) => Some(writer(cpg)?),
            None => None,
        };
        let msp = match msp {
            Some(msp) => Some(writer(msp)?),
            None => None,
        };
        let nuc = match nuc {
            Some(nuc) => Some(writer(nuc)?),
            None => None,
        };
        let all = match all {
            Some(all) => Some(writer(all)?),
            None => None,
        };
        Ok(FiberOut {
            m6a,
            cpg,
            msp,
            nuc,
            all,
            reference,
            simplify,
            quality,
            min_ml_score,
        })
    }
}

/// write to a file, but don't error on broken pipes
pub fn write_to_file(out: &str, file: &mut Box<dyn Write>) {
    let out = write!(file, "{}", out);
    if let Err(err) = out {
        if err.kind() == io::ErrorKind::BrokenPipe {
            exit(0);
        } else {
            panic!("Error: {}", err);
        }
    }
}

pub fn write_to_stdout(out: &str) {
    let mut out_f = Box::new(std::io::stdout()) as Box<dyn Write>;
    write_to_file(out, &mut out_f);
}

// This is a bam chunk reader
use colored::Colorize;
use std::time::Instant;

struct BamChunk<'a> {
    bam: bam::Records<'a, bam::Reader>,
    chunk_size: usize,
}

// The `Iterator` trait only requires a method to be defined for the `next` element.
impl<'a> Iterator for BamChunk<'a> {
    // We can refer to this type using Self::Item
    type Item = Vec<bam::Record>;

    // The return type is `Option<T>`:
    //     * When the `Iterator` is finished, `None` is returned.
    //     * Otherwise, the next value is wrapped in `Some` and returned.
    // We use Self::Item in the return type, so we can change
    // the type without having to update the function signatures.
    fn next(&mut self) -> Option<Self::Item> {
        let start = Instant::now();
        let mut cur_vec = vec![];
        for r in self.bam.by_ref().take(self.chunk_size) {
            cur_vec.push(r.unwrap())
        }
        // return
        if cur_vec.is_empty() {
            None
        } else {
            let duration = start.elapsed().as_secs_f64();
            log::info!(
                "Read {} bam records at {}.",
                format!("{:}", cur_vec.len()).bright_magenta().bold(),
                format!("{:.2?} reads/s", cur_vec.len() as f64 / duration)
                    .bright_cyan()
                    .bold(),
            );
            Some(cur_vec)
        }
    }
}
