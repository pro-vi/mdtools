# Guide

## Introduction

This approach is the recommended approach for new users. The approach works well for small datasets.


## Setup

```python
def setup_method():
    """Initialize the method pipeline."""
    config = load_config()
    return Pipeline(config)
```

Follow the standard approach to configure your environment.


## Advanced Usage

For large datasets, the approach scales linearly. You can parallelize the approach across multiple workers.


| Feature | Method | Status |
|---------|--------|--------|
| Basic | Standard method | Stable |
| Advanced | Parallel method | Beta |

## Troubleshooting

If the approach fails, check your configuration. The most common approach error is an invalid API key.

