use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Arc;

use anyhow::Context;
use clap::Parser;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use novelai_api::api::{ai_generate_voice, process_string_for_voice_generation};
use novelai_api::model::Configuration;
use tokio::sync::Semaphore;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    // Your NovelAI Access Token
    #[arg(short, long)]
    token: String,

    /// Input File path
    #[arg(short, long)]
    file: PathBuf,

    /// TTS Voice Seed
    #[arg(short, long, default_value = "Aini")]
    voice: String,

    /// Number of "Threads" to use for downloading
    #[arg(short, long, default_value_t = 10)]
    parallel_threads: usize,

    /// Use ffmpeg to combine the audio files
    #[arg(short, long, default_value_t = false)]
    combine: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let prompt = fs::read_to_string(args.file)?;
    let prompt = process_string_for_voice_generation(prompt);

    let prompt_len = prompt.len();
    let mut tasks = vec![];

    let sem = Arc::new(Semaphore::new(args.parallel_threads));

    fs::create_dir_all("./output")?;

    for (i, text) in prompt.iter().enumerate() {
        // Prepare copy of data for Task
        let text = text.clone();
        let permit = Arc::clone(&sem).acquire_owned().await?;
        let access_token = args.token.clone();
        let voice = args.voice.clone();

        let task = tokio::spawn(async move {
            // Move permit into scope to limit amount of parralel tasks
            let _permit = permit;

            println!("Downloading: {}/{}", i+1, prompt_len);

            let mut conf = Configuration::new();
            conf.bearer_access_token = Some(access_token);

            loop {
                let output = ai_generate_voice(&conf, &text, &voice, -1.0, true, "v2").await;
                if output.is_err() {
                    eprintln!("{:?}", output);
                    continue;
                }

                let file_name = format!("./output/{:05}.webm", i);
                let file = File::create(file_name).await;
                if file.is_err() {
                    eprintln!("{:?}", file);
                    continue;
                }
                let write_result = file.unwrap().write_all(&output.unwrap()).await;
                if write_result.is_err() {
                    eprintln!("{:?}", write_result);
                    continue;
                }
                break;
            }
        });
        tasks.push(task);
    }

    for task in tasks {
        task.await.unwrap();
    }

    if args.combine {
        check_for_ffmpeg().context("NovelAIVoiceCLI: Failed to execute ffmpeg. Have you installed it? In order to use --combine you must have ffmpeg installed!")?;
        concat_with_ffmpeg()?;
    }
    Ok(())
}

fn prepare_concat_for_ffmpeg() -> anyhow::Result<()> {
    // Open the directory
    let dir = fs::read_dir("./output/")?;

    let mut filenames = vec![];

    // Iterate over the entries in the directory
    for entry in dir {
        let entry = entry?;
        let path = entry.path();

        // Get the file name as a string
        if let Some(file_name) = path.file_name() {
            if let Some(file_name_str) = file_name.to_str() {
                // Write the file name to the output file
                filenames.push(file_name_str.to_owned());
            }
        }
    }

    filenames.sort();

    // Open or create the output file
    let mut file = fs::File::create("./output/inputs.txt")?;
    for filename in filenames {
        writeln!(file, "file '{}'", filename)?;
    }

    Ok(())
}

fn check_for_ffmpeg() -> anyhow::Result<()> {
    prepare_concat_for_ffmpeg()?;
    let _ = Command::new("ffmpeg")
        .args(["-version"])
        .stdout(Stdio::inherit())
        .output()?;
    Ok(())
}

fn concat_with_ffmpeg() -> anyhow::Result<()> {
    let _ = Command::new("ffmpeg")
        .current_dir("./output/")
        .args(["-f", "concat", "-i", "inputs.txt", "../output.mp3"])
        .stdout(Stdio::inherit())
        .output()?;
    Ok(())
}
