<div align="center">

  <h1><code>cooldown-buffer</code></h1>

  <h3>
    <strong>Buffers items send through channel until nothing happens for specified time</strong>
  </h3>

  <p>
    <img src="https://img.shields.io/github/workflow/status/devzbysiu/cooldown-buffer/ci?style=for-the-badge" alt="CI status badge" />
    <a href="https://codecov.io/gh/devzbysiu/cooldown-buffer">
      <img src="https://img.shields.io/codecov/c/github/devzbysiu/cooldown-buffer?style=for-the-badge&token=f2339b3de9e44be0a902458a669c1160" alt="Code coverage"/>
    </a>
    <img src="https://img.shields.io/badge/license-MIT%2FAPACHE--2.0-blue?style=for-the-badge" alt="License"/>
  </p>

  <h3>
    <a href="#about">About</a>
    <span> | </span>
    <a href="#example">Example</a>
    <span> | </span>
    <a href="#installation">Installation</a>
    <span> | </span>
    <a href="#license">License</a>
    <span> | </span>
    <a href="#contribution">Contribution</a>
  </h3>

  <sub><h4>Built with 🦀</h4></sub>
</div>

# <p id="about">About</p>

This is very small library which allows buffering items send through a channel until the channel
have a time to "cool down".

I used this code when I wanted to group new files appearing on the FS. Imagine this: you are
scanning some document pages on your printer and you want to treat scanned pages as one document.
It's very hard to group those without knowing the content of the pages and without being able to
tell that the pages are related to the same doc. So I took different approach and assumed that if
new documents constantly appear then it's probably the same document. When the duration between
appearance of new files is bigger than X, then the scan of the document was probably finished.

# <p id="example">Example</p>

```rust
use std::time::Duration;
use cooldown_buffer::cooldown_buffer;
use std::thread;
use anyhow::Result;
use std::sync::mpsc::channel;

fn main() -> Result<()> {
    // we set our buffer to cool down after 100 milliseconds
    let (tx, rx) = cooldown_buffer::<u32>(channel(), Duration::from_millis(100));

    thread::spawn(move || -> Result<()> {
      // then we send 4 items with delay of 90 milliseconds between each,
      // this means that our buffer kept buffering those items because it didn't
      // have a time to cool down

      tx.send(1)?;
      thread::sleep(Duration::from_millis(90));
      tx.send(2)?;
      thread::sleep(Duration::from_millis(90));
      tx.send(3)?;
      thread::sleep(Duration::from_millis(90));
      tx.send(4)?;

      Ok(())
    });

    // now we are allowing it to cool down by waiting more than 100 milliseconds,
    // so it will send all buffered items at once
    thread::sleep(Duration::from_millis(200));

    let buffered = rx.recv()?;
    assert_eq!(buffered.len(), 4);

    Ok(())
}
```


# <p id="installation">Installation</p>

Add as a dependency to your `Cargo.toml`:
```toml
[dependencies]
cooldown-buffer = { git = "https://github.com/devzbysiu/cooldown-buffer", rev = "e2961ca" }
```

# <p id="license">License</p>

This project is licensed under either of

- Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option.

# <p id="contribution">Contribution</p>


Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
