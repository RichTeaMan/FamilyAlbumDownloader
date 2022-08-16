mod family_album_client;
mod model;

use clap::Parser;
use git_version::git_version;
use crate::family_album_client::FamilyAlbumClient;

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
    println!("Thomas Holmes 2022. {GIT_VERSION}");

    let mut client = FamilyAlbumClient::new(&args.id_token, &args.password, &args.output_directory);
    client.login().await.unwrap();

    println!("Downloading album. This may take several minutes...");
    client.download_all_media().await;
}
