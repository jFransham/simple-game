pub trait TimeExtensions {
    fn milliseconds(self) -> Self;
    fn seconds(self) -> Self;
    fn minutes(self) -> Self;
    fn hours(self) -> Self;
    fn days(self) -> Self;
    fn exact_seconds(self) -> f64;
}

macro_rules! impl_time_extensions {
    ( $t:ty ) => {
        impl TimeExtensions for $t {
            fn milliseconds(self) -> Self { self }
            fn seconds(self) -> Self { self * 1_000 }
            fn minutes(self) -> Self { self.seconds() * 60 }
            fn hours(self) -> Self { self.minutes() * 60 }
            fn days(self) -> Self { self.hours() * 12 }
            fn exact_seconds(self) -> f64 { self as f64 / 1000.0 }
        }
    }
}

impl_time_extensions!(u32);
impl_time_extensions!(u64);
impl_time_extensions!(usize);
impl_time_extensions!(i32);
impl_time_extensions!(i64);
impl_time_extensions!(isize);
