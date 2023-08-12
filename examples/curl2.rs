use std::collections::HashMap;
use std::time::Duration;

use console::Style;
use curl::easy::{Easy2, Handler, WriteError};
use curl::multi::{Easy2Handle, Multi};
use indicatif::ProgressBar;

const URLS: &[&str] = &[
    // "https://freetestdata.com/wp-content/uploads/2022/02/Free_Test_Data_7MB_MP4.mp4",
    // "https://freetestdata.com/wp-content/uploads/2022/02/Free_Test_Data_10MB_MP4.mp4",
    // "https://freetestdata.com/wp-content/uploads/2022/02/Free_Test_Data_15MB_MP4.mp4",
    "https://freetestdata.com/wp-content/uploads/2022/02/Free_Test_Data_1MB_MP4.mp4",
    "https://freetestdata.com/wp-content/uploads/2022/02/Free_Test_Data_1MB_MP4.mp4",
    "https://freetestdata.com/wp-content/uploads/2022/02/Free_Test_Data_1MB_MP4.mp4",
    "https://freetestdata.com/wp-content/uploads/2022/02/Free_Test_Data_1MB_MP4.mp4",
    "https://freetestdata.com/wp-content/uploads/2022/02/Free_Test_Data_1MB_MP4.mp4",
    "https://freetestdata.com/wp-content/uploads/2022/02/Free_Test_Data_1MB_MP4.mp4",
    "https://freetestdata.com/wp-content/uploads/2022/02/Free_Test_Data_1MB_MP4.mp4",
    "https://freetestdata.com/wp-content/uploads/2022/02/Free_Test_Data_1MB_MP4.mp4",
    "https://freetestdata.com/wp-content/uploads/2022/02/Free_Test_Data_1MB_MP4.mp4",
    "https://freetestdata.com/wp-content/uploads/2022/02/Free_Test_Data_1MB_MP4.mp4",
    // "https://dl.google.com/go/go1.19.3.linux-amd64.tar.gz",
    // "https://dl.google.com/go/go1.19.3.linux-amd64.tar.gz",
    // "https://dl.google.com/go/go1.19.3.linux-amd64.tar.gz",
    // "https://dl.google.com/go/go1.19.3.linux-amd64.tar.gz",
    // "https://dl.google.com/go/go1.19.3.linux-amd64.tar.gz",
    // "https://dl.google.com/go/go1.19.3.linux-amd64.tar.gz",
    // "https://dl.google.com/go/go1.19.3.linux-amd64.tar.gz",
    // "https://dl.google.com/go/go1.19.3.linux-amd64.tar.gz",
    // "https://dl.google.com/go/go1.19.3.linux-amd64.tar.gz",
    // "https://dl.google.com/go/go1.19.3.linux-amd64.tar.gz",
    // "https://dl.google.com/go/go1.19.3.linux-amd64.tar.gz",
    // "https://dl.google.com/go/go1.19.3.linux-amd64.tar.gz",
    // "https://dl.google.com/go/go1.19.3.linux-amd64.tar.gz",
    // "https://dl.google.com/go/go1.19.3.linux-amd64.tar.gz",
    // "https://dl.google.com/go/go1.19.3.linux-amd64.tar.gz",
    // "https://dl.google.com/go/go1.19.3.linux-amd64.tar.gz",
    // "https://dl.google.com/go/go1.19.3.linux-amd64.tar.gz",
    // "https://dl.google.com/go/go1.19.3.linux-amd64.tar.gz",
    // "https://dl.google.com/go/go1.19.3.linux-amd64.tar.gz",
    // "https://dl.google.com/go/go1.19.3.linux-amd64.tar.gz",
    // "https://dl.google.com/go/go1.19.3.linux-amd64.tar.gz",
    // "https://dl.google.com/go/go1.19.3.linux-amd64.tar.gz",
    // "https://dl.google.com/go/go1.19.3.linux-amd64.tar.gz",
    // "https://dl.google.com/go/go1.19.3.linux-amd64.tar.gz",
    // "https://dl.google.com/go/go1.19.3.linux-amd64.tar.gz",
    // "https://dl.google.com/go/go1.19.3.linux-amd64.tar.gz",
];

struct Collector(Vec<u8>, ProgressBar);

impl Handler for Collector {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        self.0.extend_from_slice(data);
        Ok(data.len())
    }
}

fn download(
    multi: &mut Multi,
    token: usize,
    url: &str,
    mb: ProgressBar,
) -> Result<Easy2Handle<Collector>, Box<dyn std::error::Error>> {
    let version = curl::Version::get();
    // let pb = mb.add(
    //     ProgressBar::new(0).with_style(
    //         ProgressStyle::with_template(
    //             "╭ {prefix}\n╰ [{bar:40.cyan/blue}] {bytes}/{total_bytes} {bytes_per_sec} (eta {eta})",
    //         )
    //         .unwrap()
    //         .progress_chars("#>-"),
    //     ),
    // );

    // pb.set_prefix(url[..].to_string());
    mb.set_prefix(
        Style::new()
            .cyan()
            .bold()
            .apply_to(url[..].to_string())
            .to_string(),
    );

    let mut request = Easy2::new(Collector(Vec::new(), mb));
    request.url(url)?;
    request.useragent(&format!("curl/{}", version.version()))?;

    let mut handle = multi.add2(request)?;

    handle.set_token(token)?;
    Ok(handle)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let mb = Arc::new(ProgressBar::new(URLS.len() as u64));
    let mb = ProgressBar::new(URLS.len() as u64);

    mb.set_style(
        indicatif::ProgressStyle::with_template("{prefix} [{bar:57}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("=> "),
    );

    // let mb = MultiProgress::new();

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
                    let pb = handle.get_ref().1.clone();
                    let title = URLS[token];

                    pb.set_message(format!("{title} (📥)"));
                    std::thread::sleep(std::time::Duration::from_secs(1));

                    pb.set_message(format!("{title} (💾)"));
                    std::thread::sleep(std::time::Duration::from_secs(1));

                    pb.set_message(format!("{title} (🎶)"));
                    std::thread::sleep(std::time::Duration::from_secs(1));

                    // let http_status = handle
                    //     .response_code()
                    //     .expect("HTTP request finished without status code");

                    pb.println(format!(
                        "{} {title}",
                        Style::new().green().bold().apply_to("Downloaded"),
                    ));

                    pb.inc(1);

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
            multi.wait(&mut [], Duration::from_secs(1))?;
        }
    }

    // mb.println("done!")?;
    mb.finish_with_message("all done");

    Ok(())
}
