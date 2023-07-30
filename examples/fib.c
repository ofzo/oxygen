#include <stdio.h>

int fib(int num, int t[])
{
  if (t[num] > 0)
  {
    return t[num];
  }
  t[num] = fib(num - 1, t) + fib(num - 2, t);
  return t[num];
}

int main()
{
  int v = 40;
  int table[41] = {1, 1};
  int value = fib(v, table);
  printf("fib(40) = %d\n", value);
  return 1;
}
