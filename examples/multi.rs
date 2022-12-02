use std::{thread, time::Duration};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use sunny::models::{Album, Track};

fn main() {
    let disco = generate().collect::<Vec<_>>();
    let mb = MultiProgress::new();

    let total = disco.len();

    mb.println(format!("{total} album(s) found")).unwrap();

    disco.par_iter().enumerate().for_each(|(mut index, album)| {
        index += 1;

        let pb = mb.add(ProgressBar::new_spinner());

        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{prefix} {spinner} {wide_msg}")
                .unwrap(),
        );

        pb.enable_steady_tick(Duration::from_millis(100));

        pb.set_prefix(format!("[{index}/{total}] {}", album.album));

        album
            .tracks
            .par_iter()
            .enumerate()
            .for_each(|(tindex, track)| {
                // let pb
                let msg = track.name.clone();
                pb.set_message(format!("\n ├─ Downloading: {}", &msg));

                thread::sleep(Duration::from_secs(2));
                pb.set_message(format!("\n ├─ Downloaded: {}", &msg));

                thread::sleep(Duration::from_secs(2));
                pb.set_message(format!("\n ├─ Saved to disk: {}", &msg));

                thread::sleep(Duration::from_secs(2));
                pb.set_message(format!("\n ├─ Tagged: {}", &msg));

                thread::sleep(Duration::from_secs(2));
                pb.println(format!("\n ├─ Done: {}", &msg));
                pb.finish();
            });
    });

    mb.println("done").unwrap();

    // mb.clear().unwrap();
}

fn generate() -> impl Iterator<Item = Album> {
    (0..10).map(|current| Album {
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
