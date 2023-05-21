CC := gcc
CFLAGS := -Wall -Wextra -pedantic
SRC_DIR := src
FORMAT := clang-format

# search for all *.c files
SRC_FILES := $(shell find $(SRC_DIR) -name '*.c')
HEADER_FILES := $(shell find $(SRC_DIR) -name '*.h')

all: todo

# %.o: %.c %.h
# 	$(CC) $(CLFAGS) -c $^

todo: $(SRC_FILES) $(HEADER_FILES)
	$(CC) $(CFLAGS) -o $@ $(SRC_FILES)

run: todo
	./todo

format:
	$(FORMAT) -i $(SRC_FILES) $(HEADER_FILES)
	
clean:
	rm todo