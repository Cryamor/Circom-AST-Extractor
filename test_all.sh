#!/bin/bash

cp -f ./target/debug/Circom_AST_Extractor .

for i in {1..23}
do
    ./Circom_AST_Extractor "testcase/$i.circom"
done