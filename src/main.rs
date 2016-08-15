#[macro_use]
extern crate curl;
extern crate json;
use std::env;
use curl::easy::{Easy, List};

const BASE_URL: &'static str = "https://api.what3words.com/v2";
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
fn reverse_url(api: &str) -> String {
    let mut url = String::new();

    url.push_str(BASE_URL);
    url.push_str("/reverse?coords=51.521251%2C-0.203586&key=");
    url.push_str(&api);
    url.push_str("&lang=en&format=json&display=full");

    url.to_string()
}

#[allow(dead_code)]
fn call_w3w(url: &str) -> String {
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

    let data_string = String::from_utf8(data.clone());

    data_string.unwrap().to_string()
}

fn main() {
    let api_key_result = env::var("W3W_API");
    if api_key_result.is_err() {
        println!("Error! Failed to find API key W3W_API");
        return;
    }

    let api_key = api_key_result.unwrap();
    let url = reverse_url(&api_key);

    println!("url: {}", url);

    // let response = call_w3w(&url);

    let response = json::parse(W3W_RESPONSE).unwrap();

    // println!("{:?}", response);

    println!("words: {}", response["words"]);

    let (lng, lat) = (&response["geometry"]["lng"], &response["geometry"]["lat"]);

    println!("lat: {} lng: {}", lat, lng);

    // println!("{}", foo);
    // let reverse = json::parse(&foo);

    // println!("{:?}", reverse.unwrap());


}
