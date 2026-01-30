# FlowSight 测试示例项目 Makefile

CC = gcc
CFLAGS = -Wall -Wextra -g -O0
LDFLAGS =

# 目标文件
TARGET = sample
SRCS = main.c utils.c
OBJS = $(SRCS:.c=.o)

# 默认目标
all: $(TARGET)

# 链接
$(TARGET): $(OBJS)
	$(CC) $(LDFLAGS) -o $@ $^

# 编译规则
%.o: %.c utils.h
	$(CC) $(CFLAGS) -c -o $@ $<

# 运行
run: $(TARGET)
	./$(TARGET)

# 清理
clean:
	rm -f $(OBJS) $(TARGET)

# 重新构建
rebuild: clean all

# 调试构建
debug: CFLAGS += -DDEBUG
debug: all

.PHONY: all clean run rebuild debug
