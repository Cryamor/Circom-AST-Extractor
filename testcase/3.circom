pragma circom 2.0.0;

template B(N) {
    var a, b = 3;
    if ( 5 > 3 ) {
        a = 3;
    }
    else {
        b += 3;
    }
}

template C(x,y) {
    while ( x != 3) {
        x = 3;
    }
}

component main = B(2);
