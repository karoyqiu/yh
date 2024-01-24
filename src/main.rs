use std::{fs::read_dir, process::Command, time::UNIX_EPOCH};

use clap::Parser;
use main_error::{MainError, MainResult};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
  /// The show number
  #[arg(short, long)]
  show: Option<i32>,

  /// The episode number
  #[arg(short, long)]
  episode: Option<i32>,

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

fn guess_episode(dir: &str) -> Result<(i32, i32), MainError> {
  let mut filename = String::new();
  let mut created = UNIX_EPOCH;

  for entry in read_dir(dir)? {
    let entry = entry?;
    let path = entry.path();

    if path.is_file() {
      let meta = entry.metadata()?;
      let ctime = meta.created()?;

      if ctime > created {
        created = ctime;
        filename = path
          .file_stem()
          .unwrap_or_default()
          .to_os_string()
          .into_string()
          .unwrap_or_default();
      }
    }
  }

  if !filename.is_empty() {
    if let Some(pos) = filename.find('-') {
      let show: i32 = filename[0..pos].parse()?;
      let episode: i32 = filename[pos + 1..].parse()?;
      return Ok((show, episode));
    }
  }

  Err("No file found")?
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

  let (show, episode) = if args.show.is_none() || args.episode.is_none() {
    guess_episode(&dir)?
  } else {
    (args.show.unwrap(), args.episode.unwrap())
  };

  println!(
    "Downloading episode {} of show {} into {}",
    episode, show, &dir
  );

  let html = reqwest::blocking::get(format!(
    "http://www.iyinghua.io/v/{}-{}.html",
    show, episode
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
        .arg(format!("{:04}-{:04}.{}", show, episode, ext))
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
