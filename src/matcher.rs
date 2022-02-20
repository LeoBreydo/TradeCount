pub trait Matcher<T,U>{
    fn is_matched(&self, first:&T, second:&U) -> bool;
}

#[derive(Default, Clone, Copy)]
pub struct QuoteToTradeMatcher;

// (ask,bid) to (_,price)
impl Matcher<(f32,f32),(usize,f32)> for QuoteToTradeMatcher {
    fn is_matched(&self, first: &(f32, f32), second: &(usize, f32)) -> bool {
        second.1 < first.0 && second.1 > first.1
    }
}

