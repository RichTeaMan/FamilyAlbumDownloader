use std::{
    collections::HashMap,
    fs::{self, create_dir_all},
    path::Path,
};

use ffmpeg_sidecar::command::FfmpegCommand;
use filetime::FileTime;
use indicatif::{ProgressBar, ProgressStyle};
use log::debug;
use reqwest::{Client, Error, header};

use crate::model::{Mediafile, Root};

const USER_AGENT: &str =
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:144.0) Gecko/20100101 Firefox/144.0";
pub struct FamilyAlbumClient {
    base_address: String,
    password: String,
    output_directory: String,
    auth_token: Option<String>,
    client: Client,
}

pub struct AuthError;

impl FamilyAlbumClient {
    pub fn new(id_token: &str, password: &str, output_directory: &str) -> FamilyAlbumClient {
        let base_address = format!("https://mitene.us/f/{id_token}");

        FamilyAlbumClient {
            base_address,
            password: password.to_string(),
            output_directory: output_directory.to_string(),
            client: Self::build_client(),
            auth_token: None,
        }
    }

    fn build_client() -> Client {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static(USER_AGENT),
        );

        reqwest::Client::builder()
            .default_headers(headers)
            .cookie_store(true)
            .build()
            .unwrap()
    }

    fn rebuild_client(&mut self) {
        self.client = Self::build_client();
    }

    pub async fn login(&mut self) -> Result<(), Error> {
        self.rebuild_client();

        let login_response = self
            .client
            .get(format!(
                "{base_address}/login",
                base_address = self.base_address
            ))
            .send()
            .await?;

        if !login_response.status().is_success() {
            panic!(
                "Invalid login response: {status}",
                status = login_response.status()
            );
        }

        let login_page = login_response.text().await?;

        let auth_token_pattern = r#"name="authenticity_token" value=""#;
        let auth_token_start =
            login_page.as_str().find(auth_token_pattern).unwrap() + auth_token_pattern.len();
        let mut auth_token_vec = Vec::new();

        for (_, c) in login_page.chars().enumerate().skip(auth_token_start) {
            if c == '"' {
                break;
            }
            auth_token_vec.push(c);
        }
        let auth_token: String = auth_token_vec.into_iter().collect();

        let mut params = HashMap::new();
        params.insert("authenticity_token", auth_token.as_str());
        params.insert("session[password]", self.password.as_str());
        params.insert("commit", "Login");
        let response_result = self
            .client
            .post(format!(
                "{base_address}/login",
                base_address = self.base_address
            ))
            .form(&params)
            .send()
            .await;

        if let Ok(response) = response_result {
            if !response.status().is_success() {
                let status_code: String = response.status().to_string();
                eprintln!("Auth token: {auth_token}");
                panic!("Error while authenticating, received {status_code}");
            }
        }
        else {
            panic!("Failed to get response from authentication endpoint. {}", response_result.err().unwrap());
        }

        self.auth_token = Some(auth_token.to_string());

        Ok(())
    }

    pub async fn download_all_media(&mut self) -> Result<(), AuthError> {
        let media_files = self.fetch_images_urls().await;

        let total = media_files.len();

        let mut count = 0;
        let mut download_count = 0;
        println!("Saving media to {dir}...", dir = self.output_directory);
        create_dir_all(self.output_directory.as_str()).unwrap();

        let style = ProgressStyle::default_bar()
            .template("{bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap();
        let progress_bar = ProgressBar::new(media_files.len() as u64);
        progress_bar.set_style(style);

        for media_file in media_files {
            let mut filename_string =
                media_file.suggested_file_name(self.output_directory.as_str());
            let mut filename = filename_string.as_str();

            if !Path::new(filename).exists() {
                if media_file.is_video() {
                    filename_string = format!("{filename}.uncompressed");
                    filename = filename_string.as_str();
                }
                if !Path::new(filename).exists() {
                    self.save_media_file(filename, &media_file).await?;

                    download_count += 1;
                }
            }
            count += 1;
            progress_bar.inc(1);
            debug!("Processed {count} of {total}...");
        }
        progress_bar.finish_with_message(format!(
            "Finished getting media. {download_count} new files."
        ));

        Ok(())
    }

    pub async fn compress_videos(&mut self) -> anyhow::Result<()> {
        ffmpeg_sidecar::download::auto_download()?;

        let mut compress_count = 0;
        println!("Compressing videos...");

        let mut uncompressed_files = Vec::new();
        let paths = fs::read_dir(self.output_directory.as_str()).unwrap();
        for path in paths {
            let os_path = path.unwrap().file_name();
            let path_str = os_path.to_str().unwrap().to_string();
            if path_str.ends_with(".uncompressed") {
                uncompressed_files.push(path_str);
            }
        }

        if uncompressed_files.is_empty() {
            println!("No files to compress.");
            return Ok(());
        }

        let style = ProgressStyle::default_bar()
            .template("{bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap();
        let progress_bar = ProgressBar::new(uncompressed_files.len() as u64);
        progress_bar.set_style(style);

        for uncompressed_file in uncompressed_files {
            let destination_file = uncompressed_file.replace(".uncompressed", "");

            let full_uncompressed =
                format!("{p}/{n}", p = self.output_directory, n = uncompressed_file);
            let full_destination =
                format!("{p}/{n}", p = self.output_directory, n = destination_file);

            FfmpegCommand::new()
                .arg("-i")
                .arg(&full_uncompressed)
                .arg("-movflags")
                .arg("use_metadata_tags")
                .arg("-vcodec")
                .arg("libx264")
                .arg("-crf")
                .arg("28")
                .arg("-y")
                .arg(&full_destination)
                .spawn()?
                // iter seemingly necessary for invocation to finish
                .iter()?
                //.for_each(|event: FfmpegEvent| {
                //    match event {
                //        FfmpegEvent::Log(_level, msg) => {
                //            eprintln!("[ffmpeg] {}", msg); // <- granular log message from stderr
                //        }
                //        _ => {}
                //    }
                //})
                ;
            if Path::new(&full_destination).exists() {
                let uncompressed_file_metadata = fs::metadata(&full_uncompressed).unwrap();
                let mtime = FileTime::from_last_modification_time(&uncompressed_file_metadata);
                filetime::set_file_mtime(&full_destination, mtime).unwrap();

                match fs::remove_file(&full_uncompressed) {
                    Ok(_) => {}
                    Err(e) => eprint!("Unable to delete {full_uncompressed}, {e:?}"),
                }
            }
            compress_count += 1;
            progress_bar.inc(1);
        }

        progress_bar
            .finish_with_message(format!("Finished compressing. {compress_count} new files."));

        Ok(())
    }

    async fn save_media_file(
        &self,
        filename: &str,
        media_file: &Mediafile,
    ) -> Result<(), AuthError> {
        let download_url = media_file.download_url();
        let client = &self.client;
        let file_response_result = client.get(download_url).send().await;
        match file_response_result {
            Ok(file_response) => {
                let status = file_response.status();
                if status.as_u16() == 403 {
                    return Err(AuthError);
                }

                let bytes = file_response.bytes().await.unwrap();
                std::fs::write(filename, bytes).unwrap();

                filetime::set_file_mtime(
                    filename,
                    FileTime::from_unix_time(media_file.took_at.timestamp(), 0),
                )
                .unwrap();

                Ok(())
            }

            Err(e) => panic!("Error: {e}"),
        }
    }

    pub async fn fetch_images_urls(&self) -> Vec<Mediafile> {
        let mut has_images = true;
        let mut page = 1;
        let mut media_urls: Vec<Mediafile> = Vec::new();
        while has_images {
            let model = self.fetch_media_model(page).await.unwrap();
            media_urls.extend(model.media_files);
            has_images = model.has_next;
            page += 1;
        }

        media_urls
    }

    pub async fn fetch_media_model(&self, page: i32) -> Result<Root, Error> {
        let address = format!(
            "{base_address}?page={page}",
            base_address = self.base_address,
            page = page
        );
        let main_response = self.client.get(address).send().await?;

        if main_response.url().as_str().contains("login") {
            panic!("Sent to login page...");
        }

        let main_page = main_response.text().await?;

        let media_start_token = "gon.media=";
        let media_start_opt = main_page.find(media_start_token);
        if media_start_opt.is_none() {
            panic!("Could not find {media_start_token}");
        }

        let start_media_position = media_start_opt.unwrap() + media_start_token.len();
        let mut end_media_position = 0;
        let mut open_braces = 0;
        let mut json_vec = Vec::new();
        for (i, c) in main_page.chars().enumerate().skip(start_media_position) {
            // rebuild json one char at a time. tried using offsets, but variable length unicode characters complicated things.
            json_vec.push(c);
            if c == '{' || c == '[' {
                open_braces += 1;
            } else if c == '}' || c == ']' {
                open_braces -= 1;
                if open_braces == 0 {
                    end_media_position = i;
                    break;
                }
            }
        }

        if end_media_position == 0 {
            panic!("Failed finding end of media.");
        }
        let vstr: String = json_vec.into_iter().collect();

        let json_result = serde_json::from_str::<Root>(vstr.as_str());
        if let Err(json_err) = json_result {
            eprintln!("An error occurred deserialising JSON.");
            eprintln!("Page found:\n{main_page}");
            eprintln!("JSON found:\n{vstr}");
            eprintln!("{json_err:?}");
            panic!();
        }
        Ok(json_result.unwrap())
    }
}
