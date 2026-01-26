# FlowSight Desktop UI Testing Suite

Complete UI testing solution for Tauri desktop application, inspired by Claude Computer Use.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  Test Runner (test_runner.py)                               │
│  ├── Desktop Automation (xdotool, scrot)                    │
│  └── WebView Tester (Playwright)                            │
│                                                              │
│  MCP Server (mcp_server.py) - Optional, for programmatic    │
│  access from Claude Code or other tools                     │
└─────────────────────────────────────────────────────────────┘
```

## Prerequisites

### System Dependencies
```bash
# Install on Ubuntu/Debian
sudo apt-get update
sudo apt-get install -y scrot xdotool wmctrl imagemagick
```

### Python Dependencies
```bash
pip install playwright
playwright install chromium
```

## Quick Start

### 1. Start FlowSight
```bash
cd /home/parallels/github/flowsight/app
pnpm tauri dev
```

### 2. Run Automated Tests
```bash
cd app/tests/desktop
python test_runner.py
```

### 3. Run Interactive Mode
```bash
python test_runner.py --interactive
```

### 4. Run Playwright Tests
```bash
cd app
pnpm exec playwright test tests/desktop/playwright.spec.ts
```

## Tools

### Desktop Automation Commands
```python
# Screenshot
screenshot("name")  # Saves to /tmp/flowsight_tests/

# Click
click(x, y)         # Click at coordinates
double_click(x, y)
right_click(x, y)

# Mouse
move_mouse(x, y)
get_mouse_position()  # Returns (x, y)

# Keyboard
type_text("hello")
hotkey("ctrl+c")
hotkey("cmd+k")

# Wait
wait(seconds)
```

### Interactive Mode Commands
```
screenshot <name>  - Take screenshot
click <x> <y>      - Click at position
move <x> <y>       - Move mouse
hotkey <combo>     - Execute hotkey (e.g., ctrl+b)
type <text>        - Type text
wait <seconds>     - Wait
web <selector>     - Click web element
web_screenshot     - Screenshot WebView
pos                - Get mouse position
quit               - Exit
```

## Test Cases

| Test | Description | Status |
|------|-------------|--------|
| test_header_exists | Header renders | ✓ |
| test_sidebar_exists | Sidebar renders | ✓ |
| test_command_palette_opens | Cmd+K works | ✓ |
| test_colors_applied_correctly | Colors are #0a0a0b etc. | ✓ |
| test_sidebar_navigation_works | Nav buttons work | ✓ |
| test_right_panel_toggle | Right panel toggles | ✓ |
| test_animation_works | Animations present | ✓ |
| test_desktop_screenshot | Screenshot works | ✓ |

## MCP Server (Optional)

Start the MCP server for programmatic access:

```bash
python mcp_server.py
```

Available MCP tools:
- `screenshot` - Capture screen
- `get_screenshot_data` - Get screenshot as base64
- `click` - Click at coordinates
- `double_click` - Double click
- `type_text` - Type text
- `hotkey` - Execute hotkey
- `move_mouse` - Move mouse
- `wait` - Wait

## Screenshot Output

Screenshots are saved to:
- Desktop: `/tmp/flowsight_tests/`
- Playwright: `app/test-results/flowsight/`

## Extending Tests

### Add New Test Case

```python
def test_my_feature(self) -> TestResult:
    try:
        # Your test logic
        if success:
            return TestResult.PASS
        return TestResult.FAIL
    except Exception as e:
        return TestResult.ERROR
```

### Add New Desktop Command

```python
# In DesktopAutomation class
def new_command(self, arg: type) -> ReturnType:
    result = subprocess.run(["command", str(arg)], ...)
    return result
```

## CI Integration

```yaml
# .github/workflows/ui-tests.yml
name: UI Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install dependencies
        run: |
          sudo apt-get install -y scrot xdotool
          pip install playwright
      - name: Start Tauri app
        run: |
          cd app && pnpm tauri dev &
          sleep 10
      - name: Run tests
        run: |
          cd app/tests/desktop
          python test_runner.py
      - name: Upload screenshots
        uses: actions/upload-artifact@v4
        with:
          name: ui-screenshots
          path: /tmp/flowsight_tests/
```

## Troubleshooting

### "xdotool not found"
```bash
sudo apt-get install xdotool
```

### "scrot not found"
```bash
sudo apt-get install scrot
```

### Playwright browser not found
```bash
playwright install chromium
```

### Port 5173 already in use
```bash
pkill -f vite
pnpm tauri dev
```

## Comparison with Claude Computer Use

| Feature | Claude Computer Use | This Solution |
|---------|---------------------|---------------|
| Visual Understanding | ✓ Full AI | ✗ Basic |
| Screen Capture | ✓ Yes | ✓ Yes |
| Mouse Control | ✓ Yes | ✓ Yes |
| Keyboard Input | ✓ Yes | ✓ Yes |
| Free | ✗ Paid API | ✓ Open Source |
| Local | ✗ Cloud | ✓ Local |

This solution provides local, open-source desktop UI testing without needing Claude API access.
