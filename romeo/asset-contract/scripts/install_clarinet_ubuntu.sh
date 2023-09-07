#!/usr/bin/env bash

apt-get update
apt-get install -y unzip curl lcov

curl -s https://api.github.com/repos/hirosystems/clarinet/releases/latest | grep "/clarinet-linux-x64-glibc.tar.gz" | cut -d : -f 2,3 | tr -d \" | wget -qi -
tar -xzf clarinet-linux-x64-glibc.tar.gz
chmod +x ./clarinet
mv ./clarinet /usr/local/bin
