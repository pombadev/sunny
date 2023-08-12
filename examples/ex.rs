use std::thread;
use std::time;

use console::Style;
use indicatif::{ProgressBar, ProgressStyle};
use sunny::models::Album;

fn main() {
    let albums: Vec<Album> = vec![];

    let total = albums
        .iter()
        .flat_map(|album| album.tracks.iter().map(|_| ()))
        .count();

    let pb = ProgressBar::new(total as u64);

    pb.set_style(
        ProgressStyle::with_template("{prefix} [{bar:57}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("=> "),
    );

    albums.iter().for_each(|album| {
        pb.set_prefix(
            Style::new()
                .cyan()
                .bold()
                .apply_to(album.album.clone())
                .to_string(),
        );

        let _ = album
            .tracks
            .iter()
            .try_for_each(|track| -> anyhow::Result<()> {
                let title = track.name.clone();

                pb.set_message(format!("{title} (📥)"));
                thread::sleep(time::Duration::from_secs(1));

                pb.set_message(format!("{title} (💾)"));
                thread::sleep(time::Duration::from_secs(1));

                pb.set_message(format!("{title} (🎶)"));
                thread::sleep(time::Duration::from_secs(1));

                pb.println(format!(
                    "{} {title}",
                    Style::new().green().bold().apply_to("Downloaded"),
                ));

                pb.inc(1);
                Ok(())
            });
    });

    pb.finish_with_message("all downloaded");
}
