use std::collections::HashMap;
use std::time::Duration;

use curl::easy::{Easy2, Handler, WriteError};
use curl::multi::{Easy2Handle, Multi};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

const URLS: &[&str] = &[
    // "https://freetestdata.com/wp-content/uploads/2022/02/Free_Test_Data_7MB_MP4.mp4",
    // "https://freetestdata.com/wp-content/uploads/2022/02/Free_Test_Data_10MB_MP4.mp4",
    // "https://freetestdata.com/wp-content/uploads/2022/02/Free_Test_Data_15MB_MP4.mp4",
    "https://dl.google.com/go/go1.19.3.linux-amd64.tar.gz",
];

struct Collector(Vec<u8>, ProgressBar);

impl Handler for Collector {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        self.0.extend_from_slice(data);
        Ok(data.len())
    }

    fn progress(&mut self, dltotal: f64, dlnow: f64, _: f64, _: f64) -> bool {
        // println!("dltotal: {dltotal}\ndlnow: {dlnow}\nultotal: {ultotal}\nulnow: {ulnow}");
        self.1.set_position(dlnow as u64);
        self.1.set_length(dltotal as u64);
        true
    }
}

fn download(
    multi: &mut Multi,
    token: usize,
    url: &str,
    mb: MultiProgress,
) -> Result<Easy2Handle<Collector>, Box<dyn std::error::Error>> {
    let version = curl::Version::get();
    let pb = mb.add(ProgressBar::new(0).with_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})",
        )
        .unwrap()
        .progress_chars("#>-"),
    ));

    let mut request = Easy2::new(Collector(Vec::new(), pb));
    request.url(url)?;
    request.useragent(&format!("curl/{}", version.version()))?;
    request.progress(true)?;
    // request.progr

    let mut handle = multi.add2(request)?;

    handle.set_token(token)?;
    Ok(handle)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mb = MultiProgress::new();

    let mut multi = Multi::new();
    let mut handles = URLS
        .iter()
        .enumerate()
        .map(|(token, url)| Ok((token, download(&mut multi, token, url, mb.clone())?)))
        .collect::<Result<HashMap<_, _>, Box<dyn std::error::Error>>>()?;

    let mut still_alive = true;
    while still_alive {
        // We still need to process the last messages when
        // `Multi::perform` returns "0".
        if multi.perform()? == 0 {
            still_alive = false;
        }

        multi.messages(|message| {
            let token = message.token().expect("failed to get the token");
            let handle = handles
                .get_mut(&token)
                .expect("the download value should exist in the HashMap");

            match message
                .result_for2(handle)
                .expect("token mismatch with the `EasyHandle`")
            {
                Ok(()) => {
                    // let http_status = handle
                    //     .response_code()
                    //     .expect("HTTP request finished without status code");
                    handle.get_ref().1.finish();

                    // println!(
                    //     "R: Transfer succeeded (Status: {}) {} (Download length: {})",
                    //     http_status,
                    //     URLS[token],
                    //     handle.get_ref().0.len()
                    // );
                }
                Err(error) => {
                    println!("E: {} - <{}>", error, URLS[token]);
                }
            }
        });

        if still_alive {
            // The sleeping time could be reduced to allow other processing.
            // For instance, a thread could check a condition signalling the
            // thread shutdown.
            multi.wait(&mut [], Duration::from_secs(60))?;
        }
    }

    mb.println("done!")?;

    Ok(())
}
