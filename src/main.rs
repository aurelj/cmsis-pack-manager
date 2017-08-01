extern crate minidom;
extern crate quick_xml;
extern crate futures;
extern crate tokio_core;
extern crate hyper;

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use minidom::Element;
use minidom::Children;
use minidom::error::Error;
use quick_xml::reader::Reader;

use std::io::{self, Write};
use futures::{Future, Stream};
use hyper::Client;
use tokio_core::reactor::Core;

#[derive(Debug)]
struct Pdsc{
    url: String,
    vendor: String,
    name: String,
    version: String,
    date: Option<String>,
    deprecated: Option<String>,
    replacement: Option<String>,
    size: Option<String>,
}

impl Pdsc {
    fn vec_from_children(children: Children) -> Option<Vec<Self>>{
        let mut to_ret = Vec::new();
        for e in children {
            to_ret.push(Self{
                url: e.attr("url").map(String::from).unwrap(),
                vendor: e.attr("vendor").map(String::from).unwrap(),
                name: e.attr("name").map(String::from).unwrap(),
                version: e.attr("version").map(String::from).unwrap(),
                date: e.attr("date").map(String::from),
                deprecated: e.attr("deprecated").map(String::from),
                replacement: e.attr("replacement").map(String::from),
                size: e.attr("size").map(String::from),
            });
        }
        Some(to_ret)
    }
}


#[derive(Debug)]
struct Pidx{
    url: String,
    vendor: String,
    date: Option<String>,
}

impl Pidx {
    fn vec_from_children(children: Children) -> Option<Vec<Self>>{
        let mut to_ret = Vec::new();
        for e in children {
            to_ret.push(Self{
                url: e.attr("url").map(String::from).unwrap(),
                vendor: e.attr("vendor").map(String::from).unwrap(),
                date: e.attr("date").map(String::from),
            });
        }
        Some(to_ret)
    }
}

#[derive(Debug)]
struct Vidx {
    vendor: String,
    url: String,
    timestamp: Option<String>,
    pdscIndex: Option<Vec<Pdsc>>,
    vendorIndex: Option<Vec<Pidx>>,
}

fn parse(path: &Path) -> Result<Vidx, Error> {
    let mut reader = Reader::from_file(path)?;
    let root = Element::from_reader(&mut reader)?;
    Ok(Vidx {
        vendor:  root.get_child("vendor", "http://www.w3.org/2001/XMLSchema-instance").unwrap().text(),
        url:  root.get_child("url", "http://www.w3.org/2001/XMLSchema-instance").unwrap().text(),
        timestamp:  root.get_child("timestamp", "http://www.w3.org/2001/XMLSchema-instance").map(Element::text),
        vendorIndex: root.get_child("vindex", "http://www.w3.org/2001/XMLSchema-instance").map(Element::children).and_then(Pidx::vec_from_children),
        pdscIndex: root.get_child("pindex", "http://www.w3.org/2001/XMLSchema-instance").map(Element::children).and_then(Pdsc::vec_from_children),
    })
}

static PIDX_SUFFIX: &'static str = ".pidx";

fn get_vidx(vidx: Vidx) -> Result<(), Error> {
    let mut core = Core::new().unwrap();
    let client = Client::new(&core.handle());
    if let Some(vidxs) = vidx.vendorIndex {
        for Pidx{url, vendor, ..} in vidxs {
            let mut urlname = String::with_capacity(url.len() + vendor.len() + PIDX_SUFFIX.len());
            urlname += &url;
            urlname += &vendor;
            urlname += &PIDX_SUFFIX;
            println!("{}", urlname);
            match urlname.parse() {
                Ok(uri) => {
                    let work = client.get(uri).map(|res|{
                        println!("Response: {}", res.status());
                        res.body().for_each(move |body| {
                            println!("{:?}", body);
                            Ok(())
                        })
                    });
                    core.run(work).unwrap();
                }
                Err(e) => {
                    println!("{}", e)
                }
            }
        }
    }
    Ok(())
}


fn main() {
    parse(Path::new("keil.vidx")).and_then(get_vidx);
}
