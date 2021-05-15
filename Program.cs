using System;
using System.CommandLine;
using System.CommandLine.Invocation;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;

namespace FamilyAlbumDownload
{
    public class Program
    {

        public static string VersionNumber => typeof(Program).Assembly
            .GetCustomAttribute<AssemblyInformationalVersionAttribute>()
            .InformationalVersion;

        static async Task<int> Main(string[] args)
        {
            Console.WriteLine("Family Album Downloader");
            Console.WriteLine($"Thomas Holmes 2021. {VersionNumber}");
            Console.WriteLine();

            var rootCommand = new RootCommand
            {
                new Option<string>(
                    "--id-token",
                    "Token identifying the user and the album. This is the final part of the mitene URL.",
                    ArgumentArity.ExactlyOne),
                new Option<string>(
                    "--password",
                    "The user's password.",
                    ArgumentArity.ExactlyOne),
                new Option<string>(
                    "--output-directory",
                    "Directory to put album.",
                    ArgumentArity.ExactlyOne)
            };

            rootCommand.Description = "Downloads all photos from the given album of the ID token.";
            var options = new []{ "--id-token", "--password", "--output-directory" };
            foreach (var option in options)
            {
                rootCommand.AddValidator(symbolResult =>
                {
                    if (symbolResult.Children.GetByAlias(option) is null)
                    {
                        return $"Option {option} is required";
                    }
                    else
                    {
                        return null;
                    }
                });
            }

            // Note that the parameters of the handler method are matched according to the names of the options
            rootCommand.Handler = CommandHandler.Create<string, string, string>(async (idToken, password, outputDirectory) =>
            {
                using var familyAlbumClient = new FamilyAlbumClient(idToken, password, outputDirectory);

                Console.WriteLine();
                Console.WriteLine("Downloading album. This may take several minutes...");
                await familyAlbumClient.DownloadAllMedia();
            });

            // Parse the incoming args and invoke the handler
            return await rootCommand.InvokeAsync(args);
        }
    }
}
