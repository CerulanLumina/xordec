use std::fs::File;
use std::io::{Read, Write, stdout};
use structopt::StructOpt;
use std::path::{PathBuf};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let opt: Opt = Opt::from_args();
    let pad_bytes = if opt.pad_file {
        let mut v = Vec::new();
        match File::open(opt.pad.as_ref().unwrap()) {
            Ok(mut f) => {
                f.read_to_end(&mut v).expect("reading pad file");
                v
            },
            Err(err) => {
                eprintln!("Pad file {:?} does not exist.", opt.pad.as_ref().unwrap().as_os_str());
                return Err(Box::new(err))
            }
        }

    } else {
        match std::env::var("XORDEC_PAD") {
            Ok(pad) => pad.as_bytes().to_vec(),
            Err(err) => {
                eprintln!("Missing pad file or environment variable XORDEC_PAD!");
                return Err(Box::new(err));
            }
        }
    };
    let mut input_bytes = if opt.input.as_os_str() == "-" {
        let mut v = Vec::new();
        std::io::stdin().read_to_end(&mut v).expect("reading stdin");
        v
    } else {
        match File::open(&opt.input) {
            Ok(mut file) => {
                let mut v = Vec::new();
                file.read_to_end(&mut v).expect("reading file");
                v
            },
            Err(err) => {
                eprintln!("Unable to read input file");
                return Err(Box::new(err));
            }
        }
    };
    input_bytes.iter_mut().enumerate().for_each(|(i, a)| *a ^= pad_bytes[i % pad_bytes.len()]);
    let mut writer: Box<dyn Write> = opt.output.map_or_else::<Box<dyn Write>, _, _>(|| {
        Box::new(stdout())
    }, |a| {
        match File::create(&a) {
            Ok(f) => Box::new(f),
            Err(err) => {
                eprintln!("Failed to open file for writing: {:?}", a);
                eprintln!("{}", err);
                std::process::exit(1);
            }
        }
    });
    if let Err(err) = writer.write_all(input_bytes.as_slice()) {
        eprintln!("Failed to write output file.");
        return Err(Box::new(err));
    }
    Ok(())
}

#[derive(StructOpt)]
#[structopt()]
struct Opt {
    #[structopt(short, long)]
    /// Whether to use a pad file for XOR. Otherwise will use contents of environment variable XORDEC_PAD
    pad_file: bool,
    #[structopt(parse(from_os_str))]
    /// Input file, if - use stdin
    input: PathBuf,
    #[structopt(parse(from_os_str))]
    /// Output file, if omitted, use stdout
    output: Option<PathBuf>,
    #[structopt(parse(from_os_str), required_if("pad_file", "true"), long)]
    /// The XOR Pad to use
    pad: Option<PathBuf>,

}