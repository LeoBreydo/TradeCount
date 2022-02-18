use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub quote_file_path: String,
    pub trade_file_path: String,
    pub quote_file_has_header: bool,
    pub trade_file_has_header: bool,
    pub trade_price_offset: usize,
    pub quote_ask_offset: usize,
    pub quote_bid_offset: usize
}
pub enum Command{
    GetBoth,
    GetQuote,
    GetTrade
}
pub trait Converter<T> {
    fn convert(&self, from:String) -> Option<T>;
}

pub struct QuoteInfo{pub ask_offset:usize,pub bid_offset:usize}
pub struct TradeInfo{pub price_offset:usize}
impl Converter<(usize,f32,f32)> for  QuoteInfo{
    fn convert(&self, record: String) -> Option<(usize,f32,f32)>{
        let v: Vec<&str> = record.split(",").collect();
        if v.len() < self.bid_offset+1{
            println!("Corrupted data detected");
            return None;
        }
        let time = get_millis(&v[0]);
        if time.is_none(){
            println!("Corrupted data detected");
            return None;
        }
        let ask = v[self.ask_offset].parse::<f32>();
        if ask.is_err(){
            println!("Corrupted data detected");
            return None;
        }
        let bid = v[self.bid_offset].parse::<f32>();
        if bid.is_err(){
            println!("Corrupted data detected");
            return None;
        }
        Some((time.unwrap(),ask.unwrap(),bid.unwrap()))
    }
}
impl Converter<(usize,f32)> for TradeInfo{
    fn convert(&self, record: String) -> Option<(usize,f32)>{
        let v: Vec<&str> = record.split(",").collect();
        if v.len() < self.price_offset+1{
            println!("Corrupted data detected");
            return None;
        }
        let time = get_millis(&v[0]);
        if time.is_none(){
            println!("Corrupted data detected");
            return None;
        }
        let price = v[self.price_offset].parse::<f32>();
        if price.is_err(){
            println!("Corrupted data detected");
            return None;
        }
        Some((time.unwrap(), price.unwrap()))
    }
}

pub fn process_price(price:f32, ask:f32, bid: f32) -> usize{
    return if price > bid && price < ask { 1 } else { 0 }
}
const H_FACTOR:usize = 3600000;
const M_FACTOR:usize = 60000;
const S_FACTOR:usize = 1000;
fn get_millis(field:&str)->Option<usize>{
    let v:Vec<&str> = field.split(":").collect();
    if v.len() != 3{
        return None;
    }
    let vv:Vec<&str> = v[2].split(".").collect();
    if vv.len() != 2{
        return None;
    }
    let hr = v[0].parse::<usize>();
    if hr.is_err(){
        return None;
    }
    let h = hr.unwrap()* H_FACTOR;
    let mr = v[1].parse::<usize>();
    if mr.is_err(){
        return None;
    }
    let m = mr.unwrap()* M_FACTOR;
    let sr = vv[0].parse::<usize>();
    if sr.is_err(){
        return None;
    }
    let s = sr.unwrap()* S_FACTOR;
    let msr = vv[1].parse::<usize>();
    if msr.is_err(){
        return None;
    }
    let ms = msr.unwrap();
    Some(h+m+s+ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn millis_test() {
        let s = "01:01:01.900";
        let v:Vec<&str> = s.split(":").collect();
        let vv:Vec<&str> = v[2].split(".").collect();
        let h = v[0].parse::<usize>().unwrap()*3600000;
        let m:usize = v[1].parse::<usize>().unwrap()*60000;
        let s:usize = vv[0].parse::<usize>().unwrap()*1000;
        let ms:usize = vv[1].parse::<usize>().unwrap();
        assert_eq!(3600000+60000+1000+900, h+m+s+ms);
    }
}

