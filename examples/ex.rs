use std::thread;
use std::time::Duration;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

fn main() {
    let m = MultiProgress::new();
    let sty = ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
    )
    .unwrap()
    .progress_chars("##-");

    m.println("starting!").unwrap();

    let mut h = vec![];

    for _ in 0..50 {
        let pb = m.add(ProgressBar::new(128));
        pb.set_style(sty.clone());

        let m_clone = m.clone();
        let h1 = thread::spawn(move || {
            for i in 0..128 {
                pb.set_message(format!("item #{}", i + 1));
                pb.inc(1);
                thread::sleep(Duration::from_millis(50));
            }
            m_clone.println("pb1 is done!").unwrap();
            pb.finish_with_message("done");
        });

        h.push(h1);
    }

    for k in h {
        k.join().unwrap();
    }

    // m.clear().unwrap();
}
