use num_integer::Integer;

pub struct RangeQuery<T: Integer> {
    inner: Vec<T>,
}
