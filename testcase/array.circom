pragma circom 2.0.0;

template Powers(n) { // i am comment
    signal input a;
    signal output powers[n];
    /*
    i
    // am
    comment
    */
    powers[0] <== a;
    for (var i = 1; i < n; i++) {
        powers[i] <==  powers[i - 1] * a;
    }
}
component main = Powers(6);