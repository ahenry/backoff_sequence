# Backoff Sequence
## making exponential backoff (even) easy(er)

This is just a quick little crate that will hopefully make your life easier when
retrying some process that you want to take longer on each sucessive iteration.

I wrote it because there are so many places where this behaivour would make
sense, but folks haven't gone to the small effort to make it so.  Hopefully this
makes it a bit eaiser.

At this point, there's a distinct lack of convenience functions, and the API is
a tiny bit awkward on account of function pointers and closures not being Copy.

## Examples

This demonstrates use in a loop where you want to try a limited number of times,
with a minimum delay of 30ms and a max of 100ms

```rust
extern crate backoff_sequence;
use backoff_sequence::BackoffSequence;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let f = &|i| Duration::from_millis(2u64.pow(i as u32));
    let mut backoff = BackoffSequence::new(f);
    backoff.min(Duration::from_millis(30).
            max(Duration::from_millis(100)).
            max_iterations(10);

    // ... lots of code ...

    for delay in &backoff {
        sleep(delay);
    }
}
```

This next example is a bit more realistic.  We'd like to loop forever waiting
for some network resource to become available

```rust
extern crate backoff_sequence;
use backoff_sequence::BackoffSequence;
use std::net::TcpStream;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let f = &|i| Duration::from_millis(2u64.pow(i as u32));
    let mut backoff = BackoffSequence::new(f);
    backoff.max(Duration::from_millis(100));

    // .. do stuff ..

    let mut delay = backoff.iter();
    loop {
        let conn = TcpStream::connect("127.0.0.1:8000");
        match conn {
            Ok(_) => break,
            Err(_) => sleep(delay.next().unwrap()),
        }
    }
}
```
