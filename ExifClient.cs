using System;
using System.Diagnostics;
using System.IO;
using System.IO.Compression;
using System.Linq;
using System.Net;
using System.Runtime.InteropServices;
using System.Threading.Tasks;

namespace FamilyAlbumDownload
{
    public class ExifClient
    {
        private const string ROOT_URL = "https://exiftool.org/exiftool-12.21.zip";

        private readonly static string exeDirectory = AppContext.BaseDirectory;
        private readonly static string programName = "exiftool.exe";
        private readonly static string programPath = Path.Combine(exeDirectory, programName);

        private async Task DownloadExifTool()
        {
            if (!RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
            {
                throw new Exception("Unsupported OS.");
            }
            string exifZipLocation = Path.Combine(exeDirectory, "exiftool-12.21.zip");
            string exifUnzipLocation = Path.Combine(exeDirectory, "exif-decom");

            if (File.Exists(programPath))
            {
                return;
            }

            try
            {
                using var webclient = new WebClient();
                await webclient.DownloadFileTaskAsync(ROOT_URL, exifZipLocation);

                ZipFile.ExtractToDirectory(exifZipLocation, exifUnzipLocation);

                string exifK = "exiftool(-k).exe";
                var programPathFromArchive = Directory.EnumerateFiles(exifUnzipLocation, exifK, SearchOption.AllDirectories).FirstOrDefault();
                if (programPathFromArchive == null)
                {
                    throw new Exception("Could not find exif in zip archive.");
                }
                File.Move(programPathFromArchive, programPath);

                Console.WriteLine($"exif saved to {programPath}.");
            }
            finally
            {
                try
                {
                    Directory.Delete(exifUnzipLocation, true);
                }
                catch (IOException ex)
                {
                    Console.WriteLine(ex);
                }
                try
                {
                    File.Delete(exifZipLocation);
                }
                catch (IOException ex)
                {
                    Console.WriteLine(ex);
                }
            }
        }

        private async Task<ProcessStartInfo> FetchProcess()
        {
            await DownloadExifTool();

            var startInfo = new ProcessStartInfo()
            {
                FileName = programPath,
                UseShellExecute = false,
                WorkingDirectory = Directory.GetCurrentDirectory(),
                RedirectStandardOutput = true,
                CreateNoWindow = true
            };
            return startInfo;
        }

        public async Task AddExif(string mediaPath, DateTimeOffset? takenDateTime = null, double? gpsLatitude = null, double? gpsLongtitude = null, string model = null)
        {
            string convertedPath = mediaPath.Replace(@"\", "/");

            var fi = new FileInfo(convertedPath);
            if (!fi.Exists)
            {
                Console.WriteLine($"File '{convertedPath}' does not exist.");
                return;
            }

            string dateTimeOriginalFormat = "yyyy:MM:dd HH:mm:ss";

            string args = $"{convertedPath} -overwrite_original";
            if (takenDateTime.HasValue)
            {
                args += $" \"-DateTimeOriginal={takenDateTime.Value.ToString(dateTimeOriginalFormat)}\"";
                args += $" \"-CreateDate={takenDateTime.Value.ToString(dateTimeOriginalFormat)}\"";
            }
            if (gpsLatitude.HasValue && gpsLongtitude.HasValue)
            {
                args += $" -gpslatitude={gpsLatitude} -gpslongitude={gpsLongtitude}";
            }
            if (!string.IsNullOrWhiteSpace(model))
            {
                args += $" -model={model}";
            }

            var processInfo = await FetchProcess();
            processInfo.Arguments = args;

            using var process = new Process
            {
                StartInfo = processInfo
            };

            process.Start();
            process.WaitForExit();
        }
    }
}
