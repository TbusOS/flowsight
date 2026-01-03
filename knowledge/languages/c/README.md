# C 语言知识库

本目录包含 C 语言通用的异步模式和回调机制。

## 文件列表

| 文件 | 描述 |
|------|------|
| `async_patterns.yaml` | 通用异步模式（适用于 Linux 内核等） |

## 与平台知识库的关系

- 本目录定义**语言级**的通用模式
- 平台特定的模式应放在 `platforms/` 目录下
- 例如：`work_struct` 模式在此定义，但 `usb_driver` 框架在 `platforms/linux-kernel/drivers/` 中定义

