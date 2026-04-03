pub trait RtcSource {
    fn second(&self) -> u8;
    fn minute(&self) -> u8;
    fn hour(&self) -> u8;
    fn ordinal0(&self) -> u16;
    fn snapshot(&mut self);
}

pub struct MockRtc {

}

impl RtcSource for MockRtc {
    fn second(&self) -> u8 {0}
    fn minute(&self) -> u8 {0}
    fn hour(&self) -> u8 {0}
    fn ordinal0(&self) -> u16 {0}
    fn snapshot(&mut self) {}
}
