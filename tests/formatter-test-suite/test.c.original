// ── CASE 1: Headers and includes — spacing ────────────────────────────────
#include <stdio.h>
#include <stdlib.h>
#include   <string.h>

// ── CASE 2: Struct definition — mixed spacing ─────────────────────────────
typedef struct {
    int id;
    char  name[64];
    double  price;
    int   quantity;
} Product;

// ── CASE 3: Function — parameter spacing and brace style ──────────────────
void print_product ( Product *p ) {
    printf( "ID: %d, Name: %s, Price: %.2f\n" ,
        p->id , p->name , p->price );
}

// ── CASE 4: Main function — mixed indentation ─────────────────────────────
int main ( ) {
    Product products[] = {
        {1, "Laptop",     999.99 , 10},
        { 2 , "Mouse" ,   19.99, 50 },
        {3,"Keyboard",49.99,25},
    };

    int n = sizeof(products) / sizeof(products[0]);

    for (int i=0;i<n;i++) {
      print_product(&products[i]);
    }

    // ── CASE 5: Nested if/else with inconsistent braces ───────────────────
    int x = 42;
    if (x > 100) {
        printf("large\n");
    } else if (x > 10) {
      printf("medium\n");
    } else
    {
        printf("small\n");
    }

    // ── CASE 6: Pointer arithmetic and casts ─────────────────────────────
    int arr[] = {1,2,3,4,5};
    int *ptr = arr;
    int sum=0;
    for (int i=0;i<5;i++) {
        sum += *(ptr+i);
    }
    printf("sum: %d\n",sum);

    // ── CASE 7: String operations ────────────────────────────────────────
    char   buf[256];
    snprintf(buf,sizeof(buf),"Hello, %s!","World");
    int len=strlen(buf);
    printf("%s (%d chars)\n",buf,len);

    return 0;
}
