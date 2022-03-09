use anyhow::{Context, Result};
use askama::Template;
use clap::{Arg, Command};
use serde::Deserialize;
use serde_json;
use wkhtmltopdf::{pdf::PageSize, Orientation, PdfApplication, Size};

#[derive(Deserialize, Template)]
#[template(path = "postcards.html")]
struct Postcards {
    sender_address: Address,
    addresses: Vec<Address>,
    width: u32,
    height: u32,
    output: String,
}

#[derive(Clone, Deserialize)]
struct Address {
    name: Option<String>,
    address_1: String,
    address_2: Option<String>,
    city: String,
    state: Option<String>,
    post_code: String,
    country: String,
}

fn gen_postcards() -> Result<Postcards> {
    let m = Command::new("mailmerge")
        .about("Create envelope pdfs from a vector of JSON addresses")
        .args(&[
            Arg::new("input")
                .help("Input with JSON array of addresses")
                .index(1)
                .required(true),
            Arg::new("sender")
                .help("JSON string corresponding to the address of the sender")
                .short('s')
                .long("sender")
                .takes_value(true)
                .required(true),
            Arg::new("output")
                .help("Path to which the pdf will be saved")
                .short('o')
                .takes_value(true)
                .default_value("addresses.pdf"),
            Arg::new("height")
                .help("Height of envelopes, in millimeters")
                .short('h')
                .takes_value(true)
                .default_value("114"),
            Arg::new("width")
                .help("Width of envelopes, in millimeters")
                .short('w')
                .takes_value(true)
                .default_value("162"),
        ])
        .get_matches();
    let input = m.value_of("input").expect("Required");
    let width = m
        .value_of("width")
        .expect("Always present")
        .parse()
        .context("Failed to parse width as integer")?;
    let sender_address: Address =
        serde_json::from_str(m.value_of("sender").expect("Always present"))
            .context("Failed to parse sender address")?;
    let height = m
        .value_of("height")
        .expect("Always present")
        .parse()
        .context("Failed to parse width as integer")?;
    let output = m.value_of("output").expect("Always present").to_string();
    let addresses = serde_json::from_str(input)?;

    Ok(Postcards {
        sender_address,
        addresses,
        width,
        height,
        output,
    })
}

fn main() -> Result<()> {
    let postcards = gen_postcards()?;
    let html = postcards.render()?;
    let mut pdf = PdfApplication::new()?
        .builder()
        .orientation(Orientation::Landscape)
        .page_size(PageSize::Custom(
            Size::Millimeters(postcards.height),
            Size::Millimeters(postcards.width),
        ))
        .build_from_html(html)?;
    pdf.save(postcards.output)?;
    Ok(())
}
