#!/usr/bin/env bash

apt-get update
apt-get install -y unzip wget

wget -nv https://github.com/hirosystems/clarinet/releases/download/v1.7.0/clarinet-linux-x64-glibc.tar.gz -O clarinet-linux-x64.tar.gz
tar -xf clarinet-linux-x64.tar.gz
chmod +x ./clarinet
mv ./clarinet /usr/local/bin
