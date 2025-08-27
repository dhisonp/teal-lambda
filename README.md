# Teal Lambda

This project is a serverless API that acts as an AI-powered journaling companion. It uses Google's Gemini model to provide therapeutic-style responses to user's daily "tells" and tracks their emotional state over time in a DynamoDB database.

The backend is implemented as an AWS Lambda function in Rust and uses `cargo-lambda` for building, testing, and deploying.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Cargo Lambda](https://www.cargo-lambda.info/guide/installation.html)
- [AWS CLI](https://aws.amazon.com/cli/)

## Getting Started

### Build

Build the Lambda function for deployment:

```bash
cargo lambda build --release
```

### Test

Run unit tests:

```bash
cargo test
```

To test the function locally, you can start a local server:

```bash
cargo lambda watch
```

And then invoke it using `curl` or `cargo lambda invoke`.

### Deploy

Deploy the function to your AWS account:

```bash
cargo lambda deploy
```