pragma circom 2.0.0;

template Powers(n) { // i am content
    signal input a;
    signal output powers[n];
    /*
    i
    // am
    content
    */
    powers[0] <== a;
    for (var i = 1; i < n; i++) {
        powers[i] <==  powers[i - 1] * a;
    }
}
component main = Powers(6);