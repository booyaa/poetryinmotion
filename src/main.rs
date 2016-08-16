#[macro_use]
extern crate curl;
extern crate json;
extern crate csv;

use std::env;
use curl::easy::Easy;

const BASE_URL: &'static str = "https://api.what3words.com/v2";

#[derive(Debug,PartialEq)]
pub enum Error {
    InvalidApiKey,
    NoInternet,
}

fn reverse_url(api: &str, lat: &f64, lng: &f64) -> String {
    let mut url = String::new();

    url.push_str(BASE_URL);
    url.push_str(&format!("/reverse?coords={}%2C{}", &lat, &lng));
    url.push_str(&format!("&key={}", &api));
    url.push_str("&lang=en&format=json&display=full");

    url.to_string()
}

fn call_w3w(url: &str) -> Result<String, Error> {
    let mut handle = Easy::new();
    let mut data = Vec::new();

    handle.url(&url.to_string()).unwrap();

    {
        let mut transfer = handle.transfer();
        transfer.write_function(|new_data| {
                    data.extend_from_slice(new_data);
                    Ok(new_data.len())
                })
                .unwrap();

        // either of these two lines will cause data_string to combine all responses from the
        // server rather than return them as individual responses:

        // if you don't call is_err():
        // [{"crs":{"type":"link","properties":{"href":"http:\/\/spatialreference.org\/ref\/epsg\/4326\/ogcwkt\/","type":"ogcwkt"}},"words":"index.home.raft*snip*]
        // [{"crs":{"type":"link","properties":{"href":"http:\/\/spatialreference.org\/ref\/epsg\/4326\/ogcwkt\/","type":"ogcwkt"}},"words":"copper.tent.fled*snip*]

        // if you do
        // [{"crs":{"type":"link","properties":{"href":"http:\/\/spatialreference.org\/ref\/epsg\/4326\/ogcwkt\/","type":"ogcwkt"}},"words":"index.home.raft*snip*
        //  {"crs":{"type":"link","properties":{"href":"http:\/\/spatialreference.org\/ref\/epsg\/4326\/ogcwkt\/","type":"ogcwkt"}},"words":"copper.tent.fled*snip*]


        // either of these will trigger the side effect:
        // println!("transfer.perform is_err: {}", transfer.perform().is_err());

        // if transfer.perform().is_err() {
        //     return Err(Error::NoInternet);
        // }

        transfer.perform().unwrap();
    }

    let data_string = String::from_utf8(data.clone()).unwrap();

    if handle.response_code().unwrap() == 401 {
        return Err(Error::InvalidApiKey);
    }

    println!("call_w3w: [{}]", data_string.to_string());
    Ok(data_string.to_string())
}

fn main() {
    let api_key_result = env::var("W3W_API");
    if api_key_result.is_err() {
        println!("Error! Failed to find API key W3W_API");
        return;
    }

    let api_key = api_key_result.unwrap();

    let mut args = env::args();
    let _program_name = args.next().unwrap();
    let csv = args.next().expect("Missing csv file!");

    let mut coords = csv::Reader::from_file(csv).unwrap().has_headers(false);

    for row in coords.decode() {
        let (place, lat, lng): (String, f64, f64) = row.unwrap();

        let url = reverse_url(&api_key, &lat, &lng);

        let response = call_w3w(&url);

        if response.is_err() {
            let err = response.err().unwrap();
            if err == Error::InvalidApiKey {
                println!("Error: invalid API key!");
            } else {
                println!("Error: Are you connected to the internet?");
            }
            // match response.err() {
            //     Some(Error::InvalidApiKey) => println!("Error: invalid API key!"),
            //     Some(Error::NoInternet) => println!("Error: Are you connected to the internet?"),
            //     _ => (),
            // }
            std::process::exit(1);
        }

        let raw = response.unwrap();

        // println!("{}", raw);

        let parsed = json::parse(&raw);

        if parsed.is_err() {
            println!("Error: failed to parse json!\n\t{}", raw);
            return;
        }

        println!("{} - {}",
                 place,
                 parsed.unwrap()["words"].to_string().replace(".", " "));
    }
}
