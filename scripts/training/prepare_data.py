#!/usr/bin/env python3
"""
FlowSight AI 训练数据准备工具

数据来源：
1. FlowSight 知识库自动转换
2. Linux 内核源码分析
3. 开源项目分析
4. LLM 辅助生成
5. 人工标注

使用方法：
    python prepare_data.py --source knowledge --output data/
    python prepare_data.py --source kernel --kernel-path /path/to/linux
    python prepare_data.py --source llm --count 1000
    python prepare_data.py --merge --output data/train.jsonl
"""

import argparse
import json
import os
import re
from pathlib import Path
from typing import List, Dict, Optional
from dataclasses import dataclass, asdict
import subprocess

# ============================================================
# 数据结构定义
# ============================================================

@dataclass
class TrainingSample:
    """训练样本"""
    task: str  # function_pointer_target, callback_timing, async_pattern, call_chain
    instruction: str
    input: str  # 代码
    output: str  # 答案
    metadata: Optional[Dict] = None
    
    def to_dict(self):
        d = {
            "task": self.task,
            "instruction": self.instruction,
            "input": self.input,
            "output": self.output,
        }
        if self.metadata:
            d["metadata"] = self.metadata
        return d

# ============================================================
# 来源1：从 FlowSight 知识库生成
# ============================================================

class KnowledgeBaseConverter:
    """从 FlowSight 知识库转换训练数据"""
    
    # Linux 内核框架知识（硬编码，后续可以从 flowsight-knowledge 读取）
    FRAMEWORKS = {
        "usb_driver": {
            "callbacks": {
                "probe": {
                    "trigger": "USB 设备插入且 ID 匹配",
                    "context": "进程上下文（可睡眠）",
                    "call_chain": [
                        "usb_hub_port_connect",
                        "usb_new_device", 
                        "device_add",
                        "bus_probe_device",
                        "driver_probe_device",
                        "really_probe",
                        "usb_probe_interface",
                        "drv->probe()"
                    ]
                },
                "disconnect": {
                    "trigger": "USB 设备拔出",
                    "context": "进程上下文（可睡眠）",
                    "call_chain": [
                        "usb_disconnect",
                        "device_del",
                        "bus_remove_device",
                        "drv->disconnect()"
                    ]
                }
            }
        },
        "platform_driver": {
            "callbacks": {
                "probe": {
                    "trigger": "设备树匹配或 platform_device_register",
                    "context": "进程上下文",
                    "call_chain": [
                        "platform_device_add",
                        "device_add",
                        "bus_probe_device",
                        "really_probe",
                        "platform_drv_probe",
                        "drv->probe()"
                    ]
                },
                "remove": {
                    "trigger": "设备移除或模块卸载",
                    "context": "进程上下文",
                    "call_chain": []
                }
            }
        },
        "file_operations": {
            "callbacks": {
                "open": {
                    "trigger": "用户调用 open() 系统调用",
                    "context": "进程上下文",
                    "call_chain": [
                        "sys_open",
                        "do_sys_open", 
                        "do_filp_open",
                        "vfs_open",
                        "do_dentry_open",
                        "f_op->open()"
                    ]
                },
                "read": {
                    "trigger": "用户调用 read() 系统调用",
                    "context": "进程上下文",
                    "call_chain": ["sys_read", "vfs_read", "f_op->read()"]
                },
                "write": {
                    "trigger": "用户调用 write() 系统调用",
                    "context": "进程上下文",
                    "call_chain": ["sys_write", "vfs_write", "f_op->write()"]
                },
                "release": {
                    "trigger": "文件描述符关闭（最后一个引用）",
                    "context": "进程上下文",
                    "call_chain": []
                }
            }
        }
    }
    
    ASYNC_PATTERNS = {
        "workqueue": {
            "bind_funcs": ["INIT_WORK", "INIT_DELAYED_WORK"],
            "trigger_funcs": ["schedule_work", "queue_work", "schedule_delayed_work"],
            "context": "进程上下文（可睡眠）",
            "description": "工作队列，用于延迟执行耗时操作"
        },
        "timer": {
            "bind_funcs": ["timer_setup", "DEFINE_TIMER"],
            "trigger_funcs": ["mod_timer", "add_timer"],
            "context": "软中断上下文（不可睡眠）",
            "description": "定时器，在指定时间后执行"
        },
        "tasklet": {
            "bind_funcs": ["tasklet_init", "DECLARE_TASKLET"],
            "trigger_funcs": ["tasklet_schedule", "tasklet_hi_schedule"],
            "context": "软中断上下文（不可睡眠）",
            "description": "软中断，优先级高于工作队列"
        },
        "irq": {
            "bind_funcs": ["request_irq", "devm_request_irq"],
            "trigger_funcs": [],  # 硬件触发
            "context": "硬中断上下文（不可睡眠，快速执行）",
            "description": "硬件中断处理"
        }
    }
    
    def generate_samples(self) -> List[TrainingSample]:
        samples = []
        
        # 生成框架回调样本
        for framework_name, framework in self.FRAMEWORKS.items():
            for callback_name, callback_info in framework["callbacks"].items():
                samples.extend(self._generate_callback_samples(
                    framework_name, callback_name, callback_info
                ))
        
        # 生成异步模式样本
        for pattern_name, pattern_info in self.ASYNC_PATTERNS.items():
            samples.extend(self._generate_async_samples(pattern_name, pattern_info))
        
        return samples
    
    def _generate_callback_samples(self, framework: str, callback: str, info: dict) -> List[TrainingSample]:
        samples = []
        
        # 生成触发时机样本
        code = self._generate_framework_code(framework, callback)
        samples.append(TrainingSample(
            task="callback_timing",
            instruction=f"分析以下代码中 my_{callback} 函数何时被调用",
            input=code,
            output=self._format_callback_answer(framework, callback, info),
            metadata={"framework": framework, "callback": callback}
        ))
        
        # 生成函数指针目标样本
        samples.append(TrainingSample(
            task="function_pointer_target",
            instruction=f"分析 {framework} 结构体中 .{callback} 字段指向哪个函数",
            input=code,
            output=f".{callback} 字段指向 my_{callback} 函数。\n\n这是在结构体初始化时通过 .{callback} = my_{callback} 赋值的。",
            metadata={"framework": framework, "callback": callback}
        ))
        
        return samples
    
    def _generate_framework_code(self, framework: str, callback: str) -> str:
        """生成框架示例代码"""
        if framework == "usb_driver":
            return f'''static int my_{callback}(struct usb_interface *intf, const struct usb_device_id *id)
{{
    printk("my_{callback} called\\n");
    return 0;
}}

static struct usb_driver my_driver = {{
    .name = "my_usb_driver",
    .{callback} = my_{callback},
    .id_table = my_id_table,
}};

module_usb_driver(my_driver);'''
        
        elif framework == "platform_driver":
            return f'''static int my_{callback}(struct platform_device *pdev)
{{
    printk("my_{callback} called\\n");
    return 0;
}}

static struct platform_driver my_driver = {{
    .driver = {{
        .name = "my_platform_driver",
    }},
    .{callback} = my_{callback},
}};

module_platform_driver(my_driver);'''
        
        elif framework == "file_operations":
            return f'''static int my_{callback}(struct inode *inode, struct file *filp)
{{
    printk("my_{callback} called\\n");
    return 0;
}}

static struct file_operations my_fops = {{
    .owner = THIS_MODULE,
    .{callback} = my_{callback},
}};'''
        
        return ""
    
    def _format_callback_answer(self, framework: str, callback: str, info: dict) -> str:
        """格式化回调触发时机答案"""
        answer = f"my_{callback} 函数的触发时机：\n\n"
        answer += f"**触发条件**：{info['trigger']}\n\n"
        answer += f"**执行上下文**：{info['context']}\n\n"
        
        if info['call_chain']:
            answer += "**调用链**：\n"
            for i, func in enumerate(info['call_chain']):
                prefix = "└── " if i == len(info['call_chain']) - 1 else "├── "
                answer += f"{'    ' * i}{prefix}{func}\n"
        
        return answer
    
    def _generate_async_samples(self, pattern: str, info: dict) -> List[TrainingSample]:
        samples = []
        
        code = self._generate_async_code(pattern, info)
        
        samples.append(TrainingSample(
            task="async_pattern",
            instruction="识别以下代码中的异步模式，并解释执行流程",
            input=code,
            output=self._format_async_answer(pattern, info),
            metadata={"pattern": pattern}
        ))
        
        return samples
    
    def _generate_async_code(self, pattern: str, info: dict) -> str:
        """生成异步模式示例代码"""
        if pattern == "workqueue":
            return '''static void my_work_handler(struct work_struct *work)
{
    struct my_device *dev = container_of(work, struct my_device, work);
    // 耗时操作
    printk("Work executed\\n");
}

static int my_probe(struct platform_device *pdev)
{
    struct my_device *dev = devm_kzalloc(&pdev->dev, sizeof(*dev), GFP_KERNEL);
    INIT_WORK(&dev->work, my_work_handler);
    return 0;
}

static irqreturn_t my_irq_handler(int irq, void *dev_id)
{
    struct my_device *dev = dev_id;
    schedule_work(&dev->work);
    return IRQ_HANDLED;
}'''
        
        elif pattern == "timer":
            return '''static void my_timer_callback(struct timer_list *t)
{
    struct my_device *dev = from_timer(dev, t, timer);
    printk("Timer expired\\n");
    mod_timer(&dev->timer, jiffies + HZ);  // 重新启动
}

static int my_probe(struct platform_device *pdev)
{
    struct my_device *dev = devm_kzalloc(&pdev->dev, sizeof(*dev), GFP_KERNEL);
    timer_setup(&dev->timer, my_timer_callback, 0);
    mod_timer(&dev->timer, jiffies + HZ);  // 1秒后触发
    return 0;
}'''
        
        elif pattern == "tasklet":
            return '''static void my_tasklet_handler(struct tasklet_struct *t)
{
    struct my_device *dev = from_tasklet(dev, t, tasklet);
    printk("Tasklet executed\\n");
}

static int my_probe(struct platform_device *pdev)
{
    struct my_device *dev = devm_kzalloc(&pdev->dev, sizeof(*dev), GFP_KERNEL);
    tasklet_setup(&dev->tasklet, my_tasklet_handler);
    return 0;
}

static irqreturn_t my_irq_handler(int irq, void *dev_id)
{
    struct my_device *dev = dev_id;
    tasklet_schedule(&dev->tasklet);
    return IRQ_HANDLED;
}'''
        
        elif pattern == "irq":
            return '''static irqreturn_t my_irq_handler(int irq, void *dev_id)
{
    struct my_device *dev = dev_id;
    // 快速处理，不能睡眠！
    u32 status = readl(dev->regs + STATUS_REG);
    writel(status, dev->regs + STATUS_REG);  // 清除中断
    return IRQ_HANDLED;
}

static int my_probe(struct platform_device *pdev)
{
    struct my_device *dev = devm_kzalloc(&pdev->dev, sizeof(*dev), GFP_KERNEL);
    int irq = platform_get_irq(pdev, 0);
    devm_request_irq(&pdev->dev, irq, my_irq_handler, 0, "my_device", dev);
    return 0;
}'''
        
        return ""
    
    def _format_async_answer(self, pattern: str, info: dict) -> str:
        """格式化异步模式答案"""
        answer = f"**异步模式**：{pattern.upper()} ({info['description']})\n\n"
        
        answer += f"**执行上下文**：{info['context']}\n\n"
        
        answer += "**执行流程**：\n"
        if pattern == "workqueue":
            answer += "1. probe 中调用 INIT_WORK 绑定处理函数\n"
            answer += "2. 中断中调用 schedule_work 提交任务\n"
            answer += "3. schedule_work 立即返回，不等待执行\n"
            answer += "4. 内核 kworker 线程稍后调度执行处理函数\n"
            answer += "\n**时间线**：\n"
            answer += "中断发生 → schedule_work → 立即返回 → ... → kworker 调度 → handler 执行\n"
        elif pattern == "timer":
            answer += "1. probe 中调用 timer_setup 绑定回调函数\n"
            answer += "2. mod_timer 设置定时器到期时间\n"
            answer += "3. 定时器到期后，内核调用回调函数\n"
            answer += "\n**注意**：回调在软中断上下文，不能睡眠！\n"
        elif pattern == "tasklet":
            answer += "1. probe 中调用 tasklet_setup 绑定处理函数\n"
            answer += "2. 中断中调用 tasklet_schedule 调度执行\n"
            answer += "3. 中断返回后，软中断上下文执行 tasklet\n"
            answer += "\n**优先级**：高于 workqueue，低于硬中断\n"
        elif pattern == "irq":
            answer += "1. probe 中调用 request_irq 注册中断处理函数\n"
            answer += "2. 硬件触发中断时，CPU 调用处理函数\n"
            answer += "3. 必须快速执行，不能睡眠！\n"
        
        return answer


# ============================================================
# 来源2：Linux 内核源码分析
# ============================================================

class KernelCodeAnalyzer:
    """分析 Linux 内核源码生成训练数据"""
    
    def __init__(self, kernel_path: str):
        self.kernel_path = Path(kernel_path)
    
    def analyze_drivers(self, max_files: int = 100) -> List[TrainingSample]:
        """分析 drivers/ 目录下的代码"""
        samples = []
        
        drivers_path = self.kernel_path / "drivers"
        if not drivers_path.exists():
            print(f"Warning: {drivers_path} not found")
            return samples
        
        # 查找 C 文件
        c_files = list(drivers_path.rglob("*.c"))[:max_files]
        
        for c_file in c_files:
            try:
                samples.extend(self._analyze_file(c_file))
            except Exception as e:
                print(f"Error analyzing {c_file}: {e}")
        
        return samples
    
    def _analyze_file(self, file_path: Path) -> List[TrainingSample]:
        """分析单个文件"""
        samples = []
        
        with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
            content = f.read()
        
        # 查找 ops 结构体初始化
        ops_pattern = r'static\s+(?:const\s+)?struct\s+(\w+_operations?|\w+_ops)\s+(\w+)\s*=\s*\{([^}]+)\}'
        
        for match in re.finditer(ops_pattern, content, re.DOTALL):
            struct_type = match.group(1)
            var_name = match.group(2)
            body = match.group(3)
            
            # 提取字段赋值
            field_pattern = r'\.(\w+)\s*=\s*(\w+)'
            fields = re.findall(field_pattern, body)
            
            if fields:
                code_snippet = match.group(0)
                for field_name, func_name in fields:
                    if not func_name.startswith('__'):  # 跳过内部函数
                        samples.append(TrainingSample(
                            task="function_pointer_target",
                            instruction=f"分析 {var_name}.{field_name} 指向哪个函数",
                            input=code_snippet,
                            output=f"{var_name}.{field_name} 指向 {func_name} 函数。\n\n这是通过结构体初始化 .{field_name} = {func_name} 赋值的。",
                            metadata={"file": str(file_path), "struct": struct_type}
                        ))
        
        return samples


# ============================================================
# 来源3：LLM 辅助生成
# ============================================================

class LLMDataGenerator:
    """使用 LLM 辅助生成训练数据"""
    
    PROMPTS = {
        "function_pointer": '''生成一个 C 语言代码片段，包含函数指针的使用，以及对应的分析问答。

要求：
1. 代码要真实、有意义
2. 包含函数指针的定义、赋值、调用
3. 答案要详细解释函数指针指向谁

输出 JSON 格式：
{
  "code": "...",
  "question": "...",
  "answer": "..."
}''',

        "async_pattern": '''生成一个 Linux 内核异步编程的代码片段，以及对应的分析问答。

异步模式可以是：workqueue, timer, tasklet, completion, waitqueue

要求：
1. 代码要符合内核编程规范
2. 展示完整的绑定和触发过程
3. 答案要解释执行流程和时间线

输出 JSON 格式：
{
  "code": "...",
  "pattern": "workqueue|timer|tasklet|...",
  "question": "...",
  "answer": "..."
}'''
    }
    
    def generate_samples(self, count: int = 100) -> List[TrainingSample]:
        """生成训练样本（需要配置 API）"""
        print("LLM 生成需要配置 OpenAI/Claude API")
        print("请手动运行生成脚本或使用其他数据源")
        return []


# ============================================================
# 数据合并和导出
# ============================================================

class DataManager:
    """管理训练数据"""
    
    def __init__(self, output_dir: str = "data"):
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(exist_ok=True, parents=True)
        self.samples: List[TrainingSample] = []
    
    def add_samples(self, samples: List[TrainingSample]):
        """添加样本"""
        self.samples.extend(samples)
        print(f"Added {len(samples)} samples, total: {len(self.samples)}")
    
    def save(self, filename: str = "train.jsonl"):
        """保存为 JSONL 格式"""
        output_path = self.output_dir / filename
        
        with open(output_path, 'w', encoding='utf-8') as f:
            for sample in self.samples:
                # 转换为训练格式
                train_item = {
                    "instruction": sample.instruction,
                    "input": sample.input,
                    "output": sample.output,
                }
                f.write(json.dumps(train_item, ensure_ascii=False) + '\n')
        
        print(f"Saved {len(self.samples)} samples to {output_path}")
    
    def save_by_task(self):
        """按任务类型分别保存"""
        task_samples = {}
        for sample in self.samples:
            task = sample.task
            if task not in task_samples:
                task_samples[task] = []
            task_samples[task].append(sample)
        
        for task, samples in task_samples.items():
            filename = f"{task}.jsonl"
            output_path = self.output_dir / filename
            
            with open(output_path, 'w', encoding='utf-8') as f:
                for sample in samples:
                    f.write(json.dumps(sample.to_dict(), ensure_ascii=False) + '\n')
            
            print(f"Saved {len(samples)} {task} samples to {output_path}")
    
    def statistics(self):
        """打印统计信息"""
        print("\n" + "=" * 50)
        print("数据集统计")
        print("=" * 50)
        
        task_counts = {}
        for sample in self.samples:
            task = sample.task
            task_counts[task] = task_counts.get(task, 0) + 1
        
        for task, count in sorted(task_counts.items()):
            print(f"  {task}: {count}")
        
        print(f"\n  总计: {len(self.samples)} 样本")
        print("=" * 50)


# ============================================================
# 主程序
# ============================================================

def main():
    parser = argparse.ArgumentParser(description="FlowSight 训练数据准备工具")
    parser.add_argument("--source", choices=["knowledge", "kernel", "llm", "all"],
                        default="knowledge", help="数据来源")
    parser.add_argument("--kernel-path", type=str, help="Linux 内核源码路径")
    parser.add_argument("--output", type=str, default="data", help="输出目录")
    parser.add_argument("--merge", action="store_true", help="合并所有数据")
    
    args = parser.parse_args()
    
    manager = DataManager(args.output)
    
    # 来源1：知识库
    if args.source in ["knowledge", "all"]:
        print("\n[1/3] 从知识库生成数据...")
        converter = KnowledgeBaseConverter()
        samples = converter.generate_samples()
        manager.add_samples(samples)
    
    # 来源2：内核源码
    if args.source in ["kernel", "all"] and args.kernel_path:
        print("\n[2/3] 分析内核源码...")
        analyzer = KernelCodeAnalyzer(args.kernel_path)
        samples = analyzer.analyze_drivers()
        manager.add_samples(samples)
    
    # 来源3：LLM 生成
    if args.source in ["llm", "all"]:
        print("\n[3/3] LLM 辅助生成...")
        generator = LLMDataGenerator()
        samples = generator.generate_samples()
        manager.add_samples(samples)
    
    # 保存
    manager.statistics()
    manager.save()
    manager.save_by_task()
    
    print("\n完成！")
    print(f"数据保存在: {args.output}/")
    print("\n下一步:")
    print("  1. 检查生成的数据质量")
    print("  2. 人工补充更多样本")
    print("  3. 运行训练脚本")


if __name__ == "__main__":
    main()

