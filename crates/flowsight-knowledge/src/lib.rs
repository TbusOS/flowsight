//! FlowSight Knowledge Base
//!
//! Contains framework patterns, API information, and callback timing.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Framework callback information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkCallback {
    /// Callback description
    pub description: String,
    /// When the callback is triggered
    pub trigger: String,
    /// Execution context
    pub context: String,
    /// Function signature
    pub signature: Option<String>,
}

/// Framework definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Framework {
    /// Framework description
    pub description: String,
    /// Header file
    pub header: Option<String>,
    /// Callbacks defined by this framework
    pub callbacks: HashMap<String, FrameworkCallback>,
}

/// Async pattern definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncPattern {
    /// Description
    pub description: String,
    /// Execution context
    pub context: String,
    /// Bind patterns (regex)
    pub bind_patterns: Vec<String>,
    /// Trigger patterns (regex)
    pub trigger_patterns: Vec<String>,
    /// Handler signature
    pub handler_signature: Option<String>,
}

/// Kernel API information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelApi {
    /// Description
    pub description: String,
    /// Whether it can sleep
    pub can_sleep: bool,
    /// Whether it can fail
    pub can_fail: bool,
    /// Parameter descriptions
    pub params: Option<Vec<String>>,
}

/// Knowledge base
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KnowledgeBase {
    /// Framework definitions
    pub frameworks: HashMap<String, Framework>,
    /// Async patterns
    pub async_patterns: HashMap<String, AsyncPattern>,
    /// Kernel API info
    pub kernel_apis: HashMap<String, KernelApi>,
}

impl KnowledgeBase {
    /// Create an empty knowledge base
    pub fn new() -> Self {
        Self::default()
    }

    /// Load knowledge base from YAML file
    pub fn load_yaml(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let kb: KnowledgeBase = serde_yaml::from_str(&content)?;
        Ok(kb)
    }

    /// Load knowledge base from JSON file
    pub fn load_json(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let kb: KnowledgeBase = serde_json::from_str(&content)?;
        Ok(kb)
    }

    /// Load built-in knowledge base
    pub fn builtin() -> Self {
        let mut kb = Self::new();
        kb.load_builtin_frameworks();
        kb.load_builtin_apis();
        kb
    }

    fn load_builtin_frameworks(&mut self) {
        // USB driver framework
        let mut usb_callbacks = HashMap::new();
        usb_callbacks.insert(
            "probe".into(),
            FrameworkCallback {
                description: "Called when a USB device matching the ID table is connected".into(),
                trigger: "USB device insertion and ID match".into(),
                context: "Process context, can sleep".into(),
                signature: Some(
                    "int (*)(struct usb_interface *, const struct usb_device_id *)".into(),
                ),
            },
        );
        usb_callbacks.insert(
            "disconnect".into(),
            FrameworkCallback {
                description: "Called when the USB device is disconnected".into(),
                trigger: "USB device removal or driver unload".into(),
                context: "Process context, can sleep".into(),
                signature: Some("void (*)(struct usb_interface *)".into()),
            },
        );

        self.frameworks.insert(
            "usb_driver".into(),
            Framework {
                description: "USB device driver framework".into(),
                header: Some("linux/usb.h".into()),
                callbacks: usb_callbacks,
            },
        );

        // file_operations
        let mut fops_callbacks = HashMap::new();
        fops_callbacks.insert(
            "open".into(),
            FrameworkCallback {
                description: "Called when the device is opened".into(),
                trigger: "User space open() system call".into(),
                context: "Process context".into(),
                signature: Some("int (*)(struct inode *, struct file *)".into()),
            },
        );
        fops_callbacks.insert(
            "read".into(),
            FrameworkCallback {
                description: "Called to read from the device".into(),
                trigger: "User space read() system call".into(),
                context: "Process context".into(),
                signature: Some(
                    "ssize_t (*)(struct file *, char __user *, size_t, loff_t *)".into(),
                ),
            },
        );
        fops_callbacks.insert(
            "write".into(),
            FrameworkCallback {
                description: "Called to write to the device".into(),
                trigger: "User space write() system call".into(),
                context: "Process context".into(),
                signature: Some(
                    "ssize_t (*)(struct file *, const char __user *, size_t, loff_t *)".into(),
                ),
            },
        );

        self.frameworks.insert(
            "file_operations".into(),
            Framework {
                description: "Character device file operations".into(),
                header: Some("linux/fs.h".into()),
                callbacks: fops_callbacks,
            },
        );
    }

    fn load_builtin_apis(&mut self) {
        self.kernel_apis.insert(
            "kzalloc".into(),
            KernelApi {
                description: "Allocate zeroed kernel memory".into(),
                can_sleep: true,
                can_fail: true,
                params: Some(vec!["size".into(), "gfp_flags".into()]),
            },
        );

        self.kernel_apis.insert(
            "kfree".into(),
            KernelApi {
                description: "Free kernel memory".into(),
                can_sleep: false,
                can_fail: false,
                params: Some(vec!["ptr".into()]),
            },
        );

        self.kernel_apis.insert(
            "mutex_lock".into(),
            KernelApi {
                description: "Acquire a mutex".into(),
                can_sleep: true,
                can_fail: false,
                params: Some(vec!["mutex".into()]),
            },
        );

        self.kernel_apis.insert(
            "spin_lock".into(),
            KernelApi {
                description: "Acquire a spinlock".into(),
                can_sleep: false,
                can_fail: false,
                params: Some(vec!["lock".into()]),
            },
        );

        self.kernel_apis.insert(
            "printk".into(),
            KernelApi {
                description: "Print a kernel message".into(),
                can_sleep: false,
                can_fail: false,
                params: Some(vec!["fmt".into(), "...".into()]),
            },
        );
    }

    /// Get framework info
    pub fn get_framework(&self, name: &str) -> Option<&Framework> {
        self.frameworks.get(name)
    }

    /// Get callback info
    pub fn get_callback(&self, framework: &str, callback: &str) -> Option<&FrameworkCallback> {
        self.frameworks.get(framework)?.callbacks.get(callback)
    }

    /// Get API info
    pub fn get_api(&self, name: &str) -> Option<&KernelApi> {
        self.kernel_apis.get(name)
    }
}
