use std::{collections::HashMap, fs::create_dir_all, path::Path};

use fancy_regex::Regex;
use filetime::FileTime;
use reqwest::{header, Client, Error};

use crate::model::{Mediafile, Root};

const USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:85.0) Gecko/20100101 Firefox/85.0";
pub struct FamilyAlbumClient {
    base_address: String,
    password: String,
    output_directory: String,
    auth_token: Option<String>,
    client: Client,
}
impl FamilyAlbumClient {
    pub fn new(id_token: &str, password: &str, output_directory: &str) -> FamilyAlbumClient {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static(USER_AGENT),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .cookie_store(true)
            .build()
            .unwrap();

        let base_address = format!("https://mitene.us/f/{id_token}");

        FamilyAlbumClient {
            base_address: base_address,
            password: password.to_string(),
            output_directory: output_directory.to_string(),
            client: client,
            auth_token: None,
        }
    }

    pub async fn login(&mut self) -> Result<(), Error> {
        if self.auth_token == None {
            let login_response = self
                .client
                .get(format!(
                    "{base_address}/login",
                    base_address = self.base_address
                ))
                .send()
                .await?;

            let login_page = login_response.text().await?;

            let auth_regex = Regex::new(r#"(?<=name="authenticity_token" value=")[^"]+"#).unwrap();
            let auth_match = auth_regex.captures(login_page.as_str()).unwrap();
            match auth_match {
                Some(auth_capture) => {
                    let auth_token = auth_capture.get(0).unwrap().as_str();

                    let mut params = HashMap::new();
                    params.insert("authenticity_token", auth_token);
                    params.insert("session[password]", self.password.as_str());
                    params.insert("commit", "Login");
                    self.client
                        .post(format!(
                            "{base_address}/login",
                            base_address = self.base_address
                        ))
                        .form(&params)
                        .send()
                        .await?;

                    self.auth_token = Some(auth_token.to_string());
                }
                None => panic!("Could not get authentication token."),
            }

            return Ok(());
        }
        return Ok(());
    }

    pub async fn download_all_media(&self) {
        let media_files = self.fetch_images_urls().await;

        let total = media_files.len();

        let mut count = 0;
        let mut download_count = 0;
        println!("Saving media to {dir}...", dir = self.output_directory);
        create_dir_all(self.output_directory.as_str()).unwrap();
        for media_file in media_files {
            let filename_string = media_file.suggested_file_name(self.output_directory.as_str());
            let filename = filename_string.as_str();

            if !Path::new(filename).exists() {
                let download_url = media_file.download_url();
                let file_response = self.client.get(download_url).send().await;

                let bytes = file_response.unwrap().bytes().await.unwrap();
                std::fs::write(filename, bytes).unwrap();

                filetime::set_file_mtime(
                    filename,
                    FileTime::from_unix_time(media_file.took_at.timestamp(), 0),
                )
                .unwrap();

                download_count = download_count + 1;
            }

            count = count + 1;
            println!("Processed {c} of {total}...", c = count, total = total);
        }
        println!("Finished getting media. {download_count} new files.");
    }

    pub async fn fetch_images_urls(&self) -> Vec<Mediafile> {
        let mut has_images = true;
        let mut page = 1;
        let mut media_urls: Vec<Mediafile> = Vec::new();
        while has_images {
            let model = self.fetch_media_model(page).await.unwrap();
            media_urls.extend(model.media_files);
            has_images = model.has_next;
            page = page + 1;
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

        let cdata_regex = Regex::new("(?<=CDATA\\[)[^>]+").unwrap();
        let gon_id_regex = Regex::new(";gon.selfUserId=\"\\d+\";").unwrap();
        let gon_colour_map_regex = Regex::new(";gon.familyUserIdToColorMap={[^}]+}").unwrap();

        let cdata = cdata_regex
            .find_from_pos(main_page.as_str(), 0)
            .unwrap()
            .unwrap()
            .as_str();

        let gon_match = gon_id_regex
            .find_from_pos(cdata, 0)
            .unwrap()
            .unwrap()
            .as_str();

        let gon_colour_map_match = gon_colour_map_regex
            .find_from_pos(cdata, 0)
            .unwrap()
            .unwrap()
            .as_str();

        let json = cdata
            .replace(gon_match, "")
            .replace(gon_colour_map_match, "")
            .replace("window.gon={};gon.media=", "")
            .replace("//]]", "")
            .replace("gon.canSaveMedia=true;", "")
            .trim()
            .to_string();

        let json_result = serde_json::from_str::<Root>(json.as_str()).unwrap();
        Ok(json_result)
    }
}
