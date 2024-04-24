use crate::guitar_string::GuitarString;
use anyhow::{Context, Result};
use itertools::Itertools;
use std::env::{self, args};
use std::fs::File;
use std::io::{BufRead, Write};

// link guitar_string.rs
mod guitar_string;

const SAMPLE_RATE: usize = 44100;
const NUM_STRINGS: usize = 37;
const STEP: f64 = 1.0 / SAMPLE_RATE as f64;
const END_OF_SONG: i32 = -1;

fn open_files(in_file: &str, out_file: &str) -> Result<(File, File)> {
    let in_file = File::open(in_file).context("Could not open input file")?;
    let mut out_file = File::create(out_file).context("Could not open output file")?;

    out_file
        .write_all(format!("; Sample Rate {}\n; Channels 1\n\n", SAMPLE_RATE).as_bytes())
        .context("Could not write header to output file")?;

    return Ok((in_file, out_file));
}

fn close_files(in_file: File, out_file: File) -> Result<()> {
    in_file.sync_all().context("Could not sync input file")?;
    out_file.sync_all().context("Could not sync output file")?;

    return Ok(());
}

fn validate_line(prev_time: f64, time: f64, note: i32) -> Result<()> {
    if time < prev_time {
        return Err(anyhow::anyhow!("Time must be increasing"));
    }

    if note > 36 || note < -1 {
        return Err(anyhow::anyhow!("String must be in range"));
    }

    return Ok(());
}

fn main() -> Result<()> {
    let args = args().collect_vec();
    if args.len() != 3 {
        println!(
            "Usage: {} <input_file> <output_file>",
            args[0].split('/').last().unwrap()
        );
        return Err(anyhow::anyhow!("Invalid number of arguments"));
    }

    let (in_file, mut out_file) = open_files(&args[1], &args[2])?;
    let mut strings = (0..NUM_STRINGS)
        .map(|i| GuitarString::new(440.0 * 2.0_f64.powf((i as f64 - 24.0) / 12.0)))
        .collect_vec();

    print!("Processing {} into {}", &args[1], &args[2]);

    // read input file line by line
    let mut reader = std::io::BufReader::new(&in_file);
    let mut buf = String::new();

    reader.read_line(&mut buf).unwrap();

    let mut time_counter = 0.0;
    let mut sample_sum;

    let mut parts = buf.split_whitespace();
    let mut time = parts.next().unwrap().parse::<f64>().unwrap();
    let mut note = parts.next().unwrap().parse::<i32>().unwrap();

    let mut prev_time = 0.0;

    while note != END_OF_SONG {
        if time_counter >= time {
            // do while loop
            loop {
                validate_line(prev_time, time, note)?;

                if note != END_OF_SONG {
                    strings[note as usize].pluck();
                    // println!("Plucking string {} at time {}", note, time_counter);
                    print!(".");
                    // flush stdout
                    std::io::stdout().flush().unwrap();
                }

                prev_time = time;

                buf.clear();

                if reader.read_line(&mut buf).unwrap() > 0 {
                    parts = buf.split_whitespace();
                    time = parts.next().unwrap().parse::<f64>().unwrap();
                    note = parts.next().unwrap().parse::<i32>().unwrap();
                } else {
                    note = END_OF_SONG;
                    break;
                }

                if time != prev_time {
                    break;
                }
            }
        }

        while time_counter < time && note != END_OF_SONG {
            sample_sum = 0.0;
            for i in 0..NUM_STRINGS {
                strings[i].tic();
                sample_sum += strings[i].sample();
            }

            let line = format!("{}\t{}\n", time_counter, sample_sum);
            out_file.write_all(line.as_bytes())?;

            time_counter += STEP;
        }
    }

    println!("\nDone! ({} samples)", (time_counter / STEP) as i32);

    close_files(in_file, out_file)?;

    return Ok(());
}
