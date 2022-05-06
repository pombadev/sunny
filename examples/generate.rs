use sunny::models::{Album, Track};

fn main() {
    generate().for_each(|album| println!("{:#?}", album))
}

fn generate() -> impl Iterator<Item = Album> {
    (0..10).map(|current| Album {
        artist: current.to_string(),
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
