// Should really have a struct that configures the things which can be stored, and then implement
// IntoIter for it, which would return what I've got implemented here

//pub struct Backoff

// -------------------- and now the iter bits

pub trait BackoffCalculator {
    fn value(&mut self, iteration: u64) -> Option<u64>;
}

pub struct BackoffSequenceIter<C>
    where C: BackoffCalculator
{
    calculator: C,
    
    iteration: u64,
    max_iterations: Option<u64>,
}

impl<C> BackoffSequenceIter<C>
    where C: BackoffCalculator
{
    pub fn new(calculator: C, max_iterations: Option<u64>) -> Self {
        BackoffSequenceIter {
            calculator: calculator,
            iteration: 0,
            max_iterations: max_iterations,
        }
    }
}

impl<C> Iterator for BackoffSequenceIter<C>
    where C: BackoffCalculator
{
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        if let Some(mi) = self.max_iterations {
            if self.iteration >= mi {
                return None;
            }
        }
        
        self.iteration += 1;
        self.calculator.value(self.iteration)
    }
}

pub struct Exponential {
    base: u64,
}

impl Exponential {
    pub fn new(base: u64) -> Self {
        Exponential { base: base }
    }

    pub fn new_binary() -> Self {
        Exponential { base: 2 }
    }
}

impl BackoffCalculator for Exponential {
    fn value(&mut self, iteration: u64) -> Option<u64> {
        Some(self.base.pow(iteration as u32) - 1)
    }
}

pub struct Geometric {
    factor: u64
}

impl Geometric {
    pub fn new(factor: u64) -> Self {
        Geometric { factor: factor }
    }
}

impl BackoffCalculator for Geometric {
    fn value(&mut self, iteration: u64) -> Option<u64> {
        Some(self.factor * iteration)
    }
}

pub struct Clamped<T> 
    where T: BackoffCalculator
{
    calculator: T,
    current_value: u64,
    max_value: u64,
}

impl<T> Clamped<T> 
    where T: BackoffCalculator
{
    pub fn new(calculator: T, max_value: u64) -> Self {
        Clamped { calculator: calculator, current_value: 0, max_value: max_value }
    }
}

impl<T> BackoffCalculator for Clamped<T> 
    where T: BackoffCalculator
{
    fn value(&mut self, iteration: u64) -> Option<u64> {
        // check max value prior to calculations, to avoid integer overflow 
        if self.current_value >= self.max_value {
            return Some(self.max_value);
        }
        
        if let Some(val) = self.calculator.value(iteration) {
            self.current_value = val;
        } else {
            return None;
        }
        
        // now check to see if the newly calculated one is over the limit and clamp 
        // the current value and return value
        if self.current_value >= self.max_value {
            self.current_value = self.max_value;
        }
        
        Some(self.current_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct N { }
    impl BackoffCalculator for N {
        fn value(&mut self, _iteration: u64) -> Option<u64> { Some(0) }
    }

    #[test]
    fn max_iters_0() {
        assert_eq!(0, BackoffSequenceIter::new(N{}, Some(0)).collect::<Vec<_>>().len());
    }

    #[test]
    fn max_iters_1() {
        assert_eq!(1, BackoffSequenceIter::new(N{}, Some(1)).collect::<Vec<_>>().len());
    }

    #[test]
    fn max_iters_1000() {
        assert_eq!(1000, BackoffSequenceIter::new(N{}, Some(1000)).collect::<Vec<_>>().len());
    }

    #[test]
    #[should_panic(expected = "arithmetic operation overflowed")]
    fn unbounded() {
        for _ in BackoffSequenceIter::new(Exponential::new(2), None) { }
    }

    #[test]
    fn exponential() {
        let v = BackoffSequenceIter::new(Exponential::new_binary(), Some(15)).collect::<Vec<_>>();
        assert_eq!(v, vec![1, 3, 7, 15, 31, 63, 127, 255, 511, 1023, 2047, 4095, 8191, 16383, 32767]);
    }

    #[test]
    fn geometric_factor_2() {
        let v = BackoffSequenceIter::new(Geometric::new(2), Some(10)).collect::<Vec<_>>();
        assert_eq!(v, vec![2, 4, 6, 8, 10, 12, 14, 16, 18, 20]);
    }

    #[test]
    fn geometric_factor_10() {
        let v = BackoffSequenceIter::new(Geometric::new(10), Some(10)).collect::<Vec<_>>();
        assert_eq!(v, vec![10, 20, 30, 40, 50, 60, 70, 80, 90, 100]);
    }

    #[test]
    fn clamped() {
        let v = BackoffSequenceIter::new(Clamped::new(Exponential::new(10), 150), Some(4)).collect::<Vec<_>>();
        assert_eq!(v, vec![9, 99, 150, 150]);
    }
}
