use std::fs::File;
use std::io::{BufRead, BufReader, Lines};
use crate::data::{Converter};

pub struct TwoFilesIterator<'a, T, U> {
    quote_iter : Lines<BufReader<File>>,
    trade_iter : Lines<BufReader<File>>,
    to_trade_converter: &'a dyn Converter<T>,
    to_quote_converter: &'a dyn Converter<U>,
}
impl<'a, T,U> TwoFilesIterator<'a, T,U> {
    pub fn new(quote_file_path:String, trade_file_path:String,
               to_trade_converter: &'a dyn Converter<T>,
               to_quote_converter: &'a dyn Converter<U>) -> Option<TwoFilesIterator<'a, T,U>> {
        Some(Self{ quote_iter: BufReader::new(File::open(quote_file_path).ok()?).lines(),
            trade_iter: BufReader::new(File::open(trade_file_path).ok()?).lines(),
            to_trade_converter,
            to_quote_converter
        })
    }
    pub fn next_quote_row(&mut self) -> Option<std::io::Result<String>> {
        self.quote_iter.next()
    }
    pub fn next_trade_row(&mut self) -> Option<std::io::Result<String>> { self.trade_iter.next() }
    pub fn next_quote_record(&mut self) -> Option<U>{
        let q = self.quote_iter.next();
        return if q.is_none() { None } else {
            self.to_quote_converter.convert(q.unwrap().unwrap())
        }
    }
    pub fn next_trade_record(&mut self) -> Option<T>{
        let q = self.trade_iter.next();
        return if q.is_none() { None } else {
            self.to_trade_converter.convert(q.unwrap().unwrap())
        }
    }
}