CC = gcc
CFLAGS = -ffreestanding -O2 -Wall -Wextra -fno-pie -fno-stack-protector -mno-red-zone
LDFLAGS = -nostdlib -static -no-pie

all: driver.elf

driver.elf: driver.c libc.c
	$(CC) $(CFLAGS) -c driver.c -o driver.o
	$(CC) $(CFLAGS) -c libc.c -o libc.o
	$(CC) $(LDFLAGS) driver.o libc.o -o driver.elf

clean:
	rm -f *.o *.elf
