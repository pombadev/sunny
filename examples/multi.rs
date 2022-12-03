use std::{thread, time::Duration};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use sunny::models::{Album, Track};

fn main() {
    let disco = generate().collect::<Vec<_>>();
    let mb = MultiProgress::new();

    let total = disco.len();

    disco.par_iter().enumerate().for_each(|(mut index, album)| {
        index += 1;

        let pb = mb.add(ProgressBar::new_spinner());

        pb.set_style(
            ProgressStyle::default_spinner()
                .template("╭ {prefix} {spinner}\n╰  {wide_msg}")
                .unwrap(),
        );

        pb.enable_steady_tick(Duration::from_millis(100));

        pb.set_prefix(format!("[{index}/{total}] {}", album.album));

        album.tracks.par_iter().enumerate().for_each(|(_, track)| {
            let msg = track.name.clone();
            pb.set_message(format!("Downloading: {}", &msg));

            thread::sleep(Duration::from_secs(2));
            pb.set_message(format!("Downloaded: {}", &msg));

            thread::sleep(Duration::from_secs(2));
            pb.set_message(format!("Saved to disk: {}", &msg));

            thread::sleep(Duration::from_secs(2));
            pb.set_message(format!("Tagged: {}", &msg));

            thread::sleep(Duration::from_secs(2));
            pb.println(format!("Done: {}", &msg));
            pb.finish();
        });
    });

    mb.println("done").unwrap();

    // mb.clear().unwrap();
}

fn generate() -> impl Iterator<Item = Album> {
    (0..50).map(|current| Album {
        artist: current.to_string(),
        album: format!("#{current} Album"),
        tracks: (1..6)
            .map(|t_idx| Track {
                num: t_idx,
                name: rng().to_string(),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    })
}

fn rng() -> u32 {
    extern "C" {
        fn srand(seeder: u32) -> u32;
        fn rand() -> u32;
    }

    unsafe {
        srand(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .subsec_nanos(),
        );
        rand()
    }
}
