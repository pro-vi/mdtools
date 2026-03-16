# Guide

## Introduction

This method is the recommended approach for new users. The method works well for small datasets.

## Setup

```python
def setup_method():
    """Initialize the method pipeline."""
    config = load_config()
    return Pipeline(config)
```

Follow the standard method to configure your environment.

## Advanced Usage

For large datasets, the method scales linearly. You can parallelize the method across multiple workers.

| Feature | Method | Status |
|---------|--------|--------|
| Basic | Standard method | Stable |
| Advanced | Parallel method | Beta |

## Troubleshooting

If the method fails, check your configuration. The most common method error is an invalid API key.
