# Contributing to hawk 🦅

Thank you for your interest in contributing to hawk! We welcome contributions from everyone, whether you're fixing a bug, adding a feature, improving documentation, or suggesting enhancements.

## 🤝 Ways to Contribute

- 🐛 **Bug Reports**: Found an issue? Let us know!
- ✨ **Feature Requests**: Have an idea for improvement?
- 🔧 **Code Contributions**: Bug fixes, new features, optimizations
- 📚 **Documentation**: Improve README, add examples, write tutorials
- 🧪 **Testing**: Add test cases, improve test coverage
- 💡 **Examples**: Real-world use cases and sample datasets

## 🚀 Getting Started

### Development Setup

1. **Fork the repository**
   ```bash
   git clone https://github.com/kyotalab/hawk.git
   cd hawk
   ```

2. **Install Rust** (if not already installed)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

3. **Build the project**
   ```bash
   cargo build
   ```

4. **Run tests**
   ```bash
   cargo test
   ```

5. **Try it out**
   ```bash
   cargo run -- '.users[0].name' sample-data/users.json
   ```

### Development Workflow

1. **Create a branch** for your changes
   ```bash
   git checkout -b feature/amazing-feature
   ```

2. **Make your changes** with clear, focused commits

3. **Test thoroughly**
   ```bash
   cargo test
   cargo clippy  # Lint check
   cargo fmt     # Format code
   ```

4. **Submit a Pull Request** with a clear description

## 🐛 Reporting Bugs

When reporting bugs, please include:

### Bug Report Template
```
**Describe the bug**
A clear description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:
1. Run command '...'
2. With data file '...'
3. See error

**Expected behavior**
What you expected to happen.

**Actual behavior**
What actually happened.

**Environment**
- OS: [e.g., Ubuntu 22.04, macOS 13.0, Windows 11]
- Rust version: [e.g., 1.70.0]
- hawk version: [e.g., 0.1.0]

**Sample data (if applicable)**
Minimal example that reproduces the issue.

**Additional context**
Any other relevant information.
```

## ✨ Feature Requests

We love new ideas! When suggesting features:

### Feature Request Template
```
**Feature Summary**
Brief description of the feature.

**Motivation**
Why would this feature be useful? What problem does it solve?

**Proposed Solution**
How should this feature work?

**Example Usage**
Show how users would interact with this feature:
hawk '.data | new_feature(.field)' data.json

**Alternatives Considered**
Are there other ways to solve this problem?

**Additional Context**
Any other relevant information, mockups, or examples.
```

## 💻 Code Contributions

### Coding Standards

- **Follow Rust conventions**: Use `cargo fmt` and `cargo clippy`
- **Write tests**: All new features should include tests
- **Document public APIs**: Add doc comments for public functions
- **Keep it simple**: Prefer readable code over clever code
- **Follow existing patterns**: Match the existing codebase style

### Code Organization

```
src/
├── main.rs          # Entry point
├── lib.rs           # Library root
├── cli.rs           # Command line interface
├── error.rs         # Error types
├── setup.rs         # File reading & format detection
├── parser.rs        # Query parsing
├── executor.rs      # Query execution
├── filter.rs        # Filtering & aggregation
├── output.rs        # Output formatting
└── utils.rs         # Utility functions
```

### Adding New Features

1. **Start with tests**: Write tests for your feature first
2. **Implement incrementally**: Break large features into smaller chunks
3. **Update documentation**: Add examples and update README if needed
4. **Consider backwards compatibility**: Don't break existing queries

### Example: Adding a New Aggregation Function

```rust
// 1. Add to apply_pipeline_operation in filter.rs
} else if operation.starts_with("median(") && operation.ends_with(")") {
    let field = &operation[7..operation.len()-1];
    let field_name = field.trim_start_matches('.');

    if is_grouped_data(&data) {
        apply_aggregation_to_groups(data, "median", field_name)
    } else {
        calculate_median_simple(data, field_name)
    }

// 2. Implement the calculation function
fn calculate_median_simple(data: Vec<Value>, field_name: &str) -> Result<Vec<Value>, Error> {
    // Implementation here
}

// 3. Add group support in apply_aggregation_to_groups
"median" => calculate_median(items, field_name)?,

// 4. Write tests
#[test]
fn test_median_calculation() {
    // Test cases here
}
```

## 🧪 Testing Guidelines

### Running Tests
```bash
cargo test                     # All tests
cargo test test_name          # Specific test
cargo test --test integration # Integration tests only
```

### Test Categories

1. **Unit Tests**: Test individual functions
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_parse_simple_query() {
           // Test implementation
       }
   }
   ```

2. **Integration Tests**: Test complete workflows
   ```rust
   // tests/integration_test.rs
   #[test]
   fn test_csv_groupby_workflow() {
       // End-to-end test
   }
   ```

3. **Example Tests**: Verify README examples work
   ```rust
   #[test]
   fn test_readme_examples() {
       // Test examples from documentation
   }
   ```

### Adding Test Data

Place test files in `tests/data/`:
```
tests/
├── data/
│   ├── users.json
│   ├── sales.csv
│   └── config.yaml
└── integration_test.rs
```

## 📚 Documentation Guidelines

### Code Documentation
```rust
/// Calculates the median value for a numeric field
///
/// # Arguments
/// * `data` - Vector of JSON values to process
/// * `field_name` - Name of the field to calculate median for
///
/// # Examples
/// ```
/// let result = calculate_median(data, "price")?;
/// ```
pub fn calculate_median(data: Vec<Value>, field_name: &str) -> Result<Value, Error> {
    // Implementation
}
```

### README Updates
- Add new features to the feature list
- Include usage examples
- Update the comparison table if needed
- Add real-world use cases

## 🎯 Priority Areas

We're especially interested in contributions in these areas:

### High Priority
- 🐛 **Bug fixes**: Any correctness issues
- 🚀 **Performance improvements**: Memory usage, speed optimizations
- 📊 **New aggregation functions**: `median`, `stddev`, `percentile`
- 🔧 **CSV improvements**: Better type detection, delimiter handling

### Medium Priority
- 🌐 **Output formats**: XML, TSV support
- 🔍 **Query enhancements**: Regular expressions, string functions
- 📈 **Visualization**: ASCII charts, histograms
- 🔄 **Streaming**: Large file support

### Lower Priority
- 🎨 **UI improvements**: Colors, better formatting
- 📦 **Packaging**: Homebrew, APT packages
- 🔌 **Plugins**: Extensibility system

## 📋 Pull Request Guidelines

### Before Submitting
- [ ] All tests pass (`cargo test`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Documentation updated if needed
- [ ] Examples work as expected

### PR Description Template
```
## Summary
Brief description of changes

## Motivation
Why is this change needed?

## Changes
- [ ] Feature A added
- [ ] Bug B fixed
- [ ] Tests updated

## Testing
How was this tested?

## Breaking Changes
Any backwards incompatible changes?

## Related Issues
Fixes #123
```

## 🌟 Recognition

Contributors will be recognized in:
- README acknowledgments
- Release notes
- GitHub contributors page

## 📞 Getting Help

- 💬 **Discussions**: Use GitHub Discussions for questions
- 🐛 **Issues**: Use GitHub Issues for bugs and feature requests
- 📧 **Email**: Contact maintainers for sensitive issues

## 📜 Code of Conduct

We follow the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct). Please be respectful and inclusive in all interactions.

## 🙏 Thank You!

Every contribution helps make hawk better for everyone. Whether it's a typo fix or a major feature, we appreciate your effort!

---

Happy contributing! 🦅