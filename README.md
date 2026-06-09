# gitmsg

`gitmsg` suggests a concise Conventional Commit message from your staged Git diff using an OpenAI-compatible API. It can talk to OpenAI or Google AI Studio with the same chat completions flow.

## What it does

- Reads the staged diff with `git diff --cached`
- Sends the diff to an OpenAI-compatible chat completions endpoint
- Prints a suggested commit message in `type(scope): description` form
- Optionally runs `git commit -m "..."`

## Installation from source

```bash
git clone <repo-url>
cd gitmsg
cargo build --release
```

The binary is built as `target/release/gitmsg`. You can also install it locally with:

```bash
cargo install --path .
```

## Configuration

Set these environment variables before running the tool:

- `AI_PROVIDER` - optional provider selector, `openai` or `google`; defaults to auto-detect
- `OPENAI_API_KEY` - required when using OpenAI
- `OPENAI_BASE_URL` - optional custom base URL, defaults to `https://api.openai.com/v1`
- `OPENAI_MODEL` - optional model name, defaults to `gpt-5.4-mini`
- `GOOGLE_API_KEY` - required when using Google AI Studio
- `GOOGLE_BASE_URL` - optional custom base URL, defaults to `https://generativelanguage.googleapis.com/v1beta/openai`
- `GOOGLE_MODEL` - optional model name, defaults to `gemini-3.5-flash`

Example:

```bash
export OPENAI_API_KEY=your_key_here
export OPENAI_BASE_URL=https://api.openai.com/v1
export OPENAI_MODEL=gpt-5.4-mini

# or

export AI_PROVIDER=google
export GOOGLE_API_KEY=your_google_key_here
export GOOGLE_MODEL=gemini-3.5-flash
```

## Usage

Generate a suggested commit message:

```bash
git add .
gitmsg
```

Generate and commit immediately:

```bash
gitmsg --commit
```

Print debug details while running:

```bash
gitmsg --verbose
```

## Example output

```text
feat(parser): add staged diff summarization
```