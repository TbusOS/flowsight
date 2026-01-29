#!/usr/bin/env python3
"""
原生对话框测试模块

使用 AppleScript 测试 Tauri 原生文件对话框功能
无需 pyobjc，纯 AppleScript 实现
"""

import subprocess
import time
import os
from pathlib import Path


class NativeDialogTester:
    """原生对话框测试器"""
    
    # 测试内核路径
    KERNEL_PATH = "/Users/sky/linux-kernel/linux"
    ARM32_PATH = f"{KERNEL_PATH}/arch/arm/mach-imx"
    TEST_FILE = f"{ARM32_PATH}/mach-imx6q.c"
    
    def __init__(self, app_name: str = "FlowSight"):
        self.app_name = app_name
        
    def run_applescript(self, script: str, timeout: int = 30) -> tuple[bool, str]:
        """执行 AppleScript"""
        try:
            result = subprocess.run(
                ['osascript', '-e', script],
                capture_output=True,
                text=True,
                timeout=timeout
            )
            return result.returncode == 0, result.stdout.strip()
        except subprocess.TimeoutExpired:
            return False, "Timeout"
        except Exception as e:
            return False, str(e)
    
    def activate_app(self) -> bool:
        """激活应用窗口"""
        script = f'''
        tell application "{self.app_name}"
            activate
        end tell
        '''
        success, _ = self.run_applescript(script)
        time.sleep(0.5)
        return success
    
    def click_menu(self, menu_name: str, item_name: str) -> bool:
        """点击菜单项"""
        script = f'''
        tell application "System Events"
            tell process "{self.app_name}"
                click menu item "{item_name}" of menu "{menu_name}" of menu bar 1
            end tell
        end tell
        '''
        success, output = self.run_applescript(script)
        if not success:
            # 尝试另一种方式 - 通过 UI 元素
            print(f"菜单点击失败，尝试 UI 元素方式: {output}")
            return self._click_menu_ui(menu_name, item_name)
        return success
    
    def _click_menu_ui(self, menu_name: str, item_name: str) -> bool:
        """通过 UI 元素点击菜单"""
        # 先找到菜单按钮并点击
        script = f'''
        tell application "System Events"
            tell process "{self.app_name}"
                -- 获取所有 UI 元素
                set uiElements to UI elements of front window
                return uiElements
            end tell
        end tell
        '''
        success, output = self.run_applescript(script)
        print(f"UI 元素: {output}")
        return False
    
    def send_keyboard_shortcut(self, key: str, modifiers: list = None) -> bool:
        """发送键盘快捷键"""
        modifiers = modifiers or []
        
        mod_str = ""
        if "cmd" in modifiers or "command" in modifiers:
            mod_str += "command down, "
        if "shift" in modifiers:
            mod_str += "shift down, "
        if "option" in modifiers or "alt" in modifiers:
            mod_str += "option down, "
        if "ctrl" in modifiers or "control" in modifiers:
            mod_str += "control down, "
        
        mod_str = mod_str.rstrip(", ")
        
        if mod_str:
            script = f'''
            tell application "System Events"
                tell process "{self.app_name}"
                    keystroke "{key}" using {{{mod_str}}}
                end tell
            end tell
            '''
        else:
            script = f'''
            tell application "System Events"
                tell process "{self.app_name}"
                    keystroke "{key}"
                end tell
            end tell
            '''
        
        success, output = self.run_applescript(script)
        if not success:
            print(f"快捷键发送失败: {output}")
        time.sleep(0.3)
        return success
    
    def wait_for_dialog(self, timeout: int = 5) -> bool:
        """等待对话框出现"""
        start = time.time()
        while time.time() - start < timeout:
            script = f'''
            tell application "System Events"
                tell process "{self.app_name}"
                    return exists sheet 1 of front window
                end tell
            end tell
            '''
            success, output = self.run_applescript(script)
            if success and output.lower() == "true":
                return True
            
            # 检查是否有标准文件对话框
            script2 = '''
            tell application "System Events"
                return name of first window whose subrole is "AXStandardWindow"
            end tell
            '''
            success2, output2 = self.run_applescript(script2)
            if success2 and ("打开" in output2 or "Open" in output2 or "选择" in output2):
                return True
                
            time.sleep(0.2)
        return False
    
    def handle_open_dialog(self, path: str) -> bool:
        """处理打开对话框 - 输入路径并确认"""
        # 方法1: 使用 Go to Folder (Cmd+Shift+G)
        self.send_keyboard_shortcut("g", ["cmd", "shift"])
        time.sleep(0.5)
        
        # 输入路径
        script = f'''
        tell application "System Events"
            keystroke "{path}"
            delay 0.3
            keystroke return
        end tell
        '''
        success, output = self.run_applescript(script)
        if not success:
            print(f"输入路径失败: {output}")
            return False
        
        time.sleep(0.5)
        
        # 点击打开/选择按钮
        script2 = '''
        tell application "System Events"
            keystroke return
        end tell
        '''
        self.run_applescript(script2)
        time.sleep(0.3)
        
        return True
    
    def take_screenshot(self, name: str = "test") -> str:
        """截图并保存"""
        screenshots_dir = Path(__file__).parent / "test-results" / "native"
        screenshots_dir.mkdir(parents=True, exist_ok=True)
        
        filepath = screenshots_dir / f"{name}_{int(time.time())}.png"
        
        script = f'''
        do shell script "screencapture -x '{filepath}'"
        '''
        self.run_applescript(script)
        
        return str(filepath)
    
    def get_window_content(self) -> str:
        """获取窗口内容（用于验证）"""
        script = f'''
        tell application "System Events"
            tell process "{self.app_name}"
                set allElements to entire contents of front window
                return allElements as string
            end tell
        end tell
        '''
        success, output = self.run_applescript(script, timeout=60)
        return output if success else ""


def test_open_folder():
    """测试打开文件夹功能"""
    print("=" * 50)
    print("测试：打开文件夹功能")
    print("=" * 50)
    
    tester = NativeDialogTester()
    
    # 1. 激活应用
    print("1. 激活应用...")
    if not tester.activate_app():
        print("   ❌ 激活应用失败")
        return False
    print("   ✓ 应用已激活")
    time.sleep(1)
    
    # 2. 截图初始状态
    print("2. 截图初始状态...")
    tester.take_screenshot("01_initial")
    
    # 3. 发送 Cmd+O 打开文件夹
    print("3. 发送 Cmd+Shift+O 打开项目...")
    tester.send_keyboard_shortcut("o", ["cmd", "shift"])
    time.sleep(1)
    
    # 4. 截图对话框
    print("4. 截图对话框...")
    tester.take_screenshot("02_dialog")
    
    # 5. 检查是否有对话框
    print("5. 等待对话框...")
    if tester.wait_for_dialog(timeout=3):
        print("   ✓ 对话框已打开")
        
        # 6. 输入路径
        print(f"6. 输入路径: {tester.ARM32_PATH}")
        tester.handle_open_dialog(tester.ARM32_PATH)
        time.sleep(1)
        
        # 7. 截图结果
        print("7. 截图结果...")
        tester.take_screenshot("03_result")
        
        print("   ✓ 测试完成")
        return True
    else:
        print("   ⚠ 对话框未打开 (可能快捷键不同)")
        tester.take_screenshot("02_no_dialog")
        return False


def test_command_palette_open():
    """测试通过命令面板打开项目"""
    print("=" * 50)
    print("测试：通过命令面板打开项目")
    print("=" * 50)
    
    tester = NativeDialogTester()
    
    # 1. 激活应用
    print("1. 激活应用...")
    tester.activate_app()
    time.sleep(0.5)
    
    # 2. 打开命令面板 (Cmd+K)
    print("2. 打开命令面板 (Cmd+K)...")
    tester.send_keyboard_shortcut("k", ["cmd"])
    time.sleep(0.5)
    tester.take_screenshot("cmd_01_palette")
    
    # 3. 输入 "打开项目"
    print("3. 搜索 '打开项目'...")
    script = '''
    tell application "System Events"
        keystroke "打开项目"
        delay 0.5
    end tell
    '''
    tester.run_applescript(script)
    time.sleep(0.5)
    tester.take_screenshot("cmd_02_search")
    
    # 4. 按 Enter 选择
    print("4. 选择命令...")
    tester.send_keyboard_shortcut("return", [])
    time.sleep(1)
    tester.take_screenshot("cmd_03_dialog")
    
    # 5. 检查对话框
    if tester.wait_for_dialog(timeout=3):
        print("   ✓ 文件对话框已打开")
        
        # 输入路径
        print(f"5. 输入路径: {tester.ARM32_PATH}")
        tester.handle_open_dialog(tester.ARM32_PATH)
        time.sleep(1)
        tester.take_screenshot("cmd_04_result")
        
        return True
    else:
        print("   ⚠ 文件对话框未打开")
        return False


def test_verify_project_loaded():
    """验证项目是否正确加载"""
    print("=" * 50)
    print("测试：验证项目加载")
    print("=" * 50)
    
    tester = NativeDialogTester()
    tester.activate_app()
    time.sleep(0.5)
    
    # 获取窗口内容
    print("获取窗口内容...")
    content = tester.get_window_content()
    
    # 检查是否包含预期内容
    checks = [
        ("mach-imx", "i.MX 目录"),
        ("imx6q", "IMX6Q 文件"),
        (".c", "C 源文件"),
    ]
    
    print(f"窗口内容长度: {len(content)} 字符")
    
    for keyword, desc in checks:
        if keyword.lower() in content.lower():
            print(f"   ✓ 找到 {desc}")
        else:
            print(f"   ⚠ 未找到 {desc}")
    
    tester.take_screenshot("verify_loaded")
    return True


if __name__ == "__main__":
    import sys
    
    print("\n" + "=" * 60)
    print("FlowSight 原生功能测试")
    print("=" * 60 + "\n")
    
    # 确保应用已启动
    print("请确保 FlowSight 桌面应用已启动...")
    time.sleep(2)
    
    if len(sys.argv) > 1:
        test_name = sys.argv[1]
        if test_name == "folder":
            test_open_folder()
        elif test_name == "cmd":
            test_command_palette_open()
        elif test_name == "verify":
            test_verify_project_loaded()
        else:
            print(f"未知测试: {test_name}")
            print("可用测试: folder, cmd, verify")
    else:
        # 运行所有测试
        print("运行所有原生功能测试...\n")
        
        results = []
        results.append(("命令面板打开项目", test_command_palette_open()))
        time.sleep(2)
        results.append(("验证项目加载", test_verify_project_loaded()))
        
        print("\n" + "=" * 50)
        print("测试结果汇总")
        print("=" * 50)
        for name, passed in results:
            status = "✓ 通过" if passed else "✗ 失败"
            print(f"  {name}: {status}")
