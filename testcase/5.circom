pragma circom 2.0.0;

template C(x) {
    while ( x != 3) {
        x = 3;
    }
}

template D() {
    for (var i = 1; i < 10; i += 1) {
        i += 2;
    }
}

component main = C(1);
