//! Kernel macro semantics table
//!
//! Maps common Linux kernel macros to their semantic meaning,
//! so the CFG builder can treat them correctly instead of as plain function calls.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Semantic classification of a kernel macro
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MacroSemantics {
    /// Behaves like a function call
    FunctionCall,

    /// Registers an async callback (not a call at runtime)
    AsyncRegistration {
        /// Which parameter index is the handler function
        handler_arg_index: usize,
        /// Async mechanism name
        mechanism: String,
    },

    /// Iterator macro — expands to a loop construct
    Iterator,

    /// Declaration macro — no runtime effect
    Declaration,

    /// Changes execution context (e.g., spin_lock → atomic)
    ContextChange {
        /// New context name
        new_context: String,
    },

    /// Branch hint — does not alter control flow
    BranchHint,

    /// Memory barrier — not a function call
    MemoryBarrier,

    /// Type cast / pointer arithmetic — not a function call
    TypeCast,

    /// Entry point registration (module_init, module_usb_driver)
    EntryPointRegistration {
        /// Which parameter index is the entry function
        handler_arg_index: usize,
        /// Registration kind
        kind: String,
    },
}

/// Lookup table for kernel macro semantics
#[derive(Debug, Clone)]
pub struct MacroTable {
    table: HashMap<String, MacroSemantics>,
}

impl MacroTable {
    /// Create table with built-in Linux kernel macros
    pub fn kernel_defaults() -> Self {
        let mut table = HashMap::new();

        // === Async registration macros ===
        table.insert(
            "INIT_WORK".into(),
            MacroSemantics::AsyncRegistration {
                handler_arg_index: 1,
                mechanism: "WorkQueue".into(),
            },
        );
        table.insert(
            "INIT_DELAYED_WORK".into(),
            MacroSemantics::AsyncRegistration {
                handler_arg_index: 1,
                mechanism: "WorkQueue".into(),
            },
        );
        table.insert(
            "setup_timer".into(),
            MacroSemantics::AsyncRegistration {
                handler_arg_index: 1,
                mechanism: "Timer".into(),
            },
        );
        table.insert(
            "timer_setup".into(),
            MacroSemantics::AsyncRegistration {
                handler_arg_index: 1,
                mechanism: "Timer".into(),
            },
        );
        table.insert(
            "request_irq".into(),
            MacroSemantics::AsyncRegistration {
                handler_arg_index: 1,
                mechanism: "IRQ".into(),
            },
        );
        table.insert(
            "request_threaded_irq".into(),
            MacroSemantics::AsyncRegistration {
                handler_arg_index: 2,
                mechanism: "IRQ".into(),
            },
        );
        table.insert(
            "devm_request_irq".into(),
            MacroSemantics::AsyncRegistration {
                handler_arg_index: 2,
                mechanism: "IRQ".into(),
            },
        );
        table.insert(
            "tasklet_init".into(),
            MacroSemantics::AsyncRegistration {
                handler_arg_index: 1,
                mechanism: "Tasklet".into(),
            },
        );
        table.insert(
            "tasklet_setup".into(),
            MacroSemantics::AsyncRegistration {
                handler_arg_index: 1,
                mechanism: "Tasklet".into(),
            },
        );

        // === Entry point registration ===
        table.insert(
            "module_init".into(),
            MacroSemantics::EntryPointRegistration {
                handler_arg_index: 0,
                kind: "module_init".into(),
            },
        );
        table.insert(
            "module_exit".into(),
            MacroSemantics::EntryPointRegistration {
                handler_arg_index: 0,
                kind: "module_exit".into(),
            },
        );
        table.insert(
            "module_usb_driver".into(),
            MacroSemantics::EntryPointRegistration {
                handler_arg_index: 0,
                kind: "usb_driver".into(),
            },
        );
        table.insert(
            "module_platform_driver".into(),
            MacroSemantics::EntryPointRegistration {
                handler_arg_index: 0,
                kind: "platform_driver".into(),
            },
        );
        table.insert(
            "module_i2c_driver".into(),
            MacroSemantics::EntryPointRegistration {
                handler_arg_index: 0,
                kind: "i2c_driver".into(),
            },
        );
        table.insert(
            "module_spi_driver".into(),
            MacroSemantics::EntryPointRegistration {
                handler_arg_index: 0,
                kind: "spi_driver".into(),
            },
        );
        table.insert(
            "module_pci_driver".into(),
            MacroSemantics::EntryPointRegistration {
                handler_arg_index: 0,
                kind: "pci_driver".into(),
            },
        );

        // === Iterator macros ===
        for name in &[
            "list_for_each",
            "list_for_each_entry",
            "list_for_each_entry_safe",
            "list_for_each_entry_reverse",
            "hlist_for_each_entry",
            "hlist_for_each_entry_safe",
            "for_each_netdev",
            "for_each_possible_cpu",
            "for_each_online_cpu",
            "for_each_present_cpu",
            "for_each_child_of_node",
            "for_each_available_child_of_node",
            "idr_for_each_entry",
            "radix_tree_for_each_slot",
            "rbtree_postorder_for_each_entry_safe",
        ] {
            table.insert(name.to_string(), MacroSemantics::Iterator);
        }

        // === Declaration macros ===
        for name in &[
            "DEFINE_MUTEX",
            "DEFINE_SPINLOCK",
            "DEFINE_RWLOCK",
            "DEFINE_SEMAPHORE",
            "DECLARE_WAIT_QUEUE_HEAD",
            "DECLARE_COMPLETION",
            "DECLARE_WORK",
            "DECLARE_DELAYED_WORK",
            "DECLARE_TASKLET",
            "DECLARE_BITMAP",
            "LIST_HEAD",
            "HLIST_HEAD",
            "MODULE_LICENSE",
            "MODULE_AUTHOR",
            "MODULE_DESCRIPTION",
            "MODULE_VERSION",
            "MODULE_ALIAS",
            "MODULE_DEVICE_TABLE",
            "module_param",
            "module_param_named",
            "MODULE_PARM_DESC",
            "EXPORT_SYMBOL",
            "EXPORT_SYMBOL_GPL",
        ] {
            table.insert(name.to_string(), MacroSemantics::Declaration);
        }

        // === Context change macros ===
        for (name, ctx) in &[
            ("spin_lock", "atomic"),
            ("spin_lock_bh", "atomic"),
            ("spin_lock_irq", "atomic"),
            ("spin_lock_irqsave", "atomic"),
            ("raw_spin_lock", "atomic"),
            ("raw_spin_lock_irqsave", "atomic"),
            ("spin_unlock", "process"),
            ("spin_unlock_bh", "process"),
            ("spin_unlock_irq", "process"),
            ("spin_unlock_irqrestore", "process"),
            ("raw_spin_unlock", "process"),
            ("raw_spin_unlock_irqrestore", "process"),
            ("rcu_read_lock", "rcu_read"),
            ("rcu_read_unlock", "process"),
            ("srcu_read_lock", "rcu_read"),
            ("srcu_read_unlock", "process"),
            ("mutex_lock", "mutex_held"),
            ("mutex_lock_interruptible", "mutex_held"),
            ("mutex_lock_killable", "mutex_held"),
            ("mutex_unlock", "process"),
            ("down_read", "rwsem_read"),
            ("down_write", "rwsem_write"),
            ("up_read", "process"),
            ("up_write", "process"),
            ("local_irq_disable", "irq_disabled"),
            ("local_irq_enable", "process"),
            ("local_irq_save", "irq_disabled"),
            ("local_irq_restore", "process"),
            ("preempt_disable", "preempt_disabled"),
            ("preempt_enable", "process"),
            ("local_bh_disable", "bh_disabled"),
            ("local_bh_enable", "process"),
        ] {
            table.insert(
                name.to_string(),
                MacroSemantics::ContextChange {
                    new_context: ctx.to_string(),
                },
            );
        }

        // === Branch hints ===
        table.insert("likely".into(), MacroSemantics::BranchHint);
        table.insert("unlikely".into(), MacroSemantics::BranchHint);
        table.insert("__builtin_expect".into(), MacroSemantics::BranchHint);

        // === Memory barriers ===
        for name in &[
            "barrier",
            "mb",
            "rmb",
            "wmb",
            "smp_mb",
            "smp_rmb",
            "smp_wmb",
            "smp_mb__before_atomic",
            "smp_mb__after_atomic",
            "dma_wmb",
            "dma_rmb",
        ] {
            table.insert(name.to_string(), MacroSemantics::MemoryBarrier);
        }

        // === Type casts / pointer arithmetic ===
        for name in &[
            "container_of",
            "to_usb_device",
            "to_usb_interface",
            "to_platform_device",
            "to_pci_dev",
            "to_i2c_client",
            "to_spi_device",
            "netdev_priv",
            "dev_get_drvdata",
            "platform_get_drvdata",
            "usb_get_intfdata",
            "i2c_get_clientdata",
            "spi_get_drvdata",
            "pci_get_drvdata",
        ] {
            table.insert(name.to_string(), MacroSemantics::TypeCast);
        }

        Self { table }
    }

    /// Look up macro semantics
    pub fn lookup(&self, name: &str) -> Option<&MacroSemantics> {
        self.table.get(name)
    }

    /// Check if a name is a known macro
    pub fn is_known(&self, name: &str) -> bool {
        self.table.contains_key(name)
    }

    /// Add or override a macro entry
    pub fn insert(&mut self, name: String, semantics: MacroSemantics) {
        self.table.insert(name, semantics);
    }

    /// Number of entries
    pub fn len(&self) -> usize {
        self.table.len()
    }

    /// Is empty
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }
}

impl Default for MacroTable {
    fn default() -> Self {
        Self::kernel_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kernel_defaults_populated() {
        let table = MacroTable::kernel_defaults();
        assert!(table.len() > 80, "Expected 80+ macros, got {}", table.len());
    }

    #[test]
    fn test_async_registration_lookup() {
        let table = MacroTable::kernel_defaults();

        match table.lookup("INIT_WORK") {
            Some(MacroSemantics::AsyncRegistration {
                handler_arg_index,
                mechanism,
            }) => {
                assert_eq!(*handler_arg_index, 1);
                assert_eq!(mechanism, "WorkQueue");
            }
            other => panic!("Expected AsyncRegistration, got {:?}", other),
        }
    }

    #[test]
    fn test_context_change_lookup() {
        let table = MacroTable::kernel_defaults();

        match table.lookup("spin_lock") {
            Some(MacroSemantics::ContextChange { new_context }) => {
                assert_eq!(new_context, "atomic");
            }
            other => panic!("Expected ContextChange, got {:?}", other),
        }

        match table.lookup("spin_unlock") {
            Some(MacroSemantics::ContextChange { new_context }) => {
                assert_eq!(new_context, "process");
            }
            other => panic!("Expected ContextChange, got {:?}", other),
        }
    }

    #[test]
    fn test_iterator_lookup() {
        let table = MacroTable::kernel_defaults();
        assert!(matches!(
            table.lookup("list_for_each_entry"),
            Some(MacroSemantics::Iterator)
        ));
    }

    #[test]
    fn test_declaration_lookup() {
        let table = MacroTable::kernel_defaults();
        assert!(matches!(
            table.lookup("DEFINE_MUTEX"),
            Some(MacroSemantics::Declaration)
        ));
        assert!(matches!(
            table.lookup("MODULE_LICENSE"),
            Some(MacroSemantics::Declaration)
        ));
    }

    #[test]
    fn test_unknown_macro() {
        let table = MacroTable::kernel_defaults();
        assert!(table.lookup("my_custom_function").is_none());
    }
}
