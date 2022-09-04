mod family_album_client;
mod model;

use crate::family_album_client::FamilyAlbumClient;
use clap::Parser;
use git_version::git_version;

pub const GIT_VERSION: &str = git_version!();

/// Family Album Downloader
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    id_token: String,

    #[clap(short, long)]
    password: String,

    #[clap(short, long)]
    output_directory: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    println!("Family Album Downloader");
    println!("Thomas Holmes 2022. Version {GIT_VERSION}");

    let mut client = FamilyAlbumClient::new(&args.id_token, &args.password, &args.output_directory);

    println!("Downloading album. This may take several minutes...");
    loop {
        client.login().await.unwrap();
        if let Err(_) = client.download_all_media().await {
            println!("Credentials have timed out. Refreshing media list.");
        } else {
            break;
        }
    }
    println!("Complete.");
}
