#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>

#define NUM_THREADS 20
#define NUM_INCREMENTS 1000000

int counter;

void *AddThings(void *threadid) {
  for (int i = 0; i < NUM_INCREMENTS; i++)
    counter += 1;
  pthread_exit(NULL);
}

int main(int argc, char *argv[]) {
  pthread_t threads[NUM_THREADS];
  long t;
  for (t = 0; t < NUM_THREADS; t++) {
    int rc = pthread_create(&threads[t], NULL, AddThings, (void *)t);
    if (rc) {
      printf("ERROR; return code from pthread_create() is %d\n", rc);
      exit(1);
    }
  }
  // Wait for threads to finish
  for (t = 0; t < NUM_THREADS; t++)
    pthread_join(threads[t], NULL);
  printf("Final value of counter is: %d\n", counter);
  pthread_exit(NULL);
}
