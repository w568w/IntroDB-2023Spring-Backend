pub trait OptionExt<T>
where
    Self: Sized,
{
    fn apply_if_some<Applied, F: FnOnce(Applied, T) -> Applied>(
        self,
        applied: Applied,
        f: F,
    ) -> Applied;
}

impl<T> OptionExt<T> for Option<T> {
    fn apply_if_some<Applied, F: FnOnce(Applied, T) -> Applied>(
        self,
        applied: Applied,
        f: F,
    ) -> Applied {
        match self {
            Some(option) => f(applied, option),
            None => applied,
        }
    }
}
