#include <stdio.h>
#include <string.h>

void test_strncmp(const char *s1, const char *s2, size_t n) {
    int result = strncmp(s1, s2, n);
    printf("strncmp(\"%s\", \"%s\", %zu) = %d\n", s1, s2, n, result);
}

int main() {
    // Test case 1: Equal strings
    test_strncmp("hello", "hello", 5);
    
    // Test case 2: Partial comparison
    test_strncmp("hello", "hella", 4);  // Should return equal (only compares first 4 chars)
    test_strncmp("hello", "hella", 5);  // Should return positive ('o' > 'a')
    
    // Test case 3: Different lengths
    test_strncmp("short", "longer", 5);  // Compares up to min length
    test_strncmp("longer", "short", 5);  // Same as above
    
    // Test case 4: Zero-length comparison
    test_strncmp("anything", "anything", 0);  // Should always return 0
    
    // Test case 5: One empty string
    test_strncmp("", "non-empty", 3);
    test_strncmp("non-empty", "", 3);
    
    // Test case 6: Non-ASCII characters
    test_strncmp("résumé", "resume", 3);  // Depends on encoding
    
    return 0;
}