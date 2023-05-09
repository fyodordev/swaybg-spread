use std::fs;
use std::path::{Path, PathBuf};
use clap::Parser;
use std::env::var;

/// Multi-Monitor Wallpaper Utility
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
   /// Image File Path
   #[arg(short, long)]
   image: String,

   /// Cache Path
   #[arg(short, long)]
   cache_path: Option<String>,

   /// Force Resplit even if cache exists
   #[arg(short, long)]
   force_resplit: bool,

   /// Don't downscale base image, even if it's bigger than needed
   #[arg(short, long)]
   no_downscale: bool,

   /// Don't downscale base image, even if it's bigger than needed
   #[arg(short, long)]
   dont_set: bool,

   /// Don't downscale base image, even if it's bigger than needed
   #[arg(short, long)]
   silent: bool,
}

#[derive(Clone, Hash)]
pub struct AppConfig {
    pub image_path: PathBuf,
    pub cache_path: PathBuf,
    pub force_resplit: bool,
    pub no_downscale: bool,
    pub dont_set: bool,
    pub silent: bool,
}

impl AppConfig {
    pub fn new() -> Result<Self, String> {
        // handle args
        let args = Args::parse();

        // check if path is valid
        if ! fs::metadata(Path::new(&args.image)).is_ok() {
            Err("Invalid Path")?
        }

        // create new path for image
        let in_path = AppConfig::check_path(Path::new(&args.image));

        let home_dir = var("HOME").map_err(|_| "HOME env variable not set")?;
        let default_cache_path = format!("{home_dir}/.cache/");
        let cache_path_string = args.cache_path.unwrap_or(default_cache_path);
        let cache_path = AppConfig::check_path(Path::new(&cache_path_string));

        // construct
        Ok(Self {
            image_path: in_path,
            force_resplit: args.force_resplit,
            no_downscale: args.no_downscale,
            dont_set: args.dont_set,
            silent: args.silent,
            cache_path,
        })
    }

    pub fn resolve_cached_fragment_path(&self, monitor_name: &str, hash: &str) -> String {
        // TODO: Work with path properly.
        let base_path = self.cache_path.display();
        format!("{base_path}swaybg_spread_{hash}_{monitor_name}.png")
    }

    // check if target path is a symlink
    fn is_symlink(path: &Path) -> bool {
        if let Ok(metadata) = fs::symlink_metadata(path) {
            metadata.file_type().is_symlink()
        } else {
            false
        }
    }

    // path checker when we need to extend from symlink
    fn check_path(path: &Path) -> PathBuf {
        if AppConfig::is_symlink(path) {
            let parent = path.parent().unwrap_or_else(|| Path::new(""));
            let target = fs::read_link(path).unwrap();
            parent.join(target)
        } else {
            path.to_path_buf()
        }
    }
}
