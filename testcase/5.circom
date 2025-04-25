pragma circom 2.0.0;

template C(x,y) {
    while ( x != 3) {
        x = 3;
    }
}

component main = C(1,2);
