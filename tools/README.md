# CLI Tools

A command-line tool for validating, formatting, and maintaining this repository.

# Development Setup

Configure `rust-analyzer` to use it from repository root dir. 

```json
# .vscode/settings.json
{
    "rust-analyzer.linkedProjects": [
        "tools/Cargo.toml"
    ]
}
```

### Running the CLI tools

```sh
cd tools
cargo run -- ../ --fmt
```
