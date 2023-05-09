use std::cmp;
use image::{GenericImageView, DynamicImage, imageops::FilterType};
use std::path::PathBuf;
use crate::AppConfig;
use crate::outputs::Monitor;
use crate::setter::set_wallpaper;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hasher, Hash};


pub struct ImageFragment {
    pub monitor_name: String,
    pub image_full_path: String,
    pub image: DynamicImage,
}

pub struct Splitter {
    app_config: AppConfig,
    monitors: Vec<Monitor>,
}

impl Splitter {
    pub fn new(
        app_config: &AppConfig,
        monitors: &[Monitor],
    ) -> Self {
        Self {
            app_config: app_config.clone(),
            monitors: monitors.to_vec(),
        }
    }

    // Do something with the monitors:
    // Use image path in config to setsplit wallpaper. 
    pub fn run(&self) -> Result<(), String> {

        let fragments = self.get_split_image(&self.app_config.image_path)?;

        set_wallpaper(&self.app_config, fragments.as_slice())?;

        Ok(())
    }

    // From image path, get paths of image fragments corresponding to monitors. 
    // Use caching.
    pub fn get_split_image(&self, image_path: &PathBuf) -> Result<Vec<ImageFragment>, String> {

        // open original input image
        let image = image::open(image_path)
            .map_err(|_| "failed to open image")?;


        // Calculate hash of everything that makes a split unique.
        let mut hasher = DefaultHasher::new();
        for monitor in &self.monitors {
            monitor.hash(&mut hasher)
        }
        self.app_config.no_downscale.hash(&mut hasher);
        image.as_bytes().hash(&mut hasher);
        let image_hash = hasher.finish().to_string();


        // return either from cache or split.
        if let Ok(result_fragments) = self.load_from_cache(image_hash.as_str()) {
            // Return from cache.
            return Ok(result_fragments);
        } else {
            // Calculate fragments and save to file.
            return self.perform_split(image, image_hash.as_str());
        }
    }

    // do the actual splitting
    // TODO: Very slow for some reason (especially resizing and saving), try to optimize.
    fn perform_split(&self, mut image: DynamicImage, image_hash: &str) -> Result<Vec<ImageFragment>, String> {
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
            self.app_config.no_downscale == false
            || image.dimensions().0 < overall_width
            || image.dimensions().1 < overall_height
        {
            // scale image to fit calculated size
            image = image.resize_to_fill(
                overall_width,
                overall_height,
                FilterType::Lanczos3
            );
        }

        // Crop image for screens
        for monitor in &self.monitors {
            let cropped_image = image.crop(
                monitor.x as u32,
                monitor.y as u32,
                monitor.width,
                monitor.height
            );

            let fragment_save_path = self.app_config.resolve_cached_fragment_path(
                monitor.name.as_str(),
                image_hash 
            );

            cropped_image.save(fragment_save_path.as_str()).map_err(
                |err| err.to_string()
            )?;

            result.push(
                ImageFragment { 
                    monitor_name: monitor.name.clone(),
                    image_full_path: fragment_save_path,
                    image: cropped_image
                }
            );
        }

        Ok(result)
    }

    fn load_from_cache(&self, hash: &str) -> Result<Vec<ImageFragment>, String> {
        let mut result_fragments = Vec::new();

        // check for every monitor
        for monitor in &self.monitors {
            let image_full_path = self.app_config.resolve_cached_fragment_path(&monitor.name, &hash);

            // try to open cached image
            let image = image::open(
                &image_full_path
            ).map_err(
                |_| "failed to open image"
            )?;

            result_fragments.push(
                ImageFragment { 
                    monitor_name: monitor.name.clone(),
                    image_full_path,
                    image,
                }
            );
        }
        Ok(result_fragments)
    }
}


