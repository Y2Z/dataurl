FROM ekidd/rust-musl-builder as builder

RUN curl -L -o dataurl.tar.gz $(curl -s https://api.github.com/repos/y2z/dataurl/releases/latest \
                                 | grep "tarball_url.*\"," \
                                 | cut -d '"' -f 4)
RUN tar xfz dataurl.tar.gz \
    && mv Y2Z-dataurl-* dataurl \
    && rm dataurl.tar.gz

WORKDIR dataurl/
RUN make install


FROM alpine

RUN apk update && \
  apk add --no-cache openssl && \
  rm -rf "/var/cache/apk/*"

COPY --from=builder /home/rust/.cargo/bin/dataurl /usr/bin/dataurl
WORKDIR /tmp
ENTRYPOINT ["/usr/bin/dataurl"]
