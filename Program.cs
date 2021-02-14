using System;
using System.Reflection;
using System.Threading.Tasks;

namespace FamilyAlbumDownload
{
    public class Program
    {

        public static string VersionNumber => typeof(Program).Assembly
            .GetCustomAttribute<AssemblyInformationalVersionAttribute>()
            .InformationalVersion;

        static async Task Main(string[] args)
        {
            Console.WriteLine("Family Album Downloader");
            Console.WriteLine($"Thomas Holmes 2021. {VersionNumber}");

            using var familyAlbumClient = new FamilyAlbumClient("", "");

            Console.WriteLine();
            Console.WriteLine("Downloading Elliot's album. This may take several minutes...");
            await familyAlbumClient.DownloadAllMedia();
        }
    }
}
