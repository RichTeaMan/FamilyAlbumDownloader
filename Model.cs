using System;
using System.IO;

namespace FamilyAlbumDownload
{

    public class Root
    {
        public bool hasNext { get; set; }
        public bool hasPrev { get; set; }
        public int currentPage { get; set; }
        public Mediafile[] mediaFiles { get; set; }
    }

    public class Mediafile
    {
        public long id { get; set; }
        public string uuid { get; set; }
        public string userId { get; set; }
        public string mediaType { get; set; }
        public string originalHash { get; set; }
        public bool hasComment { get; set; }
        public Comment[] comments { get; set; }
        public object[] footprints { get; set; }
        public DateTime tookAt { get; set; }
        public string audienceType { get; set; }
        public int mediaWidth { get; set; }
        public int mediaHeight { get; set; }
        public int? mediaOrientation { get; set; }
        public double latitude { get; set; }
        public double longitude { get; set; }
        public string mediaDeviceModel { get; set; }
        public string deviceFilePath { get; set; }
        public int videoDuration { get; set; }
        public string contentType { get; set; }
        public string origin { get; set; }
        public bool thumbnailGenerated { get; set; }
        public string expiringUrl { get; set; }
        public string expiringThumbUrl { get; set; }
        public string expiringVideoUrl { get; set; }
        public string DownloadUrl()
        {
            string downloadUrl;
            switch(mediaType)
            {
                case "photo":
                    downloadUrl = expiringUrl;
                    break;
                case "movie":
                    downloadUrl = expiringVideoUrl;
                    break;
                default:
                    throw new Exception($"Unknown media type '{mediaType}'.");
            }
            return downloadUrl;
        }

        public string SuggestedFileName(string basePath = "")
        {
            string extension;
            switch (contentType)
            {
                case "image/jpeg":
                    extension = "jpg";
                    break;
                case "image/png":
                    extension = "png";
                    break;
                case "video/mp4":
                    extension = "mp4";
                    break;
                default:
                    throw new Exception($"Unknown type '{contentType}'.");
            }

            return Path.Combine(basePath, $"{uuid.ToLower()}.{extension}");
        }
    }

    public class Comment
    {
        public int id { get; set; }
        public long mediaFileId { get; set; }
        public User user { get; set; }
        public string body { get; set; }
        public DateTime createdAt { get; set; }
        public DateTime updatedAt { get; set; }
        public bool isDeleted { get; set; }
    }

    public class User
    {
        public string id { get; set; }
        public string nickname { get; set; }
    }

}
