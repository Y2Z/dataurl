[![dataurl build status on GNU/Linux](https://github.com/Y2Z/dataurl/workflows/GNU%2FLinux/badge.svg)](https://github.com/Y2Z/dataurl/actions?query=workflow%3AGNU%2FLinux)
[![dataurl build status on macOS](https://github.com/Y2Z/dataurl/workflows/macOS/badge.svg)](https://github.com/Y2Z/dataurl/actions?query=workflow%3AmacOS)
[![dataurl build status on Windows](https://github.com/Y2Z/dataurl/workflows/Windows/badge.svg)](https://github.com/Y2Z/dataurl/actions?query=workflow%3AWindows)

# dataurl

CLI tool / Rust crate for converting files and text into data URLs and back


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

assert_eq!(data_url.get_media_type(), "text/plain".to_string());
assert_eq!(data_url.get_media_type_no_default(), None);
assert_eq!(data_url.get_charset(), "US-ASCII".to_string());
assert_eq!(data_url.get_charset_no_default(), None);
assert!(!data_url.get_is_base64_encoded());
assert_eq!(data_url.get_data(), [72, 101, 108, 108, 111, 44, 32, 87, 111, 114, 108, 100, 33]);
assert_eq!(data_url.get_fragment(), None);
assert_eq!(data_url.to_string(), "data:,Hello%2C%20World%21");
assert_eq!(data_url.get_text(), "Hello, World!");
```


---------------------------------------------------


## Usage (CLI)

```console
dataurl "some text"
```
val#f' > index.html
```console
dataurl -d 'data:text/html,text<a id%3D"b">ok</a>?a=v#f' > index.html
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


## Flags and options

 - `-b`: Encode data using base64
 - `-d`: Attempt to parse input, output resulting data

 - `-c`: Use custom `charset`
 - `-f`: Append `fragment`
 - `-i`: Specify `file` to read data from (use `-` for STDIN)
 - `-m`: Adjust `media type`


---------------------------------------------------


## References

 - https://datatracker.ietf.org/doc/html/rfc2397
 - https://datatracker.ietf.org/doc/html/rfc6838


---------------------------------------------------


## License

To the extent possible under law, the author(s) have dedicated all copyright related and neighboring rights to this software to the public domain worldwide.
This software is distributed without any warranty.
