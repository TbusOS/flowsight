# Kernel Execution Flow Testing Methodology

This document describes the testing strategy for FlowSight's Linux kernel code analysis capabilities.

## Overview

FlowSight analyzes kernel code to visualize execution flows, detect callbacks, and track async mechanisms. The test suite verifies these capabilities across major kernel subsystems.

## Test Coverage

### 1. USB Driver Analysis (`test_usb_storage_driver_analysis`)

**Purpose:** Test analysis of real Linux kernel USB storage driver code.

**Test Approach:**
- Parses actual kernel source file: `linux_kernel/drivers/usb/storage/debug.c`
- Verifies function detection, call graph building, and flow tree generation

**Key Validations:**
- Functions discovered in real kernel code
- Struct definitions parsed correctly
- Flow trees built for entry points

### 2. Kernel Call Chain Injection (`test_kernel_callchain_injection`)

**Purpose:** Verify kernel-specific execution path injection for callbacks.

**Test Code Pattern:**
```c
struct my_usb_dev {
    struct work_struct work;
    struct timer_list timer;
};

// Async callbacks registered via macros
INIT_WORK(&dev->work, my_work_handler);
timer_setup(&dev->timer, my_timer_fn, 0);

// Ops table callbacks
static struct usb_driver my_driver = {
    .probe = my_probe,
    .disconnect = my_disconnect,
};
```

**Key Validations:**
- `my_work_handler` marked as callback (via INIT_WORK)
- `my_timer_fn` marked as callback (via timer_setup)
- `my_probe` marked as callback (via usb_driver.ops)
- `my_disconnect` marked as callback (via usb_driver.ops)

### 3. Simple Driver Execution Flow (`test_simple_driver_execution_flow`)

**Purpose:** Test basic driver patterns with module_init/module_exit and IRQ.

**Test Code Pattern:**
```c
static int my_init(void) { ... }
static void my_exit(void) { ... }
static void irq_handler(...) { ... }
static int probe_handler(...) { ... }
static int remove_handler(...) { ... }

module_init(my_init);
module_exit(my_exit);
module_platform_driver(my_driver);
```

**Key Validations:**
- Entry points detected: `my_init`, `my_exit`, `probe_handler`, `remove_handler`
- Flow trees generated for each entry point

### 4. Network Device Driver (`test_netdev_driver_analysis`)

**Purpose:** Test `net_device_ops` callback detection for NIC drivers.

**Test Code Pattern:**
```c
static int my_open(struct net_device *dev) { ... }
static int my_stop(struct net_device *dev) { ... }
static netdev_tx_t my_start_xmit(struct sk_buff *skb, struct net_device *dev) { ... }
static void my_tx_timeout(struct net_device *dev) { ... }

static const struct net_device_ops my_netdev_ops = {
    .ndo_open = my_open,
    .ndo_stop = my_stop,
    .ndo_start_xmit = my_start_xmit,
    .ndo_tx_timeout = my_tx_timeout,
};
```

**Key Validations:**
- `my_open` → `ndo_open` callback detected
- `my_stop` → `ndo_stop` callback detected
- `my_start_xmit` → `ndo_start_xmit` callback detected
- `my_interrupt` → callback via `request_irq`
- Entry points: `my_netdev_init`, `my_netdev_exit`

### 5. Block Device Driver (`test_block_device_driver_analysis`)

**Purpose:** Test block device request queue handling and bio processing.

**Test Code Pattern:**
```c
static blk_status_t my_blkdev_request(struct request_queue *q, struct request *req) {
    while ((bio = blk_fetch_request(req)) != NULL) {
        my_process_bio(priv, bio);
    }
}
static blk_status_t my_process_bio(...) { ... }

blk_queue_make_request(queue, my_blkdev_request);
```

**Key Validations:**
- Call graph: `my_blkdev_request` → `my_process_bio`
- Entry points: `my_blkdev_init`, `my_blkdev_exit`

### 6. Character Device Driver (`test_char_device_driver_analysis`)

**Purpose:** Test `file_operations` callback detection.

**Test Code Pattern:**
```c
static int my_chardev_open(...) { ... }
static int my_chardev_release(...) { ... }
static ssize_t my_chardev_read(...) {
    copy_to_user(buf, ...);  // Kernel API call
}
static ssize_t my_chardev_write(...) {
    copy_from_user(...);     // Kernel API call
}

cdev_init(&priv->cdev, &my_fops);
```

**Key Validations:**
- Entry points: `my_chardev_init`, `my_chardev_exit`
- Call edges: `my_chardev_read` → `copy_to_user`
- Call edges: `my_chardev_write` → `copy_from_user`

### 7. IRQ Handling (`test_irq_handling_analysis`)

**Purpose:** Test comprehensive async mechanism detection in interrupt context.

**Test Code Pattern:**
```c
// Threaded IRQ
request_threaded_irq(irq, my_irq_handler, my_irq_thread_fn, ...);

// Tasklet
tasklet_init(&dev->tasklet, my_tasklet_fn, data);

// Workqueue
INIT_WORK(&dev->bottom_half, my_bottom_half);

// Softirq
open_softirq(MY_SOFTIRQ, my_softirq_handler);
```

**Key Validations:**
- Async bindings detected for tasklet, workqueue, softirq, threaded IRQ
- Callback functions marked correctly
- Call edges: `my_device_open` → `request_threaded_irq`
- Call edges: `my_device_release` → `free_irq`

## Async Mechanism Detection

The test suite verifies detection of these Linux kernel async patterns:

| Mechanism | Registration Function | Handler Type |
|-----------|----------------------|--------------|
| WorkQueue | `INIT_WORK`, `schedule_work` | `work_struct` callback |
| Timer | `timer_setup`, `mod_timer` | `timer_list` callback |
| Tasklet | `tasklet_init`, `tasklet_schedule` | `tasklet_struct` callback |
| Softirq | `open_softirq`, `raise_softirq` | `softirq_action` handler |
| Threaded IRQ | `request_threaded_irq` | `irq_handler_t` + `irq_handler_t` |
| IRQ | `request_irq` | `irq_handler_t` |

## Callback Detection Methods

FlowSight detects callbacks through:

1. **Ops Table Analysis:** Scan `struct xxx_ops` initializations for `.field = function` patterns
2. **Async Binding Analysis:** Match handler registration functions with callback signatures
3. **Container Pattern Recognition:** Identify `container_of()` usage to link callbacks to their registration

## Running the Tests

```bash
# Run kernel analysis tests only
cargo test --package flowsight-analysis --test kernel_analysis_test

# Run all tests
cargo test --workspace

# Run with verbose output
cargo test --package flowsight-analysis --test kernel_analysis_test -- --nocapture
```

## Test Output Example

```
=== Network Device Driver Analysis ===
Functions found: 9
Entry points: ["my_netdev_init", "my_netdev_exit"]
Async bindings: 1
Call edges: 20
Flow trees: 6

--- Netdev Callbacks ---
  - my_open (ops: Some("netdev_ops.ndo_open"))
  - my_stop (ops: Some("netdev_ops.ndo_stop"))
  - my_start_xmit (ops: Some("netdev_ops.ndo_start_xmit"))
  - my_interrupt (ops: None)

✓ Network device callbacks correctly identified
```
