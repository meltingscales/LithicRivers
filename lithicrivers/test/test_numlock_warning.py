#!/usr/bin/env python3

import unittest
import subprocess
from unittest.mock import Mock, patch
from lithicrivers.ui import _detect_numlock_issue, get_numlock_state


class TestNumlockWarning(unittest.TestCase):
    """Test the numlock detection functionality."""

    def create_keyboard_event(self, key_code: int):
        """Create a mock keyboard event with the given key code."""
        event = Mock()
        event.key_code = key_code
        return event

    def test_numlock_off_detection(self):
        """Test that numlock off is correctly detected."""
        # Test numlock off key codes (arrow keys, home, end, page up/down)
        numlock_off_codes = [-204, -205, -206, -207, -208, -209, -210, -211]
        
        for key_code in numlock_off_codes:
            with self.subTest(f"key_code={key_code}"):
                event = self.create_keyboard_event(key_code)
                result = _detect_numlock_issue(event)
                self.assertTrue(result, f"Expected numlock issue for key code {key_code}")

    def test_numlock_on_detection(self):
        """Test that numlock on is correctly detected (no issue)."""
        # Test normal numpad key codes (when numlock is on)
        numlock_on_codes = [49, 50, 51, 52, 53, 54, 55, 56, 57]  # 1-9
        
        for key_code in numlock_on_codes:
            with self.subTest(f"key_code={key_code}"):
                event = self.create_keyboard_event(key_code)
                result = _detect_numlock_issue(event)
                self.assertFalse(result, f"Expected no numlock issue for key code {key_code}")

    def test_other_keys_detection(self):
        """Test that other keys don't trigger numlock detection."""
        # Test some other common keys
        other_keys = [ord('a'), ord('1'), ord(' '), 27, 13]  # a, 1, space, escape, enter
        
        for key_code in other_keys:
            with self.subTest(f"key_code={key_code}"):
                event = self.create_keyboard_event(key_code)
                result = _detect_numlock_issue(event)
                self.assertFalse(result, f"Expected no numlock issue for key code {key_code}")

    @patch('platform.system')
    def test_get_numlock_state_windows(self, mock_system):
        """Test numlock state detection on Windows."""
        mock_system.return_value = "Windows"
        
        # Mock the Windows API call
        with patch('lithicrivers.ui._get_numlock_state_windows') as mock_windows_func:
            # Test NumLock ON
            mock_windows_func.return_value = True
            result = get_numlock_state()
            self.assertTrue(result)
            
            # Test NumLock OFF
            mock_windows_func.return_value = False
            result = get_numlock_state()
            self.assertFalse(result)

    @patch('platform.system')
    def test_get_numlock_state_linux(self, mock_system):
        """Test numlock state detection on Linux."""
        mock_system.return_value = "Linux"
        
        # Mock the Linux-specific function directly
        with patch('lithicrivers.ui._get_numlock_state_linux') as mock_linux_func:
            # Test NumLock ON
            mock_linux_func.return_value = True
            result = get_numlock_state()
            self.assertTrue(result)
            
            # Test NumLock OFF
            mock_linux_func.return_value = False
            result = get_numlock_state()
            self.assertFalse(result)

    @patch('platform.system')
    def test_get_numlock_state_linux_headless(self, mock_system):
        """Test numlock state detection on Linux in headless environment."""
        mock_system.return_value = "Linux"
        
        # Mock headless environment (no DISPLAY)
        with patch.dict('os.environ', {}, clear=True):
            # Mock subprocess.run to fail (simulating headless)
            with patch('subprocess.run', side_effect=subprocess.TimeoutExpired("xset", 1)):
                result = get_numlock_state()
                self.assertTrue(result)  # Should assume numlock is on in headless

    @patch('platform.system')
    def test_get_numlock_state_linux_setleds_fallback(self, mock_system):
        """Test numlock state detection on Linux with setleds fallback."""
        mock_system.return_value = "Linux"
        
        # Mock xset to fail, setleds to work
        def mock_run_side_effect(cmd, *args, **kwargs):
            if cmd == ["xset", "q"]:
                raise subprocess.TimeoutExpired("xset", 1)
            elif cmd == ["setleds", "-L"]:
                mock_result = Mock()
                mock_result.returncode = 0
                mock_result.stdout = "Num Lock: on"
                return mock_result
            else:
                raise FileNotFoundError("Command not found")
        
        with patch('subprocess.run', side_effect=mock_run_side_effect):
            result = get_numlock_state()
            self.assertTrue(result)

    @patch('platform.system')
    def test_get_numlock_state_other_platform(self, mock_system):
        """Test numlock state detection on other platforms."""
        mock_system.return_value = "Darwin"  # macOS
        
        result = get_numlock_state()
        self.assertTrue(result)  # Should assume numlock is on

    @patch('platform.system')
    def test_get_numlock_state_linux_xset_variable_spacing(self, mock_system):
        """Test numlock state detection on Linux with variable spacing in xset output."""
        mock_system.return_value = "Linux"
        
        # Mock subprocess.run for Linux xset command with different spacing patterns
        with patch('subprocess.run') as mock_run:
            # Test NumLock ON with multiple spaces
            mock_result = Mock()
            mock_result.returncode = 0
            mock_result.stdout = "Num Lock:   on"  # Multiple spaces
            mock_run.return_value = mock_result
            
            result = get_numlock_state()
            self.assertTrue(result)
            
            # Test NumLock OFF with single space
            mock_result.stdout = "Num Lock: off"  # Single space
            result = get_numlock_state()
            self.assertFalse(result)
            
            # Test NumLock ON with no space
            mock_result.stdout = "Num Lock:on"  # No space
            result = get_numlock_state()
            self.assertTrue(result)
            
            # Test NumLock OFF with tabs
            mock_result.stdout = "Num Lock:\toff"  # Tab character
            result = get_numlock_state()
            self.assertFalse(result)

    @patch('platform.system')
    def test_get_numlock_state_exception_handling(self, mock_system):
        """Test that exceptions in numlock detection are handled gracefully."""
        mock_system.return_value = "Linux"
        
        # Mock subprocess.run to raise an exception
        with patch('subprocess.run', side_effect=Exception("Test exception")):
            result = get_numlock_state()
            self.assertTrue(result)  # Should default to True on exception


if __name__ == "__main__":
    unittest.main() 