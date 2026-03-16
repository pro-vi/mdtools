---
title: "Why I Switched to Rust for CLI Tools"
date: 2024-03-15
draft: false
tags: [rust, cli, tooling]
categories: [engineering]
series: "Building Developer Tools"
cover:
  image: /images/rust-cli.png
  alt: Rust CLI architecture diagram
---

# Why I Switched to Rust for CLI Tools

I've been building CLI tools for a decade. First in Python, then Go, now Rust. Here's why.

## The Problem with Scripting Languages

{{< callout type="warning" >}}
This section contains opinions. Your mileage may vary.
{{< /callout >}}

Python starts fast but ages poorly. Every `subprocess.run()` call is a liability. Dependencies rot. Virtual environments multiply.

## What Rust Gets Right

### Zero-Cost Abstractions

The `clap` derive macro gives you argument parsing that's both type-safe and zero-overhead:

```rust
#[derive(Parser)]
struct Cli {
    #[arg(long)]
    json: bool,
}
```

### Single Binary Distribution

No runtime, no dependencies, no `pip install` dance. Just `cargo install` and done.

{{< figure src="/images/binary-size.png" caption="Binary sizes across languages" >}}

## The Trade-offs

| Aspect | Python | Go | Rust |
|--------|--------|----|------|
| Compile time | N/A | Fast | Slow |
| Binary size | Large | Medium | Small |
| Safety | Runtime | Compile | Compile |

It's not all roses — compile times are genuinely painful.

## Conclusion

If you're building tools that other tools will call, Rust is the right choice. The startup latency alone justifies the switch.

*Originally published on [my blog](https://example.com/blog/rust-cli).*
