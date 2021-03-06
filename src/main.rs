extern crate fastcgi;
extern crate serde;
extern crate serde_json;
extern crate rand;
extern crate bidir_map;
extern crate toml;

use std::iter::FromIterator;
use std::net::TcpListener;
use std::io::{Read, Write};
use std::fs::File;
use rand::Rng;

use bidir_map::BidirMap;
use serde::ser::Serializer;
use serde_json::Value;

const B58_ALPHABET: &'static str = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

fn to_base58(mut number: u32) -> String {
    let mut digits = Vec::new();
    while number > 0 {
        digits.push((number % 58) as usize);
        number /= 58;
    }
    String::from_iter(digits.iter().rev().map(|x| B58_ALPHABET.chars().nth(*x).unwrap()))
}

// fn from_base58(string: &str) -> u32 {
//    string.chars().rev().enumerate().fold(0, |n, (x, i)| {
//        n + ((58 as u32).pow(x as u32) * (B58_ALPHABET.find(i).unwrap() as u32))
//    })
// }

struct UrlMap {
    path: String,
    urls: BidirMap<String, String>,
}

impl UrlMap {
    fn new(path: &str) -> UrlMap {
        let mut urlmap = UrlMap {
            path: path.to_owned(),
            urls: BidirMap::new(),
        };
        urlmap.load_urls();
        urlmap
    }

    fn load_urls(&mut self) {
        // Format "hash: url"
        let urlsfile = File::open(&self.path).unwrap();
        let json = serde_json::from_reader(urlsfile).unwrap();
        if let Value::Object(urls) = json {
            self.urls = BidirMap::from_iter(urls.iter().map(|(k, v)| {
                if let &Value::String(ref s) = v {
                    (k.clone(), s.clone())
                } else {
                    panic!()
                }
            }))
        } else {
            panic!()
        }
    }

    fn save_urls(&self) {
        let mut urlsfile = File::create(&self.path).unwrap();
        let mut ser = serde_json::Serializer::new(&mut urlsfile);
        let mut state = ser.serialize_map(Some(self.urls.len())).unwrap();
        for &(ref k, ref v) in self.urls.iter() {
            ser.serialize_map_key(&mut state, k).unwrap();
            ser.serialize_map_value(&mut state, v).unwrap();
        }
        ser.serialize_map_end(state).unwrap();
    }

    fn add_url(&mut self, url: &str) -> &str {
        if !self.urls.contains_second_key(url) {
            let mut hash = to_base58(rand::thread_rng().gen_range::<u32>(3364, 113164));
            while self.urls.contains_first_key(&hash) {
                // Handle randomly occuring duplicate
                hash = to_base58(rand::thread_rng().gen_range::<u32>(3364, 113164));
            }
            self.urls.insert(hash, url.to_owned());
            self.save_urls();
        }
        self.urls.get_by_second(url).unwrap()
    }

    fn get_url(&self, hash: &str) -> Option<&str> {
        self.urls.get_by_first(hash).map(|x| x as &str)
    }
}

fn main() {
    let mut urlspath = std::env::home_dir().unwrap();
    urlspath.push(".config/turls/urls.json");
    let mut configpath = std::env::home_dir().unwrap();
    configpath.push(".config/turls/config.toml");

    let mut urlmap = UrlMap::new(urlspath.to_str().unwrap());

    let mut configfile = File::open(&configpath).unwrap();
    let mut toml = String::new();
    configfile.read_to_string(&mut toml).unwrap();
    let config = toml::Parser::new(&toml).parse().unwrap();
    let address = config.get("address").unwrap().as_str().unwrap();
    let baseurl = config.get("baseurl").unwrap().as_str().unwrap();

    let listener = TcpListener::bind(address as &str).unwrap();

    fastcgi::run_tcp(|mut req| {
        let query = req.param("QUERY_STRING").unwrap();
        let uri = req.param("DOCUMENT_URI").unwrap();
        if uri == "/create" {
            let hash = urlmap.add_url(&query);
            write!(&mut req.stdout(),
                   "Content-Type: text/plain\n\n{}{}",
                   baseurl,
                   hash)
                .unwrap();
        } else {
            if let Some(url) = urlmap.get_url(uri.trim_matches('/')) {
                write!(&mut req.stdout(), "Status: 301\nLocation: {}\n\n", url).unwrap();
            } else {
                write!(&mut req.stdout(),
                       "Status: 404\nContent-Type: text/plain\n\n404: Page Not Found")
                    .unwrap();
            }
        }
    },
                     &listener);
}
