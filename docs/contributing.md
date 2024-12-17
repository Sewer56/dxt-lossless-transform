# Contribution Guidelines

First off, thank you for considering contributing to dxt-lossless-transform.

If your contribution is not straightforward, please first discuss the change you
wish to make by creating a new issue before making the change. We might be able to discuss
general design, etc. before you embark on a huge endeavour.

## Reporting Issues

Before reporting an issue on the
[issue tracker](https://github.com/Sewer56/dxt-lossless-transform/issues),
please check that it has not already been reported by searching for some related
keywords.

## Pull Requests

Try to do one pull request per change.  

### Commit Names

Reloaded repositories auto-generate changelogs based on commit names. 

When you make git commits; try to stick to the style of [Keep a changelog](https://keepachangelog.com/en/1.0.0/):

- `Added` for new features.  
- `Changed` for changes in existing functionality.  
- `Deprecated` for soon-to-be removed features.  
- `Removed` for now removed features.  
- `Fixed` for any bug fixes.  
- `Security` in case of vulnerabilities.  

### Code Style

Please use the standard code style `cargo fmt`, and run the `clippy` linter 
(`cargo clippy`), fixing warnings before submitting PRs.

If you are using VSCode, this should be automated (on Save) per this repository's settings.