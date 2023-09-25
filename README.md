# ai

> **Work in progress (WIP):** Some functionalities are still missing. The API is being worked on. Windows support was implemented without testing, and the testing was done with ChatGPT 3.5 Turbo. Testing with ChatGPT 4 may yield better results.

`ai` is a command-line tool that provides ChatGPT with access to your shell. Please note that this may have security implications.

## Installation

To install `ai`, follow these steps:

```
git clone https://github.com/magmast/ai.git
cd ai
go install .
```

## Usage examples

Here are some usage examples for `ai`:

- Change the format of all `.png` files to `.webp` in the current working directory:

  ```
  ai "Change format of all .png files to .webp in my current working directory."
  ```

- Read, improve, and save the `README.md` file:

  ```
  ai "Can you read, improve, and save the ./README.md file?"
  ```

- Open `google.com` in the Brave browser:

  ```
  ai "Open google.com in brave-browser."
  ```

- Install Go language development tools:
  ```
  ai "Install golang development tools."
  ```

Feel free to explore more commands and experiment with `ai` to make your shell interactions easier and more convenient.

**Note:** Please use `ai` responsibly and exercise caution when granting shell access to external tools.
