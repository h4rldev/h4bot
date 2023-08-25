use std::process::exit;

fn update() {
    if !std::process::Command::new("add-apt-repository")
        .arg("ppa:tomtomtom/yt-dlp")
        .status()
        .expect("Failure!")
        .success()
    {
        println!("Failed to add yt-dlp repository");
        exit(1);
    }
    if !std::process::Command::new("apt")
        .arg("update")
        // the apt package that a dependency of my project needs to compile
        // can add more here
        .status()
        .expect("Failure!")
        .success()
    {
        println!("Failed to update apt");
        exit(1);
    }
}

fn main() {
    // Install external dependency (in the shuttle container only)
    if std::env::var("HOSTNAME")
        .unwrap_or_default()
        .contains("shuttle")
    {
        loop {
            match std::process::Command::new("apt")
                .arg("install")
                .arg("-y")
                .arg("libopus-dev")
                .arg("opus")
                .arg("ffmpeg")
                .arg("yt-dlp")
                // the apt package that a dependency of my project needs to compile
                // can add more here
                .status()
            {
                Ok(yup) => {
                    let _ = yup.success();
                    break;
                }
                Err(why) => {
                    println!("Failed to install dependencies: {}", why);
                    update();
                    continue;
                }
            }
        }
    }
}
