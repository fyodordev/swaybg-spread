use std::cmp;
use std::env::var;
use image::{GenericImageView, DynamicImage, imageops::FilterType};
use md5::{compute, Digest};
use glob::glob;
use std::fs::remove_file;
use std::path::Path;
use crate::Config;
use crate::outputs::Monitor;
use std::process::{Command, Stdio};

pub struct ResultPaper {
    pub monitor_name: String,
    pub image_full_path: String,
    pub image: DynamicImage,
}

pub struct Splitter {
    hash: String,
    monitors: Vec<Monitor>,
    result_papers: Vec<ResultPaper>,
}

impl Splitter {
    pub fn new() -> Self {
        Self {
            hash: String::new(),
            monitors: Vec::new(),
            result_papers: Vec::new()
        }
    }

    // split main image into two seperate, utilizes scaling
    pub fn run(mut self, config: &Config) -> Result<(), String> {
        // open original input image
        let img = image::open(
            &config.image_path
        ).map_err(
            |_| "failed to open image"
        )?;

        // fetch monitors
        self.monitors = Monitor::new().map_err(
            |err| err.to_string()
        )?;

        // calculate hash
        self.hash = self.hash_config(
            compute(img.as_bytes()),
            config
        );

        // check caches
        let caches_present = self.check_caches();
        // TODO: self.result_papers not being set if cache exists.

        // do we need to resplit
        if
            true
            // config.force_resplit ||
            // ! caches_present
        {
            // cleanup caches first
            self.cleanup_cache();

            // we need to resplit
            self.result_papers = self.perform_split(
                img,
                config,
                format!("{}/.cache/",var("HOME").unwrap())
            ).map_err(
                |err| err.to_string()
            )?;
        }

        for monitor in &self.result_papers {
            if (!config.silent) {
                // Output in sway config style.
                println!("output {} bg {} fill", monitor.monitor_name.to_string(), monitor.image_full_path.to_string());
            }
            if (!config.dont_set) {
                // Set background for this monitor.
                cmd_setbg(&monitor)?;
            }
        }

        Ok(())
    }

    // do the actual splitting
    // TODO: Very slow for some reason (especially resizing and saving), try to optimize.
    fn perform_split(&self, mut img: DynamicImage, config: &Config, save_path: String) -> Result<Vec<ResultPaper>, String> {
        /*
            Calculate Overall Size
            Assuming a monitor can never be negatively offset
            from root, we can say that max width will be the biggest monitor
            with the greatest x-offset, max height will be defined in the same
            way except using y-offset
        */
        let mut result = Vec::new();
        let mut overall_width = 0;
        let mut overall_height = 0;
        for monitor in &self.monitors {
            overall_width = cmp::max(
                monitor.width + monitor.x as u32,
                overall_width
            );
            overall_height = cmp::max(
                monitor.height + monitor.y as u32,
                overall_height
            );
        }

        // check if we need to resize
        // either if user doesn't deny
        // or if image is too small
        if
            config.no_downscale == false
            || img.dimensions().0 < overall_width
            || img.dimensions().1 < overall_height
        {
            // scale image to fit calculated size
            img = img.resize_to_fill(
                overall_width,
                overall_height,
                FilterType::Lanczos3
            );
        }

        // Crop image for screens
        for monitor in &self.monitors {
            let cropped_image = img.crop(
                monitor.x as u32,
                monitor.y as u32,
                monitor.width,
                monitor.height
            );
            result.push(
                ResultPaper { 
                    monitor_name: format!("{}", &monitor.name),
                    image_full_path: format!(
                        "{}rwps_{}_{}.png",
                        save_path,
                        &self.hash[2..32],
                        format!("{}", &monitor.name),
                    ),
                    image: cropped_image
                }
            );
        }

        // save our result images
        for paper in &result {
            paper.image.save(
                &paper.image_full_path
            ).map_err(
                |err| err.to_string()
            )?;
        }

        Ok(result)
    }

    fn hash_config(&self, img_hash: Digest, config: &Config) -> String {
        // new hash string
        let mut hash_string = String::new();

        // loop over config params and add to string
        for monitor in &self.monitors {
            hash_string.push_str(&monitor.to_string());
        }

        // compute and assemble hash
        // we also factor in downscaling as images
        // might be different if we dont downscale
        format!(
            "# {:?}{:?}{:?}\n",
            img_hash,
            compute(config.no_downscale.to_string()),
            compute(hash_string.as_bytes())
        )
    }

    fn cleanup_cache(&self) {
        // wildcard search for our
        // images and delete them
        for entry in glob(
            &format!(
                "{}/.cache/rwps_*",
                var("HOME").unwrap()
            )
        ).unwrap() {
            if let Ok(path) = entry {
                // yeet any file that we cached
                remove_file(path).unwrap();
            }
        }
    }

    fn check_caches(&self) -> bool {

        // what we search for
        let base_format = format!(
            "{}/.cache/rwps_{}",
            var("HOME").unwrap(),
            &self.hash[2..32]
        );

        // check for every monitor
        for monitor in &self.monitors {
            let image_path = format!(
                "{}_{}.png",
                base_format,
                monitor.name
            );
            // check if a cached image exists
            if ! Path::new(&image_path).exists() {
                // we're missing an image, regenerate
                return false;
            }
        }

        // if we pass, we're good
        true
    }
}


fn cmd_setbg(wallpaper: &ResultPaper) -> Result<(), String> {

    let sway_socket = var("SWAYSOCK")
        .map_err(|_| "MY_VAR environment variable not set".to_string())?;

    Command::new("swaymsg")
        .arg("-s")
        .arg(&sway_socket)
        .arg("output")
        .arg(&wallpaper.monitor_name)
        .arg("bg")
        .arg(&wallpaper.image_full_path)
        .arg("fill")
        .stdout(Stdio::null())
        .status()
        //.and_then(|exit_status| exit_status.exit_ok().map_err(|e| e.to_string()))
        //.map_err(|err| err.to_string())
        .map_or_else(
            |err| Err(err.to_string()),
            |exit_status| if exit_status.success() {
                Ok(())
            } else {
                Err("Exit with error".to_string())
            }
         )
}

