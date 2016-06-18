# Backoff Sequences, making exponential backoff (even) easy(er)

This is just a quick little crate that will hopefully make your life easier when
retrying some process that you want to take longer on each sucessive iteration.

I wrote it because there are so many places where this behaivour would make
sense, but folks haven't gone to the small effort to make it so.  Hopefully this
makes it a bit eaiser.

An example wil follow.
