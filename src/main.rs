use clap::Parser;

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

fn main() {
  let args = Args::parse();
  let dir = args.directory.unwrap_or(get_current_dir());

  println!(
    "Downloading episode {} of show {} into {}",
    args.episode, args.show, &dir
  );
}
