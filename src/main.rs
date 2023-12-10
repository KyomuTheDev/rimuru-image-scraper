use clap::Parser;
use reqwest::blocking::ClientBuilder;
use select::document::Document;
use select::predicate::Name;
use std::path::PathBuf;
use std::{fs, io};

mod cli;

const IMAGE_FOLDER: &str = "C:\\Lord Rimuru";

fn main() {
    let cmds = cli::Args::parse();

    match cmds.command {
        cli::Commands::Get { url } => download(url),
        cli::Commands::Init {} => init(),
    };
}

fn init() -> () {
    let paths: Vec<_> = fs::read_dir(IMAGE_FOLDER).unwrap().collect();
    let image_folder = PathBuf::from(IMAGE_FOLDER);

    let mut index = 0;

    for path in paths.into_iter() {
        let path = path.unwrap().path();

        println!("Processing entry: {:?}", path.file_name().unwrap());

        if !path.is_file() {
            continue;
        }

        index += 1;

        let ext = path.extension().unwrap().to_str().unwrap();
        let new_name = format!("{}.{}", index, ext);
        let new_path = image_folder.clone().join(&new_name);

        println!("Renaming file: {:?} to {:?}", path, new_path);

        match fs::rename(path.clone(), new_path.clone()) {
            Ok(_) => println!(
                "Successfully renamed file to {:?}",
                new_path.file_name().unwrap()
            ),
            Err(e) => println!("Failed to rename file: {:?}", e),
        };
    }
}

fn download(url: String) -> () {
    let client = ClientBuilder::new()
        .user_agent(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/117.0",
        )
        .build()
        .expect("Failed to build client");

    let request = client.get(url).build().expect("Failed to build request");

    let response = client
        .execute(request)
        .expect("Failed to execute request")
        .text()
        .expect("Ok seriously if I have to write \".expect\" one more fucking time");

    let html = Document::from(response.as_str());

    let mut index = fs::read_dir(IMAGE_FOLDER).unwrap().count();

    let mut occurances: Vec<&str> = vec![];

    for x in html.find(Name("img")).filter_map(|n| n.attr("src")) {
        if occurances.contains(&x) {
            return;
        }

        index += 1;
        occurances.push(x.clone());
    }

    for x in occurances.into_iter() {
        let client = client.clone();

        let img_response = client.get(x.clone()).send().expect("Failed to fetch image");

        let img_bytes = img_response
            .bytes()
            .expect("Failed to convert image response into bytes");

        let file_name;

        if x.ends_with(".png") {
            file_name = format!("{}\\{}.png", IMAGE_FOLDER, index);
        } else if x.ends_with(".jpg") {
            file_name = format!("{}\\{}.jpg", IMAGE_FOLDER, index)
        } else {
            file_name = format!("{}\\{}.webp", IMAGE_FOLDER, index)
        }

        let mut out = std::fs::File::create(&file_name).expect("Failed to create file");

        io::copy(&mut &img_bytes[..], &mut out).expect("Failed to write image to file");
    }
}