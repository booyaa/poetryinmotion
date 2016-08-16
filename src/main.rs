#[macro_use]
extern crate curl;
extern crate json;
extern crate csv;

use std::env;
use curl::easy::Easy;

#[allow(dead_code)]
const W3W_RESPONSE: &'static str = r#"
{
    "crs": {
        "type": "link",
        "properties": {
            "href": "http://spatialreference.org/ref/epsg/4326/ogcwkt/",
            "type": "ogcwkt"
        }
    },
    "words": "index.home.raft",
    "bounds": {
        "southwest": {
            "lng": -0.203607,
            "lat": 51.521238
        },
        "northeast": {
            "lng": -0.203564,
            "lat": 51.521265
        }
    },
    "geometry": {
        "lng": -0.203586,
        "lat": 51.521251
    },
    "language": "en",
    "map": "http://w3w.co/index.home.raft",
    "status": {
        "code": 200,
        "message": "OK"
    },
    "thanks": "Thanks from all of us at index.home.raft for using a what3words API"
}
"#;

#[allow(dead_code)]
const CSV_TEST_DATA: &'static str = "
work, 51.521251, -0.203586
travelling, 51.5412621, \
                                     -0.08813879999999999
";

const BASE_URL: &'static str = "https://api.what3words.com/v2";

#[derive(Debug)]
pub enum Error {
    InvalidApiKey,
}

fn reverse_url(api: &str, lat: &f64, lng: &f64) -> String {
    let mut url = String::new();

    url.push_str(BASE_URL);
    url.push_str(&format!("/reverse?coords={}%2C{}", &lat, &lng));
    url.push_str(&format!("&key={}", &api));
    url.push_str("&lang=en&format=json&display=full");

    url.to_string()
}

#[allow(dead_code)]
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
        transfer.perform().unwrap();
    }

    println!("{:?}", handle.response_code());
    let data_string = String::from_utf8(data.clone());

    if handle.response_code().unwrap() == 401 {
        return Err(Error::InvalidApiKey);
    }

    Ok(data_string.unwrap().to_string())
}

fn main() {
    let api_key_result = env::var("W3W_API");
    if api_key_result.is_err() {
        println!("Error! Failed to find API key W3W_API");
        return;
    }

    let api_key = api_key_result.unwrap();

    let mut coords = csv::Reader::from_string(CSV_TEST_DATA).has_headers(false);

    for row in coords.decode() {
        let (place, lat, lng): (String, f64, f64) = row.unwrap();

        let url = reverse_url(&api_key, &lat, &lng);

        let response = call_w3w(&url);

        if response.is_err() {
            println!("Error: invalid API key!");
            std::process::exit(1);
        }

        let parsed = json::parse(&response.unwrap()).unwrap();
        println!("{} - {}",
                 place,
                 parsed["words"].to_string().replace(".", " "));
    }
}
