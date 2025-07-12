# Shipwright LiveView Client

A lightweight TypeScript client for Axum LiveView with efficient binary protocol support.

## Features

- **Ultra-lightweight**: ~6.90KB gzipped bundle size
- **Binary Protocol**: Custom MessagePack implementation for efficient data transfer
- **WebSocket Connection**: Auto-reconnect, heartbeat, and connection management
- **Efficient DOM Patching**: Minimal DOM updates using diff/patch algorithm
- **Event Delegation**: Smart event handling with debounce/throttle support
- **Form State Recovery**: Preserves form input during DOM updates
- **Client Commands**: Phoenix.LiveView.JS-like client-side commands
- **TypeScript**: Full type safety and modern development experience

## Installation

```bash
npm install shipwright-liveview-client
```

## Usage

### Basic Setup

```html
<!-- Auto-initialization via data attributes -->
<div data-liveview-url="ws://localhost:3000/live" data-liveview-token="abc123">
  <!-- LiveView content will be rendered here -->
</div>
```

### Programmatic Usage

```typescript
import { LiveView } from 'shipwright-liveview-client'

const liveView = new LiveView({
  url: 'ws://localhost:3000/live',
  container: '#app',
  token: 'abc123',
  onConnect: () => console.log('Connected'),
  onDisconnect: () => console.log('Disconnected'),
  onError: (error) => console.error('Error:', error)
})

liveView.connect()
```

### Event Binding

Use `lv-*` attributes to bind events:

```html
<!-- Basic event binding -->
<button lv-click="increment">+</button>

<!-- Event with modifiers -->
<input lv-keyup="search:debounce-300:prevent" />

<!-- Form submission -->
<form lv-submit="save:prevent">
  <input name="title" />
  <button type="submit">Save</button>
</form>
```

### Available Event Modifiers

- `prevent` - Calls `preventDefault()`
- `stop` - Calls `stopPropagation()`
- `debounce-{ms}` - Debounces the event
- `throttle-{ms}` - Throttles the event

### Sending Custom Events

```typescript
liveView.pushEvent('custom_event', { data: 'value' })
```

## Binary Protocol

The client uses a compact binary protocol based on MessagePack for efficient communication:

### Message Types

- **Connect** (0x01): Initial connection handshake
- **Event** (0x02): User interaction events
- **Heartbeat** (0x03): Keep-alive messages
- **Render** (0x10): Full page render
- **Diff** (0x11): Incremental DOM updates
- **Command** (0x13): Client-side commands
- **Redirect** (0x12): Navigation commands
- **Error** (0x14): Server error messages

### Client Commands

Similar to Phoenix.LiveView.JS, supports:

- `show` / `hide` / `toggle` - Element visibility
- `add_class` / `remove_class` - CSS class manipulation
- `set_attribute` / `remove_attribute` - Attribute management
- `focus` / `blur` - Focus management
- `dispatch` - Custom events
- Transitions with CSS animations

## Advanced Configuration

```typescript
const liveView = new LiveView({
  url: 'ws://localhost:3000/live',
  container: document.getElementById('app'),
  token: 'abc123',
  reconnectInterval: 1000,     // Reconnect delay (ms)
  maxReconnectAttempts: 10,    // Max reconnection attempts
  heartbeatInterval: 30000,    // Heartbeat interval (ms)
  onConnect: () => {
    console.log('LiveView connected')
  },
  onDisconnect: () => {
    console.log('LiveView disconnected')
  },
  onError: (error) => {
    console.error('LiveView error:', error)
  }
})
```

## Development

```bash
# Install dependencies
npm install

# Development server
npm run dev

# Build for production
npm run build

# Preview build
npm run preview
```

## Bundle Analysis

The client is optimized for minimal bundle size:

- Custom MessagePack implementation (vs full library)
- Tree-shaking friendly exports
- Minimal external dependencies
- Efficient DOM operations
- Compressed with Terser

Current bundle sizes:
- ES Module: ~6.90KB gzipped
- UMD: ~5.25KB gzipped

## Browser Support

- Chrome/Edge 88+
- Firefox 87+
- Safari 14+
- Any browser with ES2020 support

## License

MIT