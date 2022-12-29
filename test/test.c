#include <stdio.h>
int main() {
    int base, exp;
    long double result = 1.0;
    printf("Enter a base number: \n");
    base = 6;
    printf("Enter an exponent: \n");
    exp = 5;

    while (exp != 0) {
        result *= base;
        --exp;
    }
    printf("Answer = %.0Lf \n", result);
    printf("A division = %.10f \n", 10. / 3);
    return 0;
}
