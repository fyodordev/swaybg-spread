use crate::AppConfig;
use crate::splitter::ImageFragment;
use std::process::{Command, Stdio};
use std::env::var;


pub fn set_wallpaper(config: &AppConfig, fragments: &[ImageFragment]) -> Result<(), String> {
    if !config.silent {
        // Output in sway config style.
        output_sway_config_lines(fragments);
    }

    if !config.dont_set {
        // Set background for this monitor.
        set_swaymsg(fragments)?;
    }

    Ok(())
}

fn output_sway_config_lines(fragments: &[ImageFragment]) {
    for fragment in fragments {
        // Output in sway config style.
        println!(
            "output {} bg {} fill",
            fragment.monitor_name.to_string(),
            fragment.image_full_path.to_string(),
        );
    }
}


fn set_swaymsg(fragments: &[ImageFragment]) -> Result<(), String> {
    let sway_socket = var("SWAYSOCK")
        .map_err(|_| "MY_VAR environment variable not set".to_string())?;

    for fragment in fragments {
        Command::new("swaymsg")
            .arg("-s")
            .arg(&sway_socket)
            .arg("output")
            .arg(&fragment.monitor_name)
            .arg("bg")
            .arg(&fragment.image_full_path)
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
             )?;
    }
    Ok(())
}

