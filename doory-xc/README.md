# doory-xc

Dockerfile for building a docker image with the openwrt sdk for rt305x, nightly
rust / cargo and precompiled libnfc. This build environment is needed for
compiling `doory-reader` and `doory-strikeplate`.

## Building the image
```
docker build -t doory-xc .
```

## Building doory-reader using the container
```
docker run -it -v /path/to/doory:/source -w /source doory-xc
cd doory-reader
make
```

output is in `doory/target/mipsel-unknown-linux-musl/release/doory-reader`

## Building doory-strikeplate using the container
```
docker run -it -v /path/to/doory:/source -w /source doory-xc
cd doory-strikeplate
make
```

output is in `doory/target/mipsel-unknown-linux-musl/release/doory-strikeplate`
