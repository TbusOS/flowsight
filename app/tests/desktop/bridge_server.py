#!/usr/bin/env python3
"""
Python Bridge Server for @flowsight/desktop-test

This server wraps the existing Python desktop automation framework
and exposes it via JSON-RPC style communication over stdio.

Usage:
    python bridge_server.py

Communication protocol:
    Input (stdin): JSON-RPC 2.0 requests
    Output (stdout): JSON-RPC 2.0 responses

Example:
    Input:  {"jsonrpc": "2.0", "id": 1, "method": "screenshot", "params": []}
    Output: {"jsonrpc": "2.0", "id": 1, "result": "<base64_image>"}
"""

import sys
import json
import base64
import io
import logging
from typing import Any, Dict, List, Optional

# Ensure unbuffered output
sys.stdout.reconfigure(line_buffering=True)

# Set up logging to stderr
logging.basicConfig(
    level=logging.DEBUG,
    format='[%(levelname)s] %(message)s',
    stream=sys.stderr
)
logger = logging.getLogger(__name__)

# Import the existing desktop framework
try:
    from core.desktop_adapter import DesktopAdapter, Point
    from core.visual_analyzer import VisualAnalyzer
    FRAMEWORK_AVAILABLE = True
except ImportError as e:
    logger.warning(f"Desktop framework not available: {e}")
    FRAMEWORK_AVAILABLE = False


class BridgeServer:
    """JSON-RPC bridge server for desktop automation"""

    def __init__(self):
        self.adapter: Optional[DesktopAdapter] = None
        self.visual_analyzer: Optional[VisualAnalyzer] = None
        self._initialize()

    def _initialize(self):
        """Initialize the desktop adapter"""
        if FRAMEWORK_AVAILABLE:
            try:
                self.adapter = DesktopAdapter()
                logger.info(f"Desktop adapter initialized for {self.adapter.platform}")
            except Exception as e:
                logger.error(f"Failed to initialize desktop adapter: {e}")

    # =========================================================================
    # RPC Methods
    # =========================================================================

    def screenshot(self) -> str:
        """Take screenshot and return as base64"""
        if not self.adapter:
            raise RuntimeError("Desktop adapter not available")

        result = self.adapter.get_screenshot()
        
        # Convert PIL Image to base64
        buffer = io.BytesIO()
        result.image.save(buffer, format='JPEG', quality=75)
        return base64.b64encode(buffer.getvalue()).decode('utf-8')

    def click(self, x: int, y: int, button: str = 'left') -> bool:
        """Click at coordinates"""
        if not self.adapter:
            raise RuntimeError("Desktop adapter not available")

        point = Point(int(x), int(y))
        return self.adapter.click(point, button)

    def double_click(self, x: int, y: int) -> bool:
        """Double click at coordinates"""
        if not self.adapter:
            raise RuntimeError("Desktop adapter not available")

        point = Point(int(x), int(y))
        return self.adapter.double_click(point)

    def move_mouse(self, x: int, y: int) -> bool:
        """Move mouse to coordinates"""
        if not self.adapter:
            raise RuntimeError("Desktop adapter not available")

        point = Point(int(x), int(y))
        return self.adapter.move_mouse(point)

    def drag(self, from_x: int, from_y: int, to_x: int, to_y: int) -> bool:
        """Drag from one point to another"""
        if not self.adapter:
            raise RuntimeError("Desktop adapter not available")

        start = Point(int(from_x), int(from_y))
        end = Point(int(to_x), int(to_y))
        return self.adapter.drag(start, end)

    def scroll(self, x: int, y: int, delta_x: int, delta_y: int) -> bool:
        """Scroll at position"""
        if not self.adapter:
            raise RuntimeError("Desktop adapter not available")

        point = Point(int(x), int(y))
        return self.adapter.scroll(point, int(delta_x), int(delta_y))

    def type_text(self, text: str, interval: float = 0.01) -> bool:
        """Type text"""
        if not self.adapter:
            raise RuntimeError("Desktop adapter not available")

        return self.adapter.type_text(text, interval)

    def press_key(self, key: str, modifiers: Optional[List[str]] = None) -> bool:
        """Press key with optional modifiers"""
        if not self.adapter:
            raise RuntimeError("Desktop adapter not available")

        return self.adapter.press_key(key, modifiers)

    def get_accessibility_tree(self, window_id: Optional[str] = None) -> Optional[Dict]:
        """Get accessibility tree"""
        if not self.adapter:
            raise RuntimeError("Desktop adapter not available")

        tree = self.adapter.get_accessibility_tree(window_id)
        if tree:
            return self._ui_element_to_dict(tree)
        return None

    def find_application(self, app_name: str) -> Optional[Dict]:
        """Find application by name"""
        if not self.adapter:
            raise RuntimeError("Desktop adapter not available")

        app_info = self.adapter.find_application(app_name)
        if app_info:
            return {
                'pid': app_info.pid,
                'name': app_info.name,
                'bundle_id': app_info.bundle_id,
                'window_handles': app_info.window_handles,
            }
        return None

    def activate_application(self, app_name: str) -> bool:
        """Find and activate application"""
        if not self.adapter:
            raise RuntimeError("Desktop adapter not available")

        app_info = self.adapter.find_and_activate(app_name)
        return app_info is not None

    def find_element_by_name(self, name: str, element_type: Optional[str] = None) -> Optional[Dict]:
        """Find UI element by name"""
        if not self.adapter:
            raise RuntimeError("Desktop adapter not available")

        element = self.adapter.find_element_by_name(name, element_type)
        if element:
            return self._ui_element_to_dict(element)
        return None

    def get_screen_size(self) -> Dict[str, int]:
        """Get screen size"""
        if not self.adapter:
            raise RuntimeError("Desktop adapter not available")

        width, height = self.adapter.get_screen_size()
        return {'width': width, 'height': height}

    def analyze_visual(self, screenshot_base64: str) -> Optional[Dict]:
        """Analyze screenshot with visual analyzer"""
        if not self.visual_analyzer:
            # Lazy initialization
            try:
                self.visual_analyzer = VisualAnalyzer()
            except Exception as e:
                logger.error(f"Failed to initialize visual analyzer: {e}")
                raise RuntimeError(f"Visual analyzer not available: {e}")

        # Decode base64 to image
        from PIL import Image
        image_data = base64.b64decode(screenshot_base64)
        image = Image.open(io.BytesIO(image_data))

        # Analyze
        analysis = self.visual_analyzer.analyze_screenshot(image)

        return {
            'elements': [
                {
                    'element_type': e.element_type,
                    'text': e.text,
                    'bounds': e.bounds,
                    'confidence': e.confidence,
                    'attributes': e.attributes,
                }
                for e in analysis.elements
            ],
            'layout_description': analysis.layout_description,
            'color_palette': analysis.color_palette,
            'text_content': analysis.text_content,
            'issues': analysis.issues,
        }

    def shutdown(self) -> bool:
        """Shutdown the bridge"""
        return True

    # =========================================================================
    # Helpers
    # =========================================================================

    def _ui_element_to_dict(self, element) -> Dict:
        """Convert UIElement to dictionary"""
        return {
            'element_id': element.element_id,
            'element_type': element.element_type,
            'name': element.name,
            'bounds': {
                'x': element.bounds.x,
                'y': element.bounds.y,
                'width': element.bounds.width,
                'height': element.bounds.height,
            },
            'value': element.value,
            'enabled': element.enabled,
            'focused': element.focused,
            'children': [self._ui_element_to_dict(c) for c in element.children] if element.children else [],
        }

    # =========================================================================
    # Server Loop
    # =========================================================================

    def handle_request(self, request: Dict[str, Any]) -> Dict[str, Any]:
        """Handle a JSON-RPC request"""
        method = request.get('method')
        params = request.get('params', [])
        request_id = request.get('id')

        try:
            # Get method handler
            handler = getattr(self, method, None)
            if not handler or method.startswith('_'):
                raise ValueError(f"Unknown method: {method}")

            # Call handler
            if isinstance(params, list):
                result = handler(*params)
            elif isinstance(params, dict):
                result = handler(**params)
            else:
                result = handler()

            return {
                'jsonrpc': '2.0',
                'id': request_id,
                'result': result,
            }

        except Exception as e:
            logger.error(f"Error handling {method}: {e}")
            return {
                'jsonrpc': '2.0',
                'id': request_id,
                'error': {
                    'code': -32000,
                    'message': str(e),
                },
            }

    def run(self):
        """Main server loop"""
        # Send ready signal
        print(json.dumps({'status': 'ready', 'platform': self.adapter.platform if self.adapter else 'unknown'}))
        sys.stdout.flush()

        # Process requests
        for line in sys.stdin:
            line = line.strip()
            if not line:
                continue

            try:
                request = json.loads(line)
                response = self.handle_request(request)
                print(json.dumps(response))
                sys.stdout.flush()

                # Check for shutdown
                if request.get('method') == 'shutdown':
                    break

            except json.JSONDecodeError as e:
                logger.error(f"Invalid JSON: {e}")
                error_response = {
                    'jsonrpc': '2.0',
                    'id': None,
                    'error': {
                        'code': -32700,
                        'message': f'Parse error: {e}',
                    },
                }
                print(json.dumps(error_response))
                sys.stdout.flush()


def main():
    """Entry point"""
    server = BridgeServer()
    server.run()


if __name__ == '__main__':
    main()
