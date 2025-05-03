pragma circom 2.0.0;

template AA(N) {
   signal input a;
   signal input b;
   signal output c;
   c <== nn();
}

template BB(N) {
   signal input a;
   signal input b;
   signal output c;
   c <== nn(b);
}

template CC() {
   signal input a;
   signal input b;
   signal output c;
   c <== nn(b);
}

template DD() {
   signal input a;
   signal input b;
   signal output c;
   c <== nn();
}


component main = AA();