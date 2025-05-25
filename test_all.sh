#!/bin/bash

cp -f ./target/debug/main .

for i in {1..23}
do
    ./main "testcase/$i.circom"
done