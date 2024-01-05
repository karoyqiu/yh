use std::process::Command;

use clap::Parser;
use main_error::MainResult;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
  /// The show number
  #[arg(short, long)]
  show: i32,

  /// The episode number
  #[arg(short, long)]
  episode: i32,

  /// The output directory, default to current directory
  #[arg(short, long)]
  directory: Option<String>,

  /// Segment download concurrency
  #[arg(short, long, default_value_t = 8)]
  parallel: i32,

  /// The config file
  #[arg(short, long)]
  config: Option<String>,
}

#[derive(Default, Serialize, Deserialize)]
struct Config {
  streamlink: String,
}

/// 获取当前目录
fn get_current_dir() -> String {
  if let Ok(path) = std::env::current_dir() {
    if let Some(s) = path.to_str() {
      return s.to_string();
    }
  }

  String::default()
}

fn main() -> MainResult {
  let config = confy::get_configuration_file_path("yh", None)?;
  println!("Default config file: {}", &config.display());

  let args = Args::parse();
  let dir = args.directory.unwrap_or(get_current_dir());
  let config: Config = confy::load("yh", None)?;

  if config.streamlink.is_empty() {
    eprintln!("No streamlink.");
    return Ok(());
  }

  println!(
    "Downloading episode {} of show {} into {}",
    args.episode, args.show, &dir
  );

  let html = reqwest::blocking::get(format!(
    "http://www.iyinghua.io/v/{}-{}.html",
    args.show, args.episode
  ))?
  .text()?;

  let doc = Html::parse_document(&html);
  let playbox = Selector::parse("#playbox")?;

  if let Some(playbox) = doc.select(&playbox).next() {
    if let Some(vid) = playbox.attr("data-vid") {
      let mut parts = vid.split('$');
      let url = parts.next().unwrap();
      let ext = parts.next().unwrap();
      println!("URL: {}, ext: {}", url, ext);

      Command::new(config.streamlink)
        .current_dir(dir)
        .arg("--stream-segment-threads")
        .arg(args.parallel.to_string())
        .arg("-o")
        .arg(format!("{:04}-{:04}.{}", args.show, args.episode, ext))
        .arg(url)
        .arg("best")
        .status()?;
    } else {
      eprintln!("No VID.");
    }
  } else {
    eprintln!("No playbox.");
  }

  Ok(())
}
