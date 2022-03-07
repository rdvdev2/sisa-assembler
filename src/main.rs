use sisa_assembler::{assemble, DataSectionStart, Flags};
use std::path::PathBuf;
use std::{env, panic, process, time};

struct Config {
    source_file: PathBuf,
    output_file: PathBuf,
    display_help: bool,
    executable: String,
    flags: Flags,
}

fn main() {
    panic::set_hook(Box::new(|pi| {
        eprintln!("The program panicked!\nThis means you found a bug in the assembler, and this error isn't your fault. \
        Report this to the software author.\n\nPanic details:\n{}", pi);
    }));

    if let Err(e) = run() {
        eprintln!("{}", e);
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let start = time::SystemTime::now();

    let config = parse_arguments()?;

    if config.display_help {
        println!("\
The SISA assembler by rdvdev2<me@rdvdev2.com>

Usage: {} [OPTIONS]

Recognized options:
    -i, --input FILE                Uses FILE as input (source.S by default)
    -o, --output FILE               Uses FILE as output (out.bin by default)

    --text-section-start ADDRESS    Places the .text section in ADDRESS (0x0000 by default)
    --data-section-start ADDRESS    Places the .data section in ADDRESS (right after .text by default)
    --auto-align-words              Automatically aligns words to multiples of 2 (disabled by default)
    --auto-align-sections           Automatically aligns sections to multiples of 2 (disabled by default)

    -h, --help                      Shows this help message", config.executable);
        return Ok(())
    }

    eprintln!("{}", assemble(&*config.source_file, &config.output_file, config.flags)?);

    let duration = time::SystemTime::now().duration_since(start);
    if let Ok(duration) = duration {
        println!("Assembly done in {} ms", duration.as_millis());
    } else {
        println!("Assembly done in ?? ms");
    }

    Ok(())
}

fn parse_arguments() -> Result<Config, String> {
    let mut args = env::args();
    let mut config = Config {
        source_file: PathBuf::from("source.S"),
        output_file: PathBuf::from("out.bin"),
        display_help: false,
        executable: args.next().unwrap(),
        flags: Default::default(),
    };

    while let Some(arg) = args.next() {
        match arg.as_ref() {
            "-i" | "--input" => config.source_file = args.next().ok_or("Missing a parameter after --input")?.into(),
            "-o" | "--output" => config.output_file = args.next().ok_or("Missing a parameter after --output")?.into(),
            "-h" | "--help" => config.display_help = true,

            "--text-section-start" => config.flags.text_section_start =
                u16::from_str_radix(&args.next().ok_or("Missing a parameter after --text-section-start")?[2..], 16)
                    .map_err(|e| format!("Error parsing address: {}", e))?,

            "--data-section-start" => config.flags.data_section_start = DataSectionStart::Absolute(
                u16::from_str_radix(&args.next().ok_or("Missing a parameter after --data-section-start")?[2..], 16)
                    .map_err(|e| format!("Error parsing address: {}", e))?),

            "--auto-align-words" => config.flags.auto_align_words = true,
            "--auto-align-sections" => config.flags.auto_align_sections = true,

            _ => return Err(format!("Unrecognized parameter, run {} -h for help.", config.executable)),
        }
    }

    Ok(config)
}