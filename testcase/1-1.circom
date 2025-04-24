pragma circom 2.0.0;

template Multiplier(N,M) {
    signal input a;
    signal input b;
    signal output c;
    c <== a * b;
}

component main = Multiplier(6,5);
