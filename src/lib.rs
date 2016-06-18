use std::fmt;
use std::fmt::Debug;

#[derive(Clone)]
pub struct BackoffSequence<'a, F: 'a, B> {
    max_iterations: Option<u64>,
    min_value: Option<B>,
    max_value: Option<B>,
    calculator: &'a F,
}

impl<'a, F, B> Debug for BackoffSequence<'a, F, B>
    where B: Debug
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "mi: {:?}, mv: {:?}", self.max_iterations, self.max_value)
    }
}

impl<'a, F, B> BackoffSequence<'a, F, B>
    where F: Fn(u64) -> B,
          B: PartialOrd + Clone + Debug
{
    pub fn new(f: &'a F) -> Self {
        BackoffSequence {
            calculator: f,
            max_iterations: None,
            min_value: None,
            max_value: None,
        }
    }

    pub fn max_iterations(&mut self, x: u64) -> &mut Self {
        self.max_iterations = Some(x);
        self
    }

    pub fn min(&mut self, x: B) -> &mut Self {
        self.min_value = Some(x);
        self
    }

    pub fn max(&mut self, x: B) -> &mut Self {
        self.max_value = Some(x);
        self
    }

    pub fn iter(&self) -> BackoffSequenceIterator<F, B> {
        BackoffSequenceIterator {
            iteration: 0,
            max_iterations: self.max_iterations,
            calculator: self.calculator,
            current_value: None,
            max_value: self.max_value.clone(),
            min_value: self.min_value.clone(),
        }
    }
}

// Don't impl this one, it moves the BackoffSequence
// impl<'a, F, B> IntoIterator for BackoffSequence<'a, F: Fn(u64) -> B, B>

impl<'a, F, B> IntoIterator for &'a BackoffSequence<'a, F, B>
    where F: Fn(u64) -> B,
          B: PartialOrd + Clone + Debug
{
    type Item = B;
    type IntoIter = BackoffSequenceIterator<'a, F, B>;
    // TODO make this able to return any of a set of iterators in this module, so that I can go for
    // a basic unbounded iterator with very little state or logic, and then adapt it with functions
    // to do things like clamp the value or limit iterations or whatever

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct BackoffSequenceIterator<'a, F: 'a, B> {
    calculator: &'a F,

    iteration: u64,
    max_iterations: Option<u64>,
    current_value: Option<B>,
    min_value: Option<B>,
    max_value: Option<B>,
}

impl<'a, F, B> Debug for BackoffSequenceIterator<'a, F, B>
    where B: Debug
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "i: {:?}, mi: {:?}, cur: {:?}, min: {:?}, max: {:?}",
               self.iteration,
               self.max_iterations,
               self.current_value,
               self.min_value,
               self.max_value)
    }
}

impl<'a, F, B> Iterator for BackoffSequenceIterator<'a, F, B>
    where F: Fn(u64) -> B,
          B: PartialOrd + Clone + Debug
{
    type Item = B;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(mi) = self.max_iterations {
            if self.iteration >= mi {
                return None;
            }
        }

        self.iteration += 1;

        // check max value prior to calculations, to avoid integer overflow
        match (&self.current_value, &self.max_value) {
            (&Some(ref cur), &Some(ref max)) if *cur >= *max => return self.max_value.clone(),
            _ => (),
        }

        let mut new_value = Some((self.calculator)(self.iteration));

        // if the value is less than the minimum, advance the iterator until the value is >= the
        // minimum, and increase the max iterations (if required) by the corresponding #
        if self.min_value.is_some() {
            let min = self.min_value.clone().unwrap();
            let mut cur = new_value.clone().unwrap();
            let mut iter = self.iteration;

            while cur < min {
                iter += 1;
                cur = (self.calculator)(iter);
            }

            if let Some(mi) = self.max_iterations {
                self.max_iterations = Some(mi + (iter - self.iteration));
            }

            self.iteration = iter;
            new_value = Some(cur);

            self.min_value = None;
        }

        self.current_value = match (&new_value, &self.max_value) {
            //            (None, _) => return None,
            //            (_, None) => self.current_value,
            // (&Some(ref c), &Some(ref m)) if *c <= *m => new_value.clone(),
            (&Some(ref new), &Some(ref max)) if *new > *max => self.max_value.clone(),
            _ => new_value.clone(),
        };

        self.current_value.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    fn double(x: u64) -> u64 {
        x * 2
    }

    fn base_2_exp_calculator(x: u64) -> u64 {
        2u64.pow(x as u32) - 1
    }

    #[test]
    fn new_builder_with_function_ptr() {
        let f = &base_2_exp_calculator;
        let mut _x = BackoffSequence::new(f);
    }

    #[test]
    fn new_builder_with_closure_ptr() {
        let f = &|x| 2u64.pow(x as u32);
        let mut _x = BackoffSequence::new(&f);
    }

    #[test]
    fn new_builder_one_opt() {
        let f = &base_2_exp_calculator;
        let mut x = BackoffSequence::new(f);
        x.min(5);
    }

    #[test]
    fn new_builder_all_opt() {
        let f = &base_2_exp_calculator;
        let mut x = BackoffSequence::new(f);
        x.min(5).max(1500).max_iterations(500);
    }

    #[test]
    fn new_builder_duration() {
        use std::time::Duration;
        let f = &|x| Duration::new(0, 2u32.pow(x as u32));

        let mut x = BackoffSequence::new(f);
        x.min(Duration::new(0, 5)).max(Duration::new(0, 1500)).max_iterations(500);
    }

    #[test]
    fn max_iters_0() {
        let f = &|_| 0;
        let mut x = BackoffSequence::new(f);
        x.max_iterations(0);
        assert_eq!(0, x.into_iter().collect::<Vec<_>>().len());
    }

    #[test]
    fn max_iters_1() {
        let f = &|_| 0;
        let mut x = BackoffSequence::new(f);
        x.max_iterations(1);
        assert_eq!(1, x.into_iter().collect::<Vec<_>>().len());
    }

    #[test]
    fn max_iters_1000() {
        let f = &|_| 0;
        let mut x = BackoffSequence::new(f);
        x.max_iterations(1000);
        assert_eq!(1000, x.into_iter().collect::<Vec<_>>().len());
    }

    #[test]
    #[should_panic(expected = "arithmetic operation overflowed")]
    fn unbounded_for_loop() {
        let f = &base_2_exp_calculator;
        for _ in &BackoffSequence::new(f) {
        }
    }

    #[test]
    fn exponential() {
        let f = &base_2_exp_calculator;
        let v = BackoffSequence::new(f).max_iterations(15).into_iter().collect::<Vec<_>>();
        assert_eq!(v,
                   vec![1, 3, 7, 15, 31, 63, 127, 255, 511, 1023, 2047, 4095, 8191, 16383, 32767]);
    }

    #[test]
    fn clamped() {
        let f = &|x| 10u64.pow(x as u32) - 1;
        let v = BackoffSequence::new(f)
            .max_iterations(4)
            .max(150)
            .into_iter()
            .collect::<Vec<_>>();
        assert_eq!(v, vec![9, 99, 150, 150]);
    }

    #[test]
    fn min_value() {
        let f = &|x| 10u64.pow(x as u32) - 1;
        let v = BackoffSequence::new(f)
            .max_iterations(4)
            .min(10)
            .into_iter()
            .collect::<Vec<_>>();
        assert_eq!(v, vec![99, 999, 9999, 99999]);
    }

    #[test]
    fn min_and_max_value() {
        let f = &|x| 10u64.pow(x as u32) - 1;
        let v = BackoffSequence::new(f)
            .max_iterations(4)
            .min(10)
            .max(10000)
            .into_iter()
            .collect::<Vec<_>>();
        assert_eq!(v, vec![99, 999, 9999, 10000]);
    }

    #[test]
    fn min_greater_than_max() {
        let f = &|x| 10u64.pow(x as u32) - 1;
        let v = BackoffSequence::new(f)
            .max_iterations(4)
            .min(10000)
            .max(100)
            .into_iter()
            .collect::<Vec<_>>();
        assert_eq!(v, vec![100, 100, 100, 100]);
    }
}
