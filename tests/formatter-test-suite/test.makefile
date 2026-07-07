# ── CASE 1: Variables — spacing ────────────────────────────────────────────
CC      := gcc
CXX     := g++
AR      := ar
RANLIB  := ranlib
RM      := rm -f

CFLAGS  := -Wall -Wextra -std=c11   -O2
CXXFLAGS:= -Wall -Wextra -std=c++17 -O2
LDFLAGS := -L./lib
LDLIBS  := -lm -lpthread

# ── CASE 2: Directory variables ─────────────────────────────────────────────
SRC_DIR := src
OBJ_DIR := obj
BIN_DIR := bin
LIB_DIR := lib
INC_DIR := include

# ── CASE 3: Source discovery and object mapping ─────────────────────────────
SRCS    := $(wildcard $(SRC_DIR)/*.c)
OBJS    := $(patsubst $(SRC_DIR)/%.c,$(OBJ_DIR)/%.o,$(SRCS))
DEPS    := $(OBJS:.o=.d)
TARGET  := $(BIN_DIR)/myapp

# ── CASE 4: Default target ───────────────────────────────────────────────────
.PHONY: all clean install uninstall test

all: $(TARGET)

# ── CASE 5: Linking rule ─────────────────────────────────────────────────────
$(TARGET): $(OBJS) | $(BIN_DIR)
	$(CC) $(LDFLAGS) -o $@ $^ $(LDLIBS)

# ── CASE 6: Compilation pattern rule ────────────────────────────────────────
$(OBJ_DIR)/%.o: $(SRC_DIR)/%.c | $(OBJ_DIR)
	$(CC) $(CFLAGS) -I$(INC_DIR) -MMD -MP -c -o $@ $<

# ── CASE 7: Directory creation ───────────────────────────────────────────────
$(BIN_DIR) $(OBJ_DIR) $(LIB_DIR):
	mkdir -p $@

# ── CASE 8: Include dependency files ─────────────────────────────────────────
-include $(DEPS)

# ── CASE 9: Phony targets ─────────────────────────────────────────────────
clean:
	$(RM) -r $(OBJ_DIR) $(BIN_DIR)
	$(RM) $(DEPS)

install: all
	install -D $(TARGET) /usr/local/bin/myapp
	install -D -m 644 $(INC_DIR)/*.h /usr/local/include/myapp/

uninstall:
	$(RM) /usr/local/bin/myapp
	$(RM) -r /usr/local/include/myapp/

# ── CASE 10: Test target ──────────────────────────────────────────────────
TEST_SRCS := $(wildcard tests/*.c)
TEST_BINS := $(patsubst tests/%.c,$(BIN_DIR)/test_%,$(TEST_SRCS))

test: $(TEST_BINS)
	@for t in $(TEST_BINS); do \
		echo "Running $$t..."; \
		$$t || exit 1; \
	done
	@echo "All tests passed."

$(BIN_DIR)/test_%: tests/%.c $(filter-out $(OBJ_DIR)/main.o,$(OBJS)) | $(BIN_DIR)
	$(CC) $(CFLAGS) -I$(INC_DIR) $(LDFLAGS) -o $@ $< $(filter-out $(OBJ_DIR)/main.o,$(OBJS)) $(LDLIBS)

# ── CASE 11: Long recipe line ─────────────────────────────────────────────
release: CFLAGS += -DNDEBUG -O3 -march=native -flto
release: LDFLAGS += -flto
release: clean all
	strip $(TARGET)
	@echo "Release build complete: $(TARGET)"
