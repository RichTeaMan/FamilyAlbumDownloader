pub use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct Root {
    #[serde(rename = "hasNext")]
    pub has_next: bool,
    #[serde(rename = "hasPrev")]
    pub has_prev: bool,
    #[serde(rename = "currentPage")]
    pub current_page: i32,
    #[serde(rename = "mediaFiles")]
    pub media_files: Vec<Mediafile>,
}

#[derive(Serialize, Deserialize)]
pub struct Mediafile {
    pub id: i64,
    pub uuid: String,
    #[serde(rename = "userId")]
    pub user_id: String,
    #[serde(rename = "mediaType")]
    pub media_type: String,
    #[serde(rename = "originalHash")]
    pub original_hash: String,
    #[serde(rename = "hasComment")]
    pub has_comment: bool,
    pub comments: Vec<Comment>,
    #[serde(rename = "tookAt")]
    pub took_at: DateTime<Utc>,
    #[serde(rename = "audienceType")]
    pub audience_type: String,
    #[serde(rename = "mediaWidth")]
    pub media_width: i32,
    #[serde(rename = "mediaHeight")]
    pub media_height: i32,
    pub latitude: f64,
    pub longitude: f64,
    #[serde(rename = "mediaDeviceModel")]
    pub media_device_model: Option<String>,
    #[serde(rename = "deviceFilePath")]
    pub device_file_path: Option<String>,
    #[serde(rename = "videoDuration")]
    pub video_duration: i32,
    #[serde(rename = "contentType")]
    pub content_type: String,
    pub origin: String,
    #[serde(rename = "thumbnailGenerated")]
    pub thumbnail_generated: bool,
    #[serde(rename = "expiringUrl")]
    pub expiring_url: String,
    #[serde(rename = "expiringThumbUrl")]
    pub expiring_thumb_url: String,
    #[serde(rename = "expiringVideoUrl")]
    pub expiring_video_url: Option<String>,
}

impl Mediafile {
    pub fn is_video(&self) -> bool {
        self.content_type == "video/mp4"
    }

    pub fn download_url(&self) -> String {
        let download_url;
        if self.media_type == "photo" {
            download_url = self.expiring_url.clone();
        } else if self.media_type == "movie" {
            let expiring_video_url = self.expiring_video_url.clone().unwrap();
            download_url = format!("{expiring_video_url}/download")
                .replace("media_files_playlist", "media_files");
        } else {
            let media_type = self.media_type.clone();
            panic!("Unknown media type '{media_type}'.");
        }
        download_url
    }

    pub fn suggested_file_name(&self, base_path: &str) -> String {
        let extension;
        if self.content_type == "image/jpeg" {
            extension = "jpg";
        } else if self.content_type == "image/png" {
            extension = "png";
        } else if self.content_type == "video/mp4" {
            extension = "mp4";
        } else {
            let content_type = self.content_type.clone();
            panic!("Unknown type '{content_type}'.");
        }

        let uuid = self.uuid.to_lowercase();
        let path = Path::new(base_path).join(format!("{uuid}.{extension}"));
        let res = path.to_str();
        match res {
            Some(s) => s.to_string(),
            None => "".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Comment {
    pub id: i32,
    #[serde(rename = "mediaFileId")]
    pub media_file_id: i64,
    pub user: User,
    pub body: String,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
    #[serde(rename = "isDeleted")]
    pub is_deleted: bool,
}

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub nickname: String,
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::Mediafile;

    #[test]
    fn null_test() {
        let media_file_json = r#"
            {
                "id":1,
                "uuid":"307c0d3e-1d97-11ed-861d-0242ac120002",
                "userId":"1",
                "mediaType":"movie",
                "originalHash":"7",
                "hasComment":false,
                "comments":[],
                "footprints":[],
                "tookAt":"2022-05-01T00:00:00+00:00",
                "audienceType":"family",
                "mediaWidth":720,
                "mediaHeight":1280,
                "mediaOrientation":0,
                "latitude":55.0,
                "longitude":4.0,
                "mediaDeviceModel":null,
                "deviceFilePath":null,
                "videoDuration":30,
                "contentType":"video/mp4",
                "origin":"seasonal_osm",
                "thumbnailGenerated":true,
                "expiringUrl":"https://example.jpg",
                "expiringVideoUrl":"https://example.mp4",
                "expiringThumbUrl":"https://example.jpg"
             }
            "#;
        let json_result = serde_json::from_str::<Mediafile>(media_file_json).unwrap();

        assert!(json_result.media_device_model.is_none());
        assert!(json_result.device_file_path.is_none());
        let date_time = chrono::Utc.with_ymd_and_hms(2022, 5, 1, 0, 0, 0).unwrap();
        assert_eq!(date_time, json_result.took_at);
    }
}
