# CPU Performance Testing Guide

This document describes how to use the CPU profiling system for LithicRivers.

## Overview

The performance testing system focuses on **CPU profiling of the running game** using speedscope format for detailed analysis.

## Quick Start

### CPU Profiling with Speedscope

```bash
# Start the game in one terminal
make run

# In another terminal, generate a speedscope profile
make profile-speedscope
```

## Available Commands

### CPU Profiling Commands

| Command | Description |
|---------|-------------|
| `make profile-speedscope` | Generate speedscope CPU profiling report |

## Understanding the Results

### Speedscope Reports

Speedscope is a web-based profiling tool that provides:

- **Interactive Timeline**: View CPU usage over time
- **Function Analysis**: See which functions consume CPU
- **Call Stack Visualization**: Understand function call relationships
- **Performance Metrics**: Detailed timing information

### How to View Speedscope Reports

1. **Generate the report**: `make profile-speedscope`
2. **Open the file**: The report is saved as `profile_report.speedscope`
3. **View online**: Upload to https://www.speedscope.app/
4. **Or use local viewer**: Open in a web browser

### Performance Thresholds

The system can help identify:

- **High CPU Usage**: Functions using >50% CPU
- **Frequent Calls**: Functions called very frequently
- **Long Execution**: Functions taking a long time
- **Bottlenecks**: Functions blocking other operations

## Usage Examples

### Basic Profiling

```bash
# Terminal 1: Start the game
make run

# Terminal 2: Generate speedscope profile
make profile-speedscope
```

### Profiling Specific Activities

1. **Start the game**
2. **Perform the activity you want to profile** (e.g., moving around, building)
3. **Run profiling during the activity**
4. **Analyze the results in speedscope**

## Performance Optimization Tips

### Based on Profiling Results

1. **High CPU Functions**: Look for functions using >30% CPU
2. **Frequent Calls**: Optimize functions called >1000 times/second
3. **Long Execution**: Break down functions taking >100ms
4. **I/O Operations**: Look for blocking operations

### Common Optimizations

1. **Caching**: Cache expensive calculations
2. **Lazy Loading**: Load data only when needed
3. **Algorithm Changes**: Use more efficient algorithms
4. **Data Structures**: Choose appropriate data structures

## Troubleshooting

### Common Issues

1. **Permission Errors**: Some profiling requires elevated permissions
2. **Process Not Found**: Make sure game is running before profiling
3. **No Data**: Game may not be doing enough work to profile

### Performance Bottlenecks

1. **World Generation**: Most common bottleneck
2. **Rendering**: Large viewports can slow down
3. **Game Logic**: Complex game rules can be expensive
4. **I/O Operations**: File/network operations can block

## Tools and Dependencies

### Required Tools

- **py-spy**: CPU profiling tool

### Installation

All tools are automatically installed with:

```bash
make install
```

## Best Practices

1. **Profile During Normal Play**: Profile while doing typical game activities
2. **Multiple Sessions**: Profile different game states (exploring, building, etc.)
3. **Baseline First**: Establish performance baselines
4. **Measure Impact**: Test optimizations with profiling
5. **Focus on Hotspots**: Optimize the functions using most CPU

## Output Files

Profiling generates:

- `profile_report.speedscope`: Speedscope format CPU profile

## Advanced Usage

### Custom Profiling Duration

```bash
# Profile for 60 seconds instead of default 30
# Edit the Makefile to change --duration 30 to --duration 60
```

### Analyzing Specific Game Activities

1. **Start the game**
2. **Perform the activity you want to profile** (e.g., moving around, building)
3. **Run profiling during the activity**
4. **Analyze the results in speedscope**

### Comparing Performance

1. **Profile before optimization**
2. **Make your changes**
3. **Profile after optimization**
4. **Compare the results in speedscope**

## Support

For performance testing issues:

1. Check that the game is running before profiling
2. Ensure you have sufficient permissions
3. Look at the generated speedscope report for insights
4. Use the speedscope timeline to identify bottlenecks

## Future Enhancements

Planned improvements to the CPU profiling system:

- GPU profiling support
- Memory profiling integration
- Automated performance regression detection
- Performance dashboard integration
- Custom performance metrics 