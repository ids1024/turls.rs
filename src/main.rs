extern crate fastcgi;
extern crate serde_json;
extern crate rand;
extern crate bidir_map;

use std::iter::FromIterator;
use std::fs::File;
use rand::Rng;

use bidir_map::BidirMap;
use serde_json::{Value, Map};

const B58_ALPHABET: &'static str = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

fn to_base58(mut number: u32) -> String {
    let mut digits = Vec::new();
    while number > 0 {
        digits.push((number % 58) as usize);
        number /= 58;
    }
    String::from_iter(digits.iter().rev().map(|x| B58_ALPHABET.chars().nth(*x).unwrap()))
}

fn from_base58(string: &str) -> u32 {
    string.chars().rev().enumerate().fold(0, |n, (x, i)| {
        n + ((58 as u32).pow(x as u32) * (B58_ALPHABET.find(i).unwrap() as u32))
    })
}

struct UrlMap {
    path: String,
    urls: BidirMap<String, String>,
}

impl UrlMap {
    fn new(path: &str) -> UrlMap {
        let mut urlmap = UrlMap{path: path.to_owned(), urls: BidirMap::new()};
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
        let json = Value::Object(Map::from_iter(self.urls.iter().map(|&(ref k, ref v)| (k.clone(), Value::String(v.clone())))));
        let mut urlsfile = File::create(&self.path).unwrap();
        serde_json::to_writer(&mut urlsfile, &json).unwrap();
    }

    fn add_url(&mut self, url: &str) -> String {
        if let Some(hash) = self.urls.get_by_second(url) {
            return hash.to_owned();
        }
        let mut hash = to_base58(rand::thread_rng().gen_range::<u32>(3364, 113164));
        while self.urls.contains_second_key(url) {
            // Handle randomly occuring duplicate
            hash = to_base58(rand::thread_rng().gen_range::<u32>(3364, 113164));
        }
        self.urls.insert(hash.clone(), url.to_owned());
        self.save_urls();
        hash
    }

    fn get_url(&self, hash: &str) -> Option<String> {
        self.urls.get_by_first(hash).map(|x| x.to_owned())
    }
}

fn main() {
    // let mut input = String::new();
    // std::io::stdin().read_line(&mut input).unwrap();
    // input.pop();
    // let string = to_base58(input.parse::<u32>().unwrap());
    // println!("{}", string)
    // println!("{}", from_base58(&input))

    let mut urlmap = UrlMap::new("urls.json");

    
    //println!("{} : {}", number, to_base58(number))
}
