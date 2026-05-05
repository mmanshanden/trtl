# trtl

A simple Turtle graphics programming language interpreter, implemented in Rust and compiled to WebAssembly for web-based execution.

## Description

trtl is a programming language inspired by Logo's Turtle graphics. It allows you to write scripts that control a virtual turtle to draw graphics on a canvas. The language supports basic commands like moving forward or turning using variables or simple expressions.

The project consists of:
- A Rust-based interpreter and compiler
- WebAssembly compilation for browser execution
- A web-based editor and canvas interface built with TypeScript and Vite

A hosted version is available [here](https://mmanshanden.github.io/trtl/). This version may be behind the latest commit.

## Installation

### Prerequisites

- [Rust](https://rustup.rs/) (with wasm32 target)
- [Node.js](https://nodejs.org/) (for the web frontend)

### Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/mmanshanden/trtl.git
   cd trtl
   ```

2. Install the wasm32 target for Rust:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

3. Install web dependencies:
   ```bash
   cd web
   npm install
   cd ..
   ```

## Running

Before running the Vite dev server you must first build the wasm binary.

```bash
cargo build --target wasm32-unknown-unknown --release
```


The `.cargo/config.toml` should output the binary in the expected directory. It should now simply be a matter of starting the Vite dev server:
```bash
cd web
npm run dev
```

## Usage

1. Open the web application in your browser.
2. Select an example from the dropdown or write your own Turtle script in the editor.
3. The canvas on the right will update in real-time as you edit and run the code.
