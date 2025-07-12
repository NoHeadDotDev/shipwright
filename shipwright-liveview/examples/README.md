# 🚀 Shipwright LiveView Examples

This directory contains examples showcasing Shipwright LiveView with the **enhanced hot reload system**.

## 🔥 Enhanced Hot Reload Features

All examples now support our next-generation hot reload system with:

- ⚡ **Near-instant template updates** without Rust recompilation
- 🧠 **Intelligent change detection** (template vs code changes)
- 💾 **State preservation** during hot reloads (form data, component state)
- 🍞 **Toast notifications** for reload status
- 🔄 **Connection resilience** with automatic reconnection
- 🎯 **DOM patching** for minimal updates
- ⌨️ **Debug shortcuts** for development

## 📁 Available Examples

### 1. **Counter** - Interactive state management
```bash
cd counter
shipwright dev --open
```
**Features:** Basic reactive counter with increment/decrement buttons

### 2. **Chat** - Real-time communication
```bash
cd chat  
shipwright dev --open
```
**Features:** Live chat with message broadcasting

### 3. **Clock** - Time-based updates
```bash
cd clock
shipwright dev --open
```
**Features:** Real-time clock with automatic updates

### 4. **Form** - Form handling and validation
```bash
cd form
shipwright dev --open
```
**Features:** Form inputs with validation and state management

### 5. **Key Events** - Keyboard interaction
```bash
cd key-events
shipwright dev --open
```
**Features:** Keyboard event handling and shortcuts

## 🧪 Testing Hot Reload

### Quick Test Steps:

1. **Start any example:**
   ```bash
   cd counter  # or any example
   shipwright dev --open
   ```

2. **Open your browser** to the displayed URL (usually http://localhost:3000)

3. **Edit the template** in `src/main.rs`:
   - Change text content
   - Modify CSS styles  
   - Add new HTML elements
   - Update component structure

4. **Save the file** and watch for:
   - 🍞 Toast notification about the update
   - ⚡ Instant visual changes
   - 💾 Preserved component state

### Advanced Testing:

**Template Changes (Near-instant):**
- Change button text: `"+ Increment"` → `"➕ Add One"`
- Modify colors in CSS
- Add new paragraphs or sections

**Rust Code Changes (Full rebuild):**
- Change component logic in `update()` method
- Add new message types
- Modify data structures

### 🎮 Debug Features:

**Keyboard Shortcuts:**
- **Ctrl+Shift+R** - Force reconnect to hot reload server
- **Ctrl+Shift+H** - Show hot reload statistics in console
- **Ctrl+Shift+T** - Toggle toast notifications

**Console Commands:**
```javascript
// In browser console:
client.getStats()           // Show detailed statistics
client.getConnectionState() // Check connection status
```

## 🛠️ Development Commands

### Using Shipwright CLI (Recommended):
```bash
# Development with hot reload
shipwright dev

# Development with custom port
shipwright dev --port 3001

# Production build
shipwright build --release

# Production server
shipwright serve --release
```

### Manual Setup (Alternative):
```bash
# Terminal 1: Start hot reload server
cd ../shipwright-liveview-hotreload
cargo run --bin shipwright-hotreload -- --port 3001

# Terminal 2: Start example app
cd counter  # or any example
cargo run
```

## 📊 Performance Expectations

**Hot Reload Performance:**
- Template-only changes: **<100ms** update time
- CSS changes: **<50ms** update time  
- Full Rust rebuilds: **2-10s** depending on project size

**Features:**
- ✅ State preservation during template updates
- ✅ Form data retained during reloads
- ✅ Scroll position maintained
- ✅ Focus state preserved
- ✅ Connection auto-recovery

## 🐛 Troubleshooting

**Hot reload not working?**
1. Check that port 3001 is available
2. Verify `Shipwright.toml` configuration
3. Look for console errors in browser dev tools
4. Restart with `shipwright dev --log-level debug`

**Template changes not detected?**
1. Make sure you're editing files in `src/` directory
2. Check file permissions
3. Try force refresh with Ctrl+Shift+R

**Connection issues?**
1. Check that both servers are running
2. Verify WebSocket connection in Network tab
3. Look for firewall/proxy issues

## 🎯 What to Try

**Beginner:**
1. Change text content in templates
2. Modify CSS colors and styles
3. Add new HTML elements

**Intermediate:**
1. Add new component state
2. Modify event handlers
3. Change component structure

**Advanced:**
1. Add real-time features
2. Implement complex state management
3. Test error recovery scenarios

## 📈 Development Experience

The enhanced hot reload system provides a **Dioxus-level development experience** with:

- **Fast feedback loops** for rapid iteration
- **Preserved context** during development
- **Visual feedback** for all updates
- **Robust error handling** and recovery
- **Production-ready performance**

Enjoy building with Shipwright LiveView! 🚀