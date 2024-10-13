use std::path::PathBuf;
use std::thread;

use clap::Parser;
use lilac::Lilac;
use miette::{Context, IntoDiagnostic};
use rodio::{Sink, Source};

type Result = miette::Result<()>;
const OK: Result = Result::Ok(());

mod interactive;
mod transcode;

/// LILAC playback and transcoding utility
///
/// If neither of the subcommands are detected,
/// opens an interactive player and load the provided files.
#[derive(Parser)]
enum Opt {
    /// Plays a LILAC file
    Play {
        /// File to play
        #[clap(name = "FILE")]
        file: PathBuf,
        /// Playback volume
        ///
        /// Should be anywhere between 0.0 and 1.0 inclusively
        #[clap(short, long, name = "VOLUME", default_value = "1.0")]
        volume: f32,
    },
    /// Transcodes a file to or from LILAC
    ///
    /// Supports transcoding from MP3, FLAC,
    /// OGG and WAV, and transcoding to WAV.
    /// Input and output formats are automatically inferred
    Transcode {
        /// Glob matching the input files
        #[clap(name = "GLOB")]
        glob: String,
        /// Output files naming pattern
        ///
        /// %F is replaced with the input filename without extension,
        /// %E with the output format extension,
        /// %e with the input format extension,
        /// %T with the song title,
        /// %A with the song artist,
        /// %a with the song album.
        #[clap(name = "PATTERN", default_value = "%F.%E")]
        output: String,
        /// Keep input files after transcoding
        #[clap(short, long)]
        keep: bool,
    },

    Interactive {
        queue: Vec<String>,
    },
}

fn main() -> miette::Result<()> {
    match Opt::parse() {
        Opt::Play { file, volume } => play(file, volume),
        Opt::Transcode { glob, output, keep } => transcode::main(glob, output, keep),
        Opt::Interactive { queue } => interactive::main(queue),
    }?;

    Ok(())
}

fn play(file: PathBuf, volume: f32) -> Result {
    let lilac = Lilac::read_file(file)?;
    println!(
        "Now playing {} by {} on {}",
        lilac.title(),
        lilac.artist(),
        lilac.album(),
    );

    let (_stream, device) = rodio::OutputStream::try_default()
        .into_diagnostic()
        .context("no audio device")?;

    let sink = Sink::try_new(&device)
        .into_diagnostic()
        .context("failed to create sink")?;

    let source = lilac.source();
    let duration = source.total_duration().unwrap();

    sink.set_volume(volume);
    sink.append(source);
    sink.play();

    thread::sleep(duration);
    OK
}
