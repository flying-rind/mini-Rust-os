#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define TEST_SIZE 256

int main() {
    // Test 1: Basic allocation and free
    printf("=== Basic malloc/free test ===\n");
    int *arr = malloc(5 * sizeof(int));
    if (!arr) {
        perror("malloc failed");
        return 1;
    }
    
    for (int i = 0; i < 5; i++) {
        arr[i] = i * 10;
    }
    
    printf("Array values: ");
    for (int i = 0; i < 5; i++) {
        printf("%d ", arr[i]);
    }
    printf("\n");
    
    free(arr);
    printf("Memory freed successfully\n\n");

    // Test 2: String allocation
    printf("=== String allocation test ===\n");
    char *str = malloc(TEST_SIZE);
    if (!str) {
        perror("malloc failed");
        return 1;
    }
    
    strcpy(str, "Hello, memory management!");
    printf("String content: %s\n", str);
    free(str);
    printf("String memory freed\n\n");
    return 0;
}