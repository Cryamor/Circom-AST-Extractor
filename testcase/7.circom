pragma circom 2.0.0;

template Multiplier() {
    signal input a;
    signal input b;
    signal output c;
    signal d;
    c <== a * b;
    d <== a + b;
}

component main = Multiplier();
