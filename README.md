[![dataurl build status on GNU/Linux](https://github.com/Y2Z/dataurl/workflows/GNU%2FLinux/badge.svg)](https://github.com/Y2Z/dataurl/actions?query=workflow%3AGNU%2FLinux)
[![dataurl build status on macOS](https://github.com/Y2Z/dataurl/workflows/macOS/badge.svg)](https://github.com/Y2Z/dataurl/actions?query=workflow%3AmacOS)
[![dataurl build status on Windows](https://github.com/Y2Z/dataurl/workflows/Windows/badge.svg)](https://github.com/Y2Z/dataurl/actions?query=workflow%3AWindows)

# dataurl

CLI tool and Rust crate for converting files into data URLs and back


---------------------------------------------------


## Installation

#### Using [Cargo](https://crates.io/crates/dataurl)

```console
cargo install dataurl
```

#### Using [containers](https://www.docker.com/)

```console
docker build -t Y2Z/dataurl .
sudo install -b dist/run-in-container.sh /usr/local/bin/dataurl
```

#### From source

```console
git clone https://github.com/Y2Z/dataurl.git
cd dataurl
make install
```

#### Using [pre-built binaries](https://github.com/Y2Z/dataurl/releases) (Windows, ARM-based devices, etc)

Every release contains pre-built binaries for Windows, GNU/Linux, as well as platforms with non-standart CPU architecture.


---------------------------------------------------


## Usage (crate)

```rust
use dataurl::DataUrl;

let data_url: DataUrl = DataUrl::parse("data:,Hello,%20World!")?;

assert_eq!(data_url.media_type(), "text/plain".to_string());
assert_eq!(data_url.media_type_no_default(), None);
assert_eq!(data_url.charset(), "US-ASCII".to_string());
assert_eq!(data_url.charset_no_default(), None);
assert!(!data_url.encoded());
assert_eq!(String::from_utf8_lossy(data_url.data()), "Hello, World!");
assert_eq!(data_url.fragment(), None);
assert_eq!(data_url.to_string(), "data:,Hello,%20World!");
```


---------------------------------------------------


## Usage (CLI)

```console
dataurl "some text"
```

```console
dataurl -d 'data:text/html,text...<p><a name%3D"bottom">bottom</a>?arg=val#something' > index.html
```

```console
dataurl -i picture.png
```

```console
cat file.txt | dataurl -i -
```

```console
cat file.png | dataurl
```

---------------------------------------------------


## Options

 - `-b`: Prefer to use base64 even when not necessary
 - `-c`: Use custom `charset` (automatically sets `-b` if not `US-ASCII` or `windows-1252`)
 - `-d`: Attempt to parse input and output resulting data
 - `-f`: Append custom `fragment`
 - `-i`: Path to `file` to read input from (use `-` for STDIN)
 - `-t`: Specify custom `media type`


---------------------------------------------------


## References

 - https://datatracker.ietf.org/doc/html/rfc2397
 - https://datatracker.ietf.org/doc/html/rfc6838


---------------------------------------------------


## License

To the extent possible under law, the author(s) have dedicated all copyright related and neighboring rights to this software to the public domain worldwide.
This software is distributed without any warranty.
