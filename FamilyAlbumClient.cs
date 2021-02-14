using Newtonsoft.Json;
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Net;
using System.Net.Http;
using System.Text.RegularExpressions;
using System.Threading.Tasks;

namespace FamilyAlbumDownload
{
    public class FamilyAlbumClient : IDisposable
    {
        private readonly string baseAddress;

        private readonly string password;

        private readonly string outputDirectory;

        private bool disposedValue;

        private HttpClient client;

        private CookieContainer cookieContainer;

        private string authToken = null;


        public FamilyAlbumClient(string idToken, string password, string outputDirectory)
        {
            var handler = new HttpClientHandler();
            cookieContainer = new CookieContainer();
            handler.CookieContainer = cookieContainer;

            client = new HttpClient(handler);
            client.DefaultRequestHeaders.Add("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:85.0) Gecko/20100101 Firefox/85.0");
            client.DefaultRequestHeaders.Add("Accept", "text/html,application/xhtml+xml,application/xml;q = 0.9,image/webp,*/*;q=0.8");
            client.DefaultRequestHeaders.Add("Accept-Language", "en-GB,en;q=0.5");

            baseAddress = $"https://mitene.us/f/{idToken}";
            this.password = password;
            this.outputDirectory = outputDirectory;
        }

        public async Task Login()
        {
            if (authToken == null)
            {
                using var loginResponse = await client.GetAsync($"{baseAddress}/login");
                loginResponse.EnsureSuccessStatusCode();
                var loginPage = await loginResponse.Content.ReadAsStringAsync();

                var authTokenRegex = new Regex("(?<=name=\"authenticity_token\" value=\")[^\"]+");
                var matches = authTokenRegex.Match(loginPage);
                if (!matches.Success)
                {
                    throw new Exception("Could not get authentication token.");
                }
                authToken = matches.Value;

                var formContent = new FormUrlEncodedContent(new[]
                {
                    new KeyValuePair<string, string>("authenticity_token", authToken),
                    new KeyValuePair<string, string>("session[password]", password),
                    new KeyValuePair<string, string>("commit", "Login")
                });

                using var loginPostResponse = await client.PostAsync($"{baseAddress}/login", formContent);
                loginPostResponse.EnsureSuccessStatusCode();

                var cookies = cookieContainer.GetCookies(new Uri(baseAddress));

                string[] requiredCookies = { "follower_access_token", "_mitene_session", "follower_session_token" };
                foreach (var requiredCookieName in requiredCookies)
                {
                    if (!cookies.Any(c => c.Name.StartsWith(requiredCookieName)))
                    {
                        throw new Exception($"Missing cookie '{requiredCookieName}'.");
                    }
                }
            }
        }
        public async Task DownloadAllMedia()
        {
            var mediaFiles = await FetchImagesUrls();
            var exifClient = new ExifClient();

            int count = 0;
            int downloadCount = 0;
            Console.WriteLine($"Saving media to {outputDirectory}...");
            Directory.CreateDirectory(outputDirectory);
            foreach (var mediaFile in mediaFiles)
            {
                string filename = mediaFile.SuggestedFileName(outputDirectory);

                if (!File.Exists(filename))
                {

                    using (var mediaStream = await client.GetStreamAsync(mediaFile.DownloadUrl()))
                    using (var fs = File.Create(filename))
                    {
                        mediaStream.CopyTo(fs);
                    }

                    File.SetCreationTime(filename, mediaFile.tookAt);
                    File.SetLastAccessTime(filename, mediaFile.tookAt);

                    try
                    {
                        await exifClient.AddExif(filename, mediaFile.tookAt, mediaFile.latitude, mediaFile.longitude, mediaFile.mediaDeviceModel);
                    }
                    catch (Exception ex)
                    {
                        Console.WriteLine($"Error while updating meta fdata for '{filename}'");
                        Console.WriteLine(ex);
                    }

                    downloadCount++;
                }

                count++;
                Console.Write($"\rProcessed {count} of {mediaFiles.Count}...");
            }
            Console.WriteLine();
            Console.WriteLine($"Finished getting media. {downloadCount} new files.");
        }

        public async Task<List<Mediafile>> FetchImagesUrls()
        {
            bool hasImages = true;
            int page = 1;
            List<Mediafile> mediaUrls = new List<Mediafile>();
            while (hasImages)
            {
                var model = await FetchMediaModel(page);
                mediaUrls.AddRange(model.mediaFiles);
                hasImages = model.hasNext;
                page++;
            }

            return mediaUrls.ToList();
        }

        public async Task<Root> FetchMediaModel(int page = 1)
        {
            await Login();

            using var mainResponse = await client.GetAsync($"{baseAddress}?page={page}");

            if (mainResponse.RequestMessage.RequestUri.ToString().Contains("login"))
            {
                throw new Exception("Sent to login page...");
            }


            mainResponse.EnsureSuccessStatusCode();

            var mainPage = await mainResponse.Content.ReadAsStringAsync();

            var cdataRegex = new Regex("(?<=CDATA\\[)[^>]+", RegexOptions.Multiline);
            var gonIdRegex = new Regex(";gon.selfUserId=\"\\d+\";");
            var gonColourMapRegex = new Regex(";gon.familyUserIdToColorMap={[^}]+}");

            var cDataMatch = cdataRegex.Match(mainPage);
            if (!cDataMatch.Success)
            {
                throw new Exception("Could not match JSON from CDATA.");
            }
            string cdata = cDataMatch.Value;

            var gonMatch = gonIdRegex.Match(cdata);
            if (!gonMatch.Success)
            {
                throw new Exception("Could not match gon");
            }
            cdata = cdata.Replace(gonMatch.Value, "");

            var gonColourMapMatch = gonColourMapRegex.Match(cdata);
            cdata = cdata.Replace(gonColourMapMatch.Value, "");

            string json = cdata.Replace("window.gon={};gon.media=", "").Replace("//]]", "").Trim();

            var model = JsonConvert.DeserializeObject<Root>(json);
            return model;
        }

        protected virtual void Dispose(bool disposing)
        {
            if (!disposedValue)
            {
                if (disposing)
                {
                    client?.Dispose();
                }
                disposedValue = true;
            }
        }

        public void Dispose()
        {
            Dispose(disposing: true);
            GC.SuppressFinalize(this);
        }
    }
}
