use std::{
    fs::File,
    io::{BufRead, BufReader, Lines, Result}
};
use crate::converters::Converter;

pub struct TwoFilesIterator<T, U> {
    quote_iter: Lines<BufReader<File>>,
    trade_iter: Lines<BufReader<File>>,
    to_trade_converter: Box<dyn Converter<T>>,
    to_quote_converter: Box<dyn Converter<U>>
}
impl<T,U> TwoFilesIterator<T,U> {
    pub fn new(quote_file_path:String, trade_file_path:String,
               to_trade_converter: impl Converter<T> + 'static,
               to_quote_converter: impl Converter<U> + 'static) -> Option<TwoFilesIterator<T,U>> {
        Some(Self{ quote_iter: BufReader::new(File::open(quote_file_path).ok()?).lines(),
            trade_iter: BufReader::new(File::open(trade_file_path).ok()?).lines(),
            to_trade_converter:Box::new(to_trade_converter),
            to_quote_converter:Box::new(to_quote_converter)
        })
    }
    pub fn next_quote_row(&mut self) -> Option<Result<String>> {
        self.quote_iter.next()
    }
    pub fn next_trade_row(&mut self) -> Option<Result<String>> { self.trade_iter.next() }
    pub fn next_quote_record(&mut self) -> Option<U>{
        let q = self.quote_iter.next();
        return if q.is_none() { None } else { self.to_quote_converter.convert(q.unwrap()) }
    }
    pub fn next_trade_record(&mut self) -> Option<T>{
        let q = self.trade_iter.next();
        return if q.is_none() { None } else { self.to_trade_converter.convert(q.unwrap()) }
    }
}