/*
gcc -g -O2 -Wall -c src/example.c -o main.o && gcc -o test main.o ./target/debug/libresource_mng.so && ./test
*/

#include <stdio.h>
#include <inttypes.h>
#include <stdbool.h>

typedef struct Instance *box;

extern uint8_t add_material (box b, double supply);
extern uint8_t add_product (box b, size_t material_id, double material_amount, size_t priority, double work_complexity);
extern uint8_t order_product (box b, size_t id, double amount, size_t variant_id, size_t user_id, bool allow_substitution);
extern double get_material_supply (box b, size_t id);

extern box init ();

int main ()
{
    size_t material_id = 0;
    size_t product_id = material_id;
    box b = init();
    uint8_t result1 = add_material (b, 0.0); // 2
    printf("%u\n", result1);
    result1 = add_material (b, 100.0); // 0 ok
    printf("%u\n", result1);

    double result2 = get_material_supply (b, 0); // 100
    printf("%f\n", result2);

    result1 = add_product (b, material_id, 50.0, 0, 1.0); // 0
    printf("%u\n", result1);
    result1 = add_product (b, material_id, 101.0, 0, 1.0); // 0
    printf("%u\n", result1);
    result1 = add_product (b, 9999, 101.0, 0, 1.0); // 4
    printf("%u\n", result1);

    result1 = order_product (b, product_id, 2.0, 0, 0, true); // 1
    printf("%u\n", result1);

    return 0;
}
