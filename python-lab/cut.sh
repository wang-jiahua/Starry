#!/bin/bash
cd ./opt/python3.11

rm lib/*.a lib/*/*.a lib/*/*/*.a
rm -r `find -name __pycache__`

aarch64-linux-musl-strip -s bin/* lib/* lib/python3.11/lib-dynload/*