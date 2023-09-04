#!/bin/sh
genhtml .coverage/lcov.info --branch-coverage -o .coverage/
open .coverage/index.html
