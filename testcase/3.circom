pragma circom 2.0.0;

template B(N) {
    var a, b = 3;
    var x=1;
    var y,z,w;
    if ( 5 > 3 ) {
        a = 3;
    }
    else {
        b += 3;
    }
}

component main = B(2);
