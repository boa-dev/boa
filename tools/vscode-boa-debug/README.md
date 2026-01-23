# Boa DAP Test Extension

**A minimal VS Code extension for developing and testing Boa's Debug Adapter Protocol (DAP) server implementation.**

## Purpose

This is NOT a production extension for end users. This is a **development tool** that provides:

1. **Minimal glue code** to connect VS Code to `boa-cli --dap`
2. **Reproducible test environment** for DAP server development
3. **Quick iteration cycle** - modify DAP server, test immediately in VS Code

Think of it as a test harness that lets you use VS Code as your DAP client while developing the Boa debugger.

## Quick Start (2 Minutes)

### 1. Build Boa CLI with DAP Support

```bash
cd /path/to/boa
cargo build --package boa_cli
```

This creates `target/debug/boa[.exe]` with DAP support.

### 2. Open Extension in VS Code

```bash
cd tools/vscode-boa-debug
code .
```

### 3. Launch Extension Development Host

Press **F5** in VS Code. This:
- Starts a new VS Code window titled "Extension Development Host"
- Loads the extension in that window
- Shows extension logs in Debug Console (original window)

### 4. Test the DAP Server

In the **Extension Development Host** window:

1. **File → Open Folder** → Select `test-files/` subdirectory
2. Open any `.js` file (e.g., `basic.js`)
3. Press **F5** to start debugging
4. Should pause at `debugger;` statement

**That's it!** You're now testing the Boa DAP server through VS Code.

## How It Works

### Extension Code (`extension.js`)

Minimal code (~150 lines) that:

```javascript
// 1. Register 'boa' debug type
vscode.debug.registerDebugAdapterDescriptorFactory('boa', {
    createDebugAdapterDescriptor(session) {
        // 2. Find boa-cli executable
        const boaPath = findBoaCli();
        
        // 3. Launch: boa-cli --dap
        return new vscode.DebugAdapterExecutable(boaPath, ['--dap']);
    }
});
```

**That's the entire extension.** Just connects VS Code to `boa-cli --dap` via stdio.

### Finding boa-cli

Extension automatically searches:
1. `../../target/debug/boa[.exe]` (debug build)
2. `../../target/release/boa[.exe]` (release build)
3. System PATH

No configuration needed.

### DAP Protocol Flow

```
VS Code (this extension)
    ↓ launches
boa-cli --dap
    ↓ implements
DAP Server (cli/src/debug/dap.rs)
    ↓ controls
Boa Debugger (core/engine/src/debugger/)
    ↓ hooks into
Boa VM
```

## Testing Workflow

### Typical Development Cycle

1. **Modify DAP server code** in `cli/src/debug/dap.rs` or `core/engine/src/debugger/`
2. **Rebuild**: `cargo build --package boa_cli`
3. **Restart debugging** in Extension Development Host (Ctrl+Shift+F5)
4. **Test changes** immediately

No need to restart the Extension Development Host - just rebuild and restart the debug session.

## Test Files

Located in `test-files/` subdirectory:

### basic.js - Minimal Test
```javascript
console.log("Starting...");
debugger; // Should pause here
console.log("Resumed");
```

**Tests**: Basic pause/resume, `debugger;` statement

### factorial.js - Recursion
```javascript
function factorial(n) {
    if (n <= 1) return 1;
    debugger;
    return n * factorial(n - 1);
}
console.log(factorial(5));
```

**Tests**: Call stack, recursive frames, stepping

### exception.js - Error Handling
```javascript
try {
    throw new Error("Test error");
} catch (e) {
    console.log("Caught:", e.message);
}
```

**Tests**: Exception hooks, error handling

### closures.js - Scoping
```javascript
function makeCounter() {
    let count = 0;
    return function() {
        debugger;
        return ++count;
    };
}
const counter = makeCounter();
counter(); counter();
```

**Tests**: Variable scoping, closures, environment access

## Debugging the Extension Itself

If something goes wrong with the extension (not the DAP server):

### Check Extension Activation

In the **original VS Code window** (not Extension Development Host):

1. **View → Output** → Select "Extension Host"
2. Look for:
   ```
   [BOA EXTENSION] 🚀 Activation starting...
   [BOA EXTENSION] ✅ Extension activated successfully!
   ```

### Check DAP Server Launch

In the **Extension Development Host** window:

1. **Help → Toggle Developer Tools → Console**
2. Look for:
   ```
   [Boa Debug] Found boa-cli at: Q:\RsWs\boa\target\debug\boa.exe
   [Boa Debug] Starting debug session with args: ["--dap"]
   ```

### Check DAP Communication

In **Debug Console** (Extension Development Host):
```
Content-Length: 123

{"seq":1,"type":"request","command":"initialize",...}
```

You should see DAP messages flowing back and forth.

## Common Issues

### "boa-cli not found"

**Problem**: Extension can't find the executable.

**Solution**:
```bash
# Build it
cargo build --package boa_cli

# Verify it exists
Test-Path target\debug\boa.exe  # Windows
ls target/debug/boa             # Linux/Mac
```

### Extension doesn't activate

**Problem**: Extension not loaded in Extension Development Host.

**Solution**:
1. Check `package.json` has correct `activationEvents`
2. Restart Extension Development Host (close window, press F5 again)
3. Check for errors in Output → Extension Host

### Debug session starts then immediately ends

**Problem**: DAP server crashed or failed to start.

**Solution**:
1. Test manually: `.\target\debug\boa.exe --dap`
2. Should print: `[DAP] Starting Boa Debug Adapter`
3. Check for Rust panics/errors
4. Look at Debug Console for crash output

### Breakpoints don't work

**Status**: This is expected - VM integration incomplete.

**Workaround**: Use `debugger;` statements for now.

**Why**: The extension works fine. The issue is in the DAP server implementation - breakpoint checking not fully integrated with VM execution loop. See [ROADMAP.MD](../../core/engine/src/debugger/ROADMAP.MD#21-pc-based-breakpoint-checking) for details.

## Extension Structure

```
vscode-boa-debug/
├── package.json          # Extension manifest (debug type registration)
├── extension.js          # Extension code (~150 lines)
├── README.md            # This file
└── test-files/          # JavaScript test cases
    ├── basic.js
    ├── factorial.js
    ├── exception.js
    └── closures.js
```

### package.json Key Parts

```json
{
  "name": "boa-debugger",
  "contributes": {
    "debuggers": [{
      "type": "boa",
      "label": "Boa Debug",
      "program": "./extension.js"
    }]
  },
  "activationEvents": [
    "onDebug"
  ]
}
```

### extension.js Key Functions

- `activate()` - Registers debug adapter factory
- `findBoaCli()` - Searches for boa executable
- `createDebugAdapterDescriptor()` - Launches `boa-cli --dap`

## What This Extension Does NOT Do

- ❌ Implement DAP protocol (that's in `cli/src/debug/dap.rs`)
- ❌ Handle breakpoints (that's in `core/engine/src/debugger/`)
- ❌ Execute JavaScript (that's the Boa VM)
- ❌ Provide end-user features (this is a dev tool)

**All it does**: Launch `boa-cli --dap` and connect VS Code to it.

## For DAP Server Development

When implementing DAP commands:

1. **Implement command handler** in `cli/src/debug/dap.rs`
2. **Rebuild**: `cargo build --package boa_cli`
3. **Test here**: Restart debug session, try the feature
4. **Check logs**: Debug Console shows DAP messages
5. **Iterate**: Repeat until working

### Enable Debug Logging

```bash
# PowerShell
$env:BOA_DAP_DEBUG=1
cargo build --package boa_cli

# Bash
BOA_DAP_DEBUG=1 cargo build --package boa_cli
```

Then restart debugging to see verbose DAP logs.

## Launch Configuration

The extension uses this default configuration:

```json
{
  "type": "boa",
  "request": "launch",
  "name": "Debug JavaScript",
  "program": "${file}"
}
```

You can create `.vscode/launch.json` in `test-files/` to customize:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "boa",
      "request": "launch",
      "name": "Custom Config",
      "program": "${file}",
      "stopOnEntry": false,
      "trace": true
    }
  ]
}
```

## Related Documentation

**DAP Server Implementation**:
- [DAP Server Code](../../cli/src/debug/dap.rs) - The actual DAP implementation
- [Debugger Core](../../core/engine/src/debugger/) - Core debugging functionality

**Development Guides**:
- [README.MD](../../core/engine/src/debugger/README.MD) - Architecture & design
- [ROADMAP.MD](../../core/engine/src/debugger/ROADMAP.MD) - Implementation plan

## Summary

This extension is a **test harness**, not a product. It's intentionally minimal to:

1. Reduce maintenance burden
2. Keep focus on DAP server implementation
3. Provide fast iteration cycle for development

All the real work happens in:
- `cli/src/debug/dap.rs` - DAP protocol implementation
- `core/engine/src/debugger/` - Debugger core

This extension just connects them to VS Code for testing.

---

**Version**: 0.1.0 (Development Tool)  
**Last Updated**: January 2026

```bash
# 1. Build Boa CLI (requires cmake)
cargo build --package boa_cli --release

# 2. Open extension in VS Code
cd tools/vscode-boa-debug
code .

# 3. Press F5 to launch Extension Development Host

# 4. In new window: Open test-files/basic.js and press F5
```

## Prerequisites

- **Visual Studio Code** 1.70.0 or higher
- **Rust toolchain** (to build Boa)
- **cmake** (required by aws-lc-sys dependency)

### Installing cmake

**Ubuntu/Debian:**
```bash
sudo apt-get install cmake
```

**macOS:**
```bash
brew install cmake
```

**Windows:**
- Download from https://cmake.org/download/
- Or: `choco install cmake`

## Installation

### Method 1: Extension Development Host (Recommended for Testing)

1. **Build Boa CLI**:
   ```bash
   cd /path/to/boa
   cargo build --package boa_cli --release
   ```

2. **Open extension folder**:
   ```bash
   cd tools/vscode-boa-debug
   code .
   ```

3. **Press F5** to launch Extension Development Host

4. **In new window**:
   - Open folder: `test-files/`
   - Open file: `basic.js`
   - Press F5 to start debugging

### Method 2: Install from VSIX (For Distribution)

1. **Package extension**:
   ```bash
   npm install -g @vscode/vsce
   cd tools/vscode-boa-debug
   vsce package
   ```

2. **Install in VS Code**:
   - Extensions → "..." menu → Install from VSIX
   - Select `boa-debugger-0.1.0.vsix`

## Usage

### Basic Debugging

1. **Open JavaScript file** in VS Code
2. **Set breakpoint** by clicking in gutter (or use `debugger;` statement)
3. **Press F5** or Run → Start Debugging
4. **Select "Boa Debug"** (first time only)

### Launch Configuration

Create `.vscode/launch.json`:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "boa",
      "request": "launch",
      "name": "Debug with Boa",
      "program": "${file}",
      "stopOnEntry": false
    }
  ]
}
```

### Configuration Options

| Option | Type | Description | Default |
|--------|------|-------------|---------|
| `program` | string | JavaScript file to debug | `${file}` |
| `stopOnEntry` | boolean | Pause at entry | `false` |
| `args` | array | Command line arguments | `[]` |
| `cwd` | string | Working directory | `${workspaceFolder}` |
| `trace` | boolean | Enable verbose logging | `false` |
| `useHttp` | boolean | Use HTTP transport | `false` |
| `httpPort` | number | HTTP port for DAP | `4711` |

### Available Variables

- `${file}` - Currently open file
- `${workspaceFolder}` - Workspace root
- `${fileBasename}` - Current filename
- `${fileDirname}` - Current file directory

## DAP Transport Modes

### Stdio Mode (Default)

Uses stdin/stdout for DAP communication. Standard for VS Code debugging.

```bash
boa --dap
```

```json
{
  "type": "boa",
  "request": "launch",
  "program": "${file}"
}
```

### HTTP Mode

Runs HTTP server on specified port. Useful for web debugging or testing.

```bash
boa --dap --dap-http-port 4711
```

```json
{
  "type": "boa",
  "request": "launch",
  "program": "${file}",
  "useHttp": true,
  "httpPort": 4711
}
```

**Test HTTP mode**:
```bash
curl -X POST http://127.0.0.1:4711 \
  -H "Content-Type: application/json" \
  -d '{"type":"request","seq":1,"command":"initialize","arguments":{}}'
```

### Debug Logging

Enable verbose logging:

```bash
# PowerShell
$env:BOA_DAP_DEBUG=1
boa --dap

# Bash
BOA_DAP_DEBUG=1 boa --dap
```

## Testing

### Test Files

Sample files in `test-files/`:

**basic.js** - Basic debugging
```javascript
console.log("Starting...");
debugger; // Pauses here
console.log("Resumed");
```

**factorial.js** - Recursion testing
```javascript
function factorial(n) {
    if (n <= 1) return 1;
    debugger; // Set breakpoint here
    return n * factorial(n - 1);
}
```

**exception.js** - Exception handling
```javascript
try {
    throw new Error("Test error");
} catch (e) {
    debugger; // Pauses on exception
}
```

### Testing Checklist

#### ✅ Basic Functionality
- [ ] Debugger statement pauses execution
- [ ] Step over (F10) works
- [ ] Continue (F5) resumes
- [ ] Variables panel shows (may be placeholder)
- [ ] Call stack displays current function

#### ✅ Recursion
- [ ] Step into (F11) follows recursive calls
- [ ] Call stack grows with recursion
- [ ] Can step out (Shift+F11) from nested calls

#### ✅ Exceptions
- [ ] Enable "Pause on Exceptions"
- [ ] Pauses when exception thrown
- [ ] Shows exception details

## Debugging the Extension

If the extension isn't working, follow these steps:

### Step 1: Launch Extension Development Host

1. Open `tools/vscode-boa-debug` in VS Code
2. Press **F5** (starts Extension Development Host)
3. New window opens with title "Extension Development Host"

### Step 2: Trigger Activation

In the **new window**:
1. Open folder: `test-files/`
2. Open file: `basic.js`
3. Press **F5** to start debugging

### Step 3: Check Extension Activation

In Extension Development Host:
- **Help → Toggle Developer Tools → Console**
- Look for:
  ```
  [BOA EXTENSION] 🚀 Activation starting...
  [BOA EXTENSION] ✅ Extension activated successfully!
  ```

### Step 4: Verify DAP Communication

Check for these messages:
```
[Boa Debug] Creating debug adapter for session
[Boa Debug] Found boa-cli at: Q:\RsWs\boa\target\debug\boa.exe
[DAP] Starting Boa Debug Adapter
```

### Common Issues

#### "Couldn't find a debug adapter descriptor"

**Cause**: Extension not activated

**Fix**:
1. Check Extensions view → "Boa JavaScript Debugger" is enabled
2. Check Developer Console for activation errors
3. Verify `package.json` has correct `activationEvents`

#### "boa-cli not found"

**Cause**: Executable not built or not in PATH

**Fix**:
1. Build: `cargo build --package boa_cli`
2. Verify exists: `Test-Path target\debug\boa.exe`
3. Extension looks in:
   - `target/debug/boa[.exe]`
   - `target/release/boa[.exe]`
   - System PATH

#### Debug session starts but immediately ends

**Cause**: DAP protocol issue

**Fix**:
1. Test manually: `.\target\debug\boa.exe --dap`
2. Should print: `[DAP] Starting Boa Debug Adapter`
3. Check Debug Console for errors

#### Breakpoints don't hit

**Status**: Known limitation - breakpoint checking in VM not fully integrated

**Workaround**: Use `debugger;` statements
```javascript
function test() {
    debugger; // Will pause here
    console.log("test");
}
```

#### Variables show "Not yet implemented"

**Status**: Known limitation - requires `DebuggerFrame::eval()` implementation

**Progress**: See [debugger implementation status](../../core/engine/src/debugger/README.MD)

## Implementation Status

### ✅ Fully Working
- Extension activation and registration
- DAP protocol communication (stdio/HTTP)
- `debugger;` statement pauses execution
- Step commands (in/over/out)
- Exception hook called on errors
- Process lifecycle management

### ⚠️ Partially Working
- Variable inspection (returns placeholders)
- Call stack display (basic info only)
- Breakpoints (DAP messages sent, VM checking incomplete)
- Expression evaluation (limited)

### ❌ Not Yet Implemented
- Line-based breakpoints (needs line-to-PC mapping)
- Frame enter/exit hooks (needs deeper VM integration)
- Full variable inspection (needs eval implementation)
- Conditional breakpoints via DAP
- Watch expressions
- Hot reload

See [main debugger README](../../core/engine/src/debugger/README.MD) for complete status.

## Architecture

### Communication Flow

```
VS Code UI
    ↕ DAP Protocol (JSON-RPC)
extension.js (this extension)
    ↕ stdio/HTTP
boa-cli --dap (DAP Server)
    ↕ Internal API
Boa Debugger (core/engine/src/debugger/)
    ↕ Hooks
Boa VM (JavaScript execution)
```

### Components

**VS Code Extension** (`extension.js`)
- Registers 'boa' debug type
- Launches `boa-cli --dap`
- Manages debug sessions

**DAP Server** (`cli/src/debug/dap.rs`)
- Implements DAP protocol
- Handles requests: initialize, launch, setBreakpoints, threads, etc.
- Uses Content-Length framing

**Debugger Core** (`core/engine/src/debugger/`)
- Breakpoint management
- Execution control (pause/resume/step)
- Event hooks
- Reflection objects

**VM Integration** (`core/engine/src/vm/`)
- Calls debugger hooks during execution
- Checks breakpoints before instructions
- Pauses when requested

## File Structure

```
vscode-boa-debug/
├── package.json           # Extension manifest
├── extension.js           # Extension code
├── README.md             # This file
├── test-files/           # Sample JavaScript files
│   ├── basic.js
│   ├── factorial.js
│   ├── exception.js
│   └── closures.js
└── docs_archive/         # Historical documentation
    ├── CHANGELOG.md
    ├── SETUP.md
    ├── QUICKSTART.md
    ├── TESTING.md
    ├── DEBUGGING_EXTENSION.md
    └── DAP_TRANSPORT.md
```

## Development

### Extension Development Workflow

1. **Modify** `extension.js`
2. **Press F5** to reload Extension Development Host
3. **Test** changes in new window
4. **Check** Developer Console for logs

### Debugging Extension Code

Set breakpoints in `extension.js`:
- `activate()` - Extension loads
- `createDebugAdapterDescriptor()` - Debug session starts
- `findBoaCli()` - Finding executable
- `resolveDebugConfiguration()` - Resolving config

Press F5, then start debugging in Extension Development Host to hit breakpoints.

### Viewing Logs

**Extension logs** (original window):
- View → Output → "Extension Host"

**DAP logs** (Extension Development Host):
- Help → Toggle Developer Tools → Console
- Look for `[BOA EXTENSION]` and `[Boa Debug]` messages

**Debug Console** (Extension Development Host):
- View → Debug Console
- Shows DAP protocol messages

### Enable Trace Logging

```json
{
  "type": "boa",
  "request": "launch",
  "program": "${file}",
  "trace": true
}
```

## Version History

### [0.1.0] - January 2026

**Added:**
- Initial release of Boa JavaScript Debugger
- DAP protocol support (stdio and HTTP modes)
- Basic debugging features:
  - `debugger;` statement support ✅
  - Step in/over/out commands
  - Exception breakpoints (hook level)
  - Call stack inspection (basic)
  - Variable inspection (placeholder)
- VS Code extension with launch configurations
- Test files for validation

**Known Issues:**
- Breakpoint checking not fully integrated (use `debugger;`)
- Variable inspection incomplete (needs eval implementation)
- Frame enter/exit hooks not called
- Line-to-PC mapping not implemented

**Requirements:**
- Boa CLI with DAP support
- cmake (for aws-lc-sys dependency)

### Future Plans

**[0.2.0] - Planned:**
- Complete breakpoint VM integration
- Full variable inspection with eval
- Watch expressions
- Conditional breakpoints in DAP
- Hot reload support

**[0.3.0] - Planned:**
- Multi-context debugging
- Remote debugging support
- Performance profiling integration
- Source map support (TypeScript)

## Contributing

This extension is part of the Boa JavaScript engine project.

**Main Repository**: https://github.com/boa-dev/boa

**Related Documentation**:
- [Debugger Implementation](../../core/engine/src/debugger/README.MD)
- [Development Roadmap](../../core/engine/src/debugger/ROADMAP.MD)

**File Locations**:
- Debugger Core: `core/engine/src/debugger/`
- DAP Server: `cli/src/debug/dap.rs`
- This Extension: `tools/vscode-boa-debug/`

### How to Contribute

1. Check [implementation status](../../core/engine/src/debugger/README.MD#implementation-status)
2. Pick a feature from the [roadmap](../../core/engine/src/debugger/ROADMAP.MD)
3. Implement and test
4. Submit pull request to Boa repository

## Resources

### Documentation
- [DAP Specification](https://microsoft.github.io/debug-adapter-protocol/)
- [VS Code Debug API](https://code.visualstudio.com/api/extension-guides/debugger-extension)
- [Boa Documentation](https://docs.rs/boa_engine/)

### Support
- [Boa Discord](https://discord.gg/tUFFk9Y)
- [GitHub Issues](https://github.com/boa-dev/boa/issues)

## License

This extension is part of the Boa project and is dual-licensed under:
- **MIT License**
- **Apache License 2.0**

Choose whichever works best for your use case.

---

**Status**: Active Development  
**Version**: 0.1.0  
**Last Updated**: January 2026
