# TUI Testing Framework

This document describes the comprehensive TUI testing framework for LithicRivers, which provides multiple approaches to test the Terminal User Interface components.

## Overview

The TUI testing framework provides four different testing approaches, each with different trade-offs:

1. **Advanced Mock-based Testing** - Fast, reliable, but limited realism
2. **Headless TUI Testing** - Realistic, uses asciimatics headless mode
3. **Visual Regression Testing** - Captures and compares screen outputs
4. **Improved Bash Script Testing** - Real terminal testing with tmux

## Quick Start

```bash
# Run all TUI tests
make test-tui

# Run quick tests (mock-based only)
make test-tui-quick

# Run specific test types
make test-tui-headless
make test-tui-visual
make test-tui-bash
```

## 1. Advanced Mock-based Testing

**File**: `lithicrivers/test/test_tui_advanced.py`

**Pros**:
- Fast execution (no real terminal needed)
- Reliable and deterministic
- Good for unit testing individual components
- No external dependencies

**Cons**:
- Limited realism (mocked asciimatics)
- May miss real terminal issues
- Doesn't test actual rendering

**Best for**: Unit testing, CI/CD, quick development feedback

### Usage

```python
from lithicrivers.test.test_tui_advanced import AdvancedUITestCase

class TestMyWidget(AdvancedUITestCase):
    def test_widget_rendering(self):
        widget = GameWidget(self.game)
        widget._frame = self.mock_frame
        widget.update(0)
        
        # Check rendered content
        self.assert_content_contains("Expected Text")
```

## 2. Headless TUI Testing

**File**: `lithicrivers/test/test_tui_headless.py`

**Pros**:
- Uses real asciimatics headless mode
- Tests actual rendering and event handling
- More realistic than mocks
- Good for integration testing

**Cons**:
- Slower than mock tests
- Requires asciimatics headless support
- May have platform-specific issues

**Best for**: Integration testing, rendering verification

### Usage

```python
from lithicrivers.test.test_tui_headless import HeadlessTUITestCase

class TestMyWidget(HeadlessTUITestCase):
    def test_widget_headless(self):
        screen = self.create_headless_screen()
        
        try:
            widget = GameWidget(self.game)
            widget._frame = Mock()
            widget._frame.canvas = screen.canvas
            widget.update(0)
            
            content = self.get_screen_content(screen)
            self.assertIn("Expected Text", content)
        finally:
            screen.close()
```

## 3. Visual Regression Testing

**File**: `lithicrivers/test/test_tui_visual.py`

**Pros**:
- Captures actual screen output
- Detects visual regressions
- Creates baseline screenshots
- Good for UI consistency

**Cons**:
- Requires baseline screenshots
- May be flaky with minor changes
- Larger test artifacts

**Best for**: Visual regression testing, UI consistency

### Usage

```python
from lithicrivers.test.test_tui_visual import VisualTUITestCase

class TestMyWidget(VisualTUITestCase):
    def test_widget_visual(self):
        screen = self.create_headless_screen()
        
        try:
            widget = GameWidget(self.game)
            # ... setup widget ...
            widget.update(0)
            
            content = self.capture_screen_content(screen, "my_widget")
            self.assert_screen_hash_matches(content, "my_widget")
        finally:
            screen.close()
```



## Test Categories

### Component Testing

Test individual UI components in isolation:

```python
class TestGameWidget(AdvancedUITestCase):
    def test_widget_rendering(self):
        # Test widget renders correctly
        pass
    
    def test_widget_dimensions(self):
        # Test widget calculates dimensions
        pass
    
    def test_widget_with_game_state(self):
        # Test widget with different game states
        pass
```

### Input Handling Testing

Test keyboard and mouse input handling:

```python
class TestInputHandler(AdvancedUITestCase):
    def test_movement_input(self):
        # Test WASD movement
        pass
    
    def test_mining_input(self):
        # Test spacebar mining
        pass
    
    def test_viewport_input(self):
        # Test viewport controls
        pass
```

### Integration Testing

Test complete workflows and interactions:

```python
class TestUIIntegration(AdvancedUITestCase):
    def test_complete_game_flow(self):
        # Test player movement -> mining -> viewport changes
        pass
    
    def test_ui_responsiveness(self):
        # Test UI responds to various inputs
        pass
```

### Performance Testing

Test UI performance and responsiveness:

```python
class TestUIPerformance(AdvancedUITestCase):
    def test_rendering_performance(self):
        # Test rendering speed
        pass
    
    def test_input_handling_performance(self):
        # Test input handling speed
        pass
```

## Best Practices

### 1. Choose the Right Testing Approach

- **Unit testing**: Use mock-based tests
- **Integration testing**: Use headless tests
- **Visual testing**: Use visual regression tests
- **End-to-end testing**: Use bash script tests

### 2. Test Structure

```python
class TestMyComponent(AdvancedUITestCase):
    def setUp(self):
        super().setUp()
        # Set up component-specific fixtures
    
    def test_specific_behavior(self):
        # Arrange
        # Act
        # Assert
    
    def tearDown(self):
        # Clean up if needed
        super().tearDown()
```

### 3. Assertion Patterns

```python
# Content assertions
self.assert_content_contains("Expected Text")
self.assert_content_not_contains("Unexpected Text")

# Paint call assertions
self.assert_paint_called_with("Text", x=10, y=5)

# Screen content assertions (headless)
content = self.get_screen_content(screen)
self.assertIn("Text", content)

# Visual assertions (visual tests)
self.assert_screen_hash_matches(content, "test_name")
```

### 4. Test Data Management

```python
def create_test_world(self, tiles: dict) -> None:
    """Create a test world with specified tiles."""
    for pos_str, tile in tiles.items():
        x, y, z = map(int, pos_str.split(','))
        self.game_engine.set_tile(VectorN(x, y, z), tile)
```

## Troubleshooting

### Common Issues

1. **Mock tests failing**: Check that mocks are properly configured
2. **Headless tests failing**: Ensure asciimatics supports headless mode
3. **Visual tests failing**: Regenerate baselines with `self.skipTest()`
4. **Bash tests failing**: Check tmux installation and terminal compatibility

### Debugging Tips

1. **Enable verbose output**: Use `-v` flag with unittest
2. **Check screen content**: Use `get_screen_content()` to inspect output
3. **Examine paint calls**: Use `get_paint_calls()` to see what was rendered
4. **Review screenshots**: Check `test_screenshots/` directory for visual output

### Performance Optimization

1. **Use appropriate test types**: Mock tests for fast feedback
2. **Limit test scope**: Test specific components rather than full workflows
3. **Reuse fixtures**: Set up common test data in `setUp()`
4. **Parallel execution**: Run different test types in parallel

## Continuous Integration

The TUI testing framework is designed to work well in CI environments:

```yaml
# Example GitHub Actions configuration
- name: Run TUI Tests
  run: |
    make test-tui-quick  # Fast tests for CI
    make test-tui-headless  # More comprehensive tests
```

## Future Enhancements

1. **Automated visual diffing**: Compare screenshots automatically
2. **Performance benchmarking**: Track rendering performance over time
3. **Accessibility testing**: Test keyboard navigation and screen readers
4. **Cross-platform testing**: Test on different terminal types
5. **Automated UI testing**: Generate tests from user interactions

## Conclusion

The TUI testing framework provides comprehensive coverage of the LithicRivers user interface through multiple testing approaches. Choose the appropriate testing method based on your needs:

- **Development**: Use mock-based tests for fast feedback
- **Integration**: Use headless tests for realistic scenarios
- **Visual**: Use visual regression tests for UI consistency
- **End-to-end**: Use bash script tests for complete workflows

This multi-layered approach ensures robust testing of the TUI components while maintaining good performance and reliability. 