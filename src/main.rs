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
    BadUrl,
    BadResponseCode,
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

    try!(handle.url(&url).map_err(|_| Error::BadUrl));
    let _ = handle.fail_on_error(true);
    {
        let mut transfer = handle.transfer();
        transfer.write_function(|new_data| {
                    data.extend_from_slice(new_data);
                    Ok(new_data.len())
                })
                .unwrap();

        if transfer.perform().is_err() {
            return Err(Error::NoInternet);
        }
    }

    let data_string = String::from_utf8(data.clone()).unwrap();

    if try!(handle.response_code().map_err(|_| Error::BadResponseCode)) == 401 {
        return Err(Error::InvalidApiKey);
    }

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
            std::process::exit(1);
        }

        let raw = response.unwrap();

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
