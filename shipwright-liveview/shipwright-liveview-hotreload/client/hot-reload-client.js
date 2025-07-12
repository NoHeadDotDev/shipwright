/**
 * Hot Reload Client for Shipwright LiveView
 * 
 * This module connects to the hot reload server and handles template updates.
 */

export class HotReloadClient {
  constructor(url = 'ws://localhost:3001/ws') {
    this.url = url;
    this.ws = null;
    this.reconnectDelay = 1000;
    this.maxReconnectDelay = 30000;
    this.reconnectAttempts = 0;
    this.templates = new Map();
    this.updateHandlers = new Set();
  }

  /**
   * Connect to the hot reload server
   */
  connect() {
    console.log('[HotReload] Connecting to', this.url);
    
    try {
      this.ws = new WebSocket(this.url);
      
      this.ws.onopen = () => {
        console.log('[HotReload] Connected');
        this.reconnectAttempts = 0;
        this.reconnectDelay = 1000;
      };
      
      this.ws.onmessage = (event) => {
        this.handleMessage(JSON.parse(event.data));
      };
      
      this.ws.onclose = () => {
        console.log('[HotReload] Disconnected');
        this.scheduleReconnect();
      };
      
      this.ws.onerror = (error) => {
        console.error('[HotReload] WebSocket error:', error);
      };
    } catch (error) {
      console.error('[HotReload] Failed to connect:', error);
      this.scheduleReconnect();
    }
  }

  /**
   * Schedule a reconnection attempt
   */
  scheduleReconnect() {
    if (this.reconnectAttempts === 0) {
      console.log('[HotReload] Will attempt to reconnect...');
    }
    
    this.reconnectAttempts++;
    const delay = Math.min(this.reconnectDelay * this.reconnectAttempts, this.maxReconnectDelay);
    
    setTimeout(() => {
      this.connect();
    }, delay);
  }

  /**
   * Handle incoming messages from the server
   */
  handleMessage(message) {
    switch (message.type) {
      case 'connected':
        console.log('[HotReload] Server version:', message.version);
        break;
        
      case 'template_updated':
        this.handleTemplateUpdate(message);
        break;
        
      case 'batch_update':
        message.updates.forEach(update => this.handleTemplateUpdate(update));
        break;
        
      case 'error':
        console.error('[HotReload] Server error:', message.message);
        break;
        
      case 'ping':
        this.send({ type: 'pong' });
        break;
    }
  }

  /**
   * Handle a template update
   */
  handleTemplateUpdate(update) {
    console.log('[HotReload] Template updated:', update.id);
    
    // Store the update
    this.templates.set(update.hash, update);
    
    // Notify all handlers
    this.updateHandlers.forEach(handler => {
      try {
        handler(update);
      } catch (error) {
        console.error('[HotReload] Error in update handler:', error);
      }
    });
  }

  /**
   * Register a handler for template updates
   */
  onUpdate(handler) {
    this.updateHandlers.add(handler);
    
    // Return unsubscribe function
    return () => {
      this.updateHandlers.delete(handler);
    };
  }

  /**
   * Request a reload for a specific template
   */
  requestReload(templateId) {
    this.send({
      type: 'reload_request',
      template_id: templateId
    });
  }

  /**
   * Send a message to the server
   */
  send(message) {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
    }
  }

  /**
   * Disconnect from the server
   */
  disconnect() {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  /**
   * Get a cached template by hash
   */
  getTemplate(hash) {
    return this.templates.get(hash);
  }

  /**
   * Check if hot reload is available
   */
  static isAvailable() {
    return typeof WebSocket !== 'undefined' && process.env.NODE_ENV === 'development';
  }
}

/**
 * Global hot reload client instance
 */
let globalClient = null;

/**
 * Initialize the global hot reload client
 */
export function initHotReload(url) {
  if (!HotReloadClient.isAvailable()) {
    console.log('[HotReload] Not available in this environment');
    return null;
  }
  
  if (!globalClient) {
    globalClient = new HotReloadClient(url);
    globalClient.connect();
    
    // Add visual indicator
    addHotReloadIndicator();
  }
  
  return globalClient;
}

/**
 * Get the global hot reload client
 */
export function getHotReloadClient() {
  return globalClient;
}

/**
 * Add a visual indicator for hot reload status
 */
function addHotReloadIndicator() {
  const indicator = document.createElement('div');
  indicator.id = 'hot-reload-indicator';
  indicator.style.cssText = `
    position: fixed;
    bottom: 10px;
    right: 10px;
    padding: 5px 10px;
    background: #4CAF50;
    color: white;
    font-size: 12px;
    border-radius: 4px;
    z-index: 9999;
    transition: opacity 0.3s;
    opacity: 0;
  `;
  indicator.textContent = 'Hot Reload Active';
  document.body.appendChild(indicator);
  
  // Show briefly on connection
  globalClient.onUpdate(() => {
    indicator.style.opacity = '1';
    setTimeout(() => {
      indicator.style.opacity = '0';
    }, 2000);
  });
}