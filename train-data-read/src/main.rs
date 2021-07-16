extern crate parquet;

use parquet::file::reader::{FileReader, SerializedFileReader};
use std::fs::File;
use std::path::Path;

fn main() {
    println!("Hello, world!");
    let file = File::open(&Path::new("/data1/lxj/tmp/hdfs-down/part-0-24")).unwrap();
    let reader = SerializedFileReader::new(file).unwrap();
    let mut iter = reader.get_row_iter(None).unwrap();
    while let Some(record) = iter.next() {
        println!("{}", record);
    }
}
