/**
 * Hot Reload Client for Shipwright LiveView
 * 
 * This module connects to the hot reload server and handles template updates
 * with intelligent DOM patching, toast notifications, and state preservation.
 */

/**
 * DOM Diffing and Patching Engine
 * Provides intelligent updates with minimal DOM changes
 */
class DOMPatcher {
  constructor() {
    this.templateCache = new Map();
    this.statePreserver = null; // Will be set by parent
  }

  /**
   * Apply an intelligent patch to the DOM (legacy HTML replacement)
   */
  patch(targetSelector, newHTML, templateId) {
    const targetElement = document.querySelector(targetSelector);
    if (!targetElement) {
      console.warn(`[DOMPatcher] Target element not found: ${targetSelector}`);
      return false;
    }

    // Create a temporary container to parse the new HTML
    const tempContainer = document.createElement('div');
    tempContainer.innerHTML = newHTML;
    const newElement = tempContainer.firstElementChild;

    if (!newElement) {
      console.warn(`[DOMPatcher] Invalid HTML provided for ${templateId}`);
      return false;
    }

    // Perform intelligent diffing and patching
    this.diffAndPatch(targetElement, newElement);
    return true;
  }

  /**
   * Apply a structured patch with individual operations
   */
  applyPatch(patchData) {
    console.log('[DOMPatcher] Applying structured patch with', patchData.operations.length, 'operations');
    
    let appliedCount = 0;
    let errors = [];

    for (const operation of patchData.operations) {
      try {
        this.applyOperation(operation);
        appliedCount++;
      } catch (error) {
        console.error('[DOMPatcher] Failed to apply operation:', operation, error);
        errors.push({ operation, error: error.message });
      }
    }

    console.log(`[DOMPatcher] Applied ${appliedCount}/${patchData.operations.length} operations`);
    
    if (errors.length > 0) {
      console.warn('[DOMPatcher] Some operations failed:', errors);
    }

    return {
      success: errors.length === 0,
      appliedCount,
      errorCount: errors.length,
      errors
    };
  }

  /**
   * Apply a single patch operation
   */
  applyOperation(operation) {
    switch (operation.type) {
      case 'replace':
        this.applyReplace(operation);
        break;
      case 'update_text':
        this.applyUpdateText(operation);
        break;
      case 'set_attribute':
        this.applySetAttribute(operation);
        break;
      case 'remove_attribute':
        this.applyRemoveAttribute(operation);
        break;
      case 'insert_child':
        this.applyInsertChild(operation);
        break;
      case 'remove_child':
        this.applyRemoveChild(operation);
        break;
      case 'move_child':
        this.applyMoveChild(operation);
        break;
      default:
        throw new Error(`Unknown operation type: ${operation.type}`);
    }
  }

  /**
   * Apply a replace operation
   */
  applyReplace(operation) {
    const element = document.querySelector(operation.selector);
    if (!element) {
      throw new Error(`Element not found: ${operation.selector}`);
    }

    // Preserve focus and selection if the element or its children have focus
    const focusedElement = document.activeElement;
    const preserveFocus = element.contains(focusedElement);
    let focusData = null;

    if (preserveFocus && focusedElement) {
      focusData = {
        element: focusedElement,
        selector: this.generateSelector(focusedElement),
        selectionStart: focusedElement.selectionStart,
        selectionEnd: focusedElement.selectionEnd
      };
    }

    // Create new element
    const tempContainer = document.createElement('div');
    tempContainer.innerHTML = operation.html;
    const newElement = tempContainer.firstElementChild;

    if (!newElement) {
      throw new Error('Invalid HTML in replace operation');
    }

    // Replace the element
    element.parentNode.replaceChild(newElement, element);

    // Restore focus if needed
    if (focusData) {
      this.restoreFocus(focusData);
    }
  }

  /**
   * Apply an update text operation
   */
  applyUpdateText(operation) {
    const element = document.querySelector(operation.selector);
    if (!element) {
      throw new Error(`Element not found: ${operation.selector}`);
    }

    // For input elements, update the value
    if (element.tagName === 'INPUT' || element.tagName === 'TEXTAREA') {
      element.value = operation.text;
    } else {
      // For other elements, update text content
      element.textContent = operation.text;
    }
  }

  /**
   * Apply a set attribute operation
   */
  applySetAttribute(operation) {
    const element = document.querySelector(operation.selector);
    if (!element) {
      throw new Error(`Element not found: ${operation.selector}`);
    }

    element.setAttribute(operation.name, operation.value);
  }

  /**
   * Apply a remove attribute operation
   */
  applyRemoveAttribute(operation) {
    const element = document.querySelector(operation.selector);
    if (!element) {
      throw new Error(`Element not found: ${operation.selector}`);
    }

    element.removeAttribute(operation.name);
  }

  /**
   * Apply an insert child operation
   */
  applyInsertChild(operation) {
    const parent = document.querySelector(operation.parent_selector);
    if (!parent) {
      throw new Error(`Parent element not found: ${operation.parent_selector}`);
    }

    // Create new child element
    const tempContainer = document.createElement('div');
    tempContainer.innerHTML = operation.html;
    const newChild = tempContainer.firstElementChild;

    if (!newChild) {
      throw new Error('Invalid HTML in insert child operation');
    }

    // Insert at specified index
    const children = Array.from(parent.children);
    if (operation.index >= children.length) {
      parent.appendChild(newChild);
    } else {
      parent.insertBefore(newChild, children[operation.index]);
    }
  }

  /**
   * Apply a remove child operation
   */
  applyRemoveChild(operation) {
    const parent = document.querySelector(operation.parent_selector);
    if (!parent) {
      throw new Error(`Parent element not found: ${operation.parent_selector}`);
    }

    const children = Array.from(parent.children);
    if (operation.index < children.length) {
      parent.removeChild(children[operation.index]);
    } else {
      throw new Error(`Child index out of range: ${operation.index}`);
    }
  }

  /**
   * Apply a move child operation
   */
  applyMoveChild(operation) {
    const parent = document.querySelector(operation.parent_selector);
    if (!parent) {
      throw new Error(`Parent element not found: ${operation.parent_selector}`);
    }

    const children = Array.from(parent.children);
    if (operation.from_index >= children.length) {
      throw new Error(`Source index out of range: ${operation.from_index}`);
    }

    const childToMove = children[operation.from_index];
    
    // Remove the child
    parent.removeChild(childToMove);
    
    // Reinsert at new position
    const newChildren = Array.from(parent.children);
    if (operation.to_index >= newChildren.length) {
      parent.appendChild(childToMove);
    } else {
      parent.insertBefore(childToMove, newChildren[operation.to_index]);
    }
  }

  /**
   * Generate a reliable selector for an element
   */
  generateSelector(element) {
    // Try ID first
    if (element.id) {
      return `#${element.id}`;
    }

    // Try data attributes
    for (const attr of element.attributes) {
      if (attr.name.startsWith('data-') && attr.value) {
        return `[${attr.name}="${attr.value}"]`;
      }
    }

    // Try class combinations
    if (element.className) {
      const classes = element.className.split(' ').filter(c => c.length > 0);
      if (classes.length > 0) {
        return `.${classes.join('.')}`;
      }
    }

    // Fallback to nth-child
    if (element.parentNode) {
      const siblings = Array.from(element.parentNode.children);
      const index = siblings.indexOf(element);
      return `${element.tagName.toLowerCase()}:nth-child(${index + 1})`;
    }

    return element.tagName.toLowerCase();
  }

  /**
   * Restore focus after DOM updates
   */
  restoreFocus(focusData) {
    // Try to find the element by selector
    let targetElement = document.querySelector(focusData.selector);
    
    if (!targetElement) {
      // If selector doesn't work, try to find by similar attributes
      targetElement = this.findSimilarElement(focusData.element);
    }

    if (targetElement && targetElement.focus) {
      targetElement.focus();
      
      // Restore selection if it's a text input
      if (focusData.selectionStart !== undefined && 
          focusData.selectionEnd !== undefined &&
          targetElement.setSelectionRange) {
        try {
          targetElement.setSelectionRange(focusData.selectionStart, focusData.selectionEnd);
        } catch (e) {
          // Ignore selection errors
        }
      }
    }
  }

  /**
   * Find an element similar to the original focused element
   */
  findSimilarElement(originalElement) {
    const tagName = originalElement.tagName;
    const candidates = document.querySelectorAll(tagName);
    
    for (const candidate of candidates) {
      // Check if attributes are similar
      let similarityScore = 0;
      for (const attr of originalElement.attributes) {
        if (candidate.hasAttribute(attr.name) && 
            candidate.getAttribute(attr.name) === attr.value) {
          similarityScore++;
        }
      }
      
      // If we find a reasonable match, use it
      if (similarityScore > 0) {
        return candidate;
      }
    }
    
    return null;
  }

  /**
   * Perform intelligent DOM diffing and patching
   */
  diffAndPatch(oldNode, newNode) {
    // Handle text nodes
    if (oldNode.nodeType === Node.TEXT_NODE && newNode.nodeType === Node.TEXT_NODE) {
      if (oldNode.textContent !== newNode.textContent) {
        oldNode.textContent = newNode.textContent;
      }
      return;
    }

    // Handle element nodes
    if (oldNode.nodeType === Node.ELEMENT_NODE && newNode.nodeType === Node.ELEMENT_NODE) {
      // Update attributes
      this.updateAttributes(oldNode, newNode);

      // Handle children
      this.updateChildren(oldNode, newNode);
      return;
    }

    // If nodes are of different types, replace entirely
    if (oldNode.nodeType !== newNode.nodeType || oldNode.tagName !== newNode.tagName) {
      oldNode.parentNode?.replaceChild(newNode.cloneNode(true), oldNode);
      return;
    }
  }

  /**
   * Update element attributes
   */
  updateAttributes(oldElement, newElement) {
    // Remove old attributes that don't exist in new element
    for (const attr of oldElement.attributes) {
      if (!newElement.hasAttribute(attr.name)) {
        oldElement.removeAttribute(attr.name);
      }
    }

    // Add or update attributes from new element
    for (const attr of newElement.attributes) {
      if (oldElement.getAttribute(attr.name) !== attr.value) {
        oldElement.setAttribute(attr.name, attr.value);
      }
    }
  }

  /**
   * Update child elements
   */
  updateChildren(oldElement, newElement) {
    const oldChildren = Array.from(oldElement.childNodes);
    const newChildren = Array.from(newElement.childNodes);

    // Handle the common case of equal-length lists
    const minLength = Math.min(oldChildren.length, newChildren.length);
    for (let i = 0; i < minLength; i++) {
      this.diffAndPatch(oldChildren[i], newChildren[i]);
    }

    // Remove extra old children
    for (let i = oldChildren.length - 1; i >= minLength; i--) {
      oldElement.removeChild(oldChildren[i]);
    }

    // Add new children
    for (let i = minLength; i < newChildren.length; i++) {
      oldElement.appendChild(newChildren[i].cloneNode(true));
    }
  }

  /**
   * Find the best selector for an element
   */
  findSelector(element) {
    // Try ID first
    if (element.id) {
      return `#${element.id}`;
    }

    // Try data attributes
    if (element.dataset.templateId) {
      return `[data-template-id="${element.dataset.templateId}"]`;
    }

    // Try class combinations
    if (element.className) {
      const classes = element.className.split(' ').filter(c => c.length > 0);
      if (classes.length > 0) {
        return `.${classes.join('.')}`;
      }
    }

    // Fallback to tag name with position
    const siblings = Array.from(element.parentNode?.children || []);
    const index = siblings.indexOf(element);
    return `${element.tagName.toLowerCase()}:nth-child(${index + 1})`;
  }
}

/**
 * Toast Notification System
 * Provides non-intrusive status updates
 */
class ToastNotifier {
  constructor() {
    this.container = null;
    this.activeToasts = new Set();
    this.init();
  }

  /**
   * Initialize the toast container
   */
  init() {
    this.container = document.createElement('div');
    this.container.id = 'shipwright-toast-container';
    this.container.style.cssText = `
      position: fixed;
      top: 20px;
      right: 20px;
      z-index: 10000;
      pointer-events: none;
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    `;
    document.body.appendChild(this.container);
  }

  /**
   * Show a toast notification
   */
  show(message, type = 'info', duration = 3000) {
    const toast = document.createElement('div');
    const toastId = Date.now() + Math.random();
    toast.dataset.toastId = toastId;
    
    const colors = {
      info: { bg: '#2196F3', text: 'white' },
      success: { bg: '#4CAF50', text: 'white' },
      warning: { bg: '#FF9800', text: 'white' },
      error: { bg: '#F44336', text: 'white' }
    };

    const color = colors[type] || colors.info;
    
    toast.style.cssText = `
      background: ${color.bg};
      color: ${color.text};
      padding: 12px 16px;
      border-radius: 6px;
      margin-bottom: 8px;
      box-shadow: 0 2px 8px rgba(0,0,0,0.15);
      transform: translateX(100%);
      transition: transform 0.3s ease-out, opacity 0.3s ease-out;
      opacity: 0;
      pointer-events: auto;
      font-size: 14px;
      max-width: 300px;
      word-wrap: break-word;
    `;
    
    toast.textContent = message;
    this.container.appendChild(toast);
    this.activeToasts.add(toastId);

    // Animate in
    requestAnimationFrame(() => {
      toast.style.transform = 'translateX(0)';
      toast.style.opacity = '1';
    });

    // Auto-remove after duration
    if (duration > 0) {
      setTimeout(() => {
        this.remove(toastId);
      }, duration);
    }

    return toastId;
  }

  /**
   * Remove a toast notification
   */
  remove(toastId) {
    if (!this.activeToasts.has(toastId)) return;

    const toast = this.container.querySelector(`[data-toast-id="${toastId}"]`);
    if (toast) {
      toast.style.transform = 'translateX(100%)';
      toast.style.opacity = '0';
      
      setTimeout(() => {
        if (toast.parentNode) {
          toast.parentNode.removeChild(toast);
        }
        this.activeToasts.delete(toastId);
      }, 300);
    }
  }

  /**
   * Clear all toast notifications
   */
  clear() {
    this.activeToasts.forEach(id => this.remove(id));
  }
}

/**
 * State Preservation System
 * Preserves form data, scroll position, and focus during updates
 */
class StatePreserver {
  constructor() {
    this.formData = new Map();
    this.scrollPositions = new Map();
    this.focusedElement = null;
  }

  /**
   * Capture current page state
   */
  capture() {
    this.captureFormData();
    this.captureScrollPositions();
    this.captureFocus();
  }

  /**
   * Restore captured page state
   */
  restore() {
    // Use RAF to ensure DOM is updated
    requestAnimationFrame(() => {
      this.restoreFormData();
      this.restoreScrollPositions();
      this.restoreFocus();
    });
  }

  /**
   * Capture form data
   */
  captureFormData() {
    this.formData.clear();
    const forms = document.querySelectorAll('form');
    
    forms.forEach((form, formIndex) => {
      const formData = new FormData(form);
      const formState = {};
      
      // Capture form inputs
      const inputs = form.querySelectorAll('input, textarea, select');
      inputs.forEach((input, inputIndex) => {
        const key = input.name || input.id || `${formIndex}-${inputIndex}`;
        
        if (input.type === 'checkbox' || input.type === 'radio') {
          formState[key] = { value: input.value, checked: input.checked };
        } else {
          formState[key] = { value: input.value };
        }
      });
      
      this.formData.set(formIndex, formState);
    });
  }

  /**
   * Restore form data
   */
  restoreFormData() {
    const forms = document.querySelectorAll('form');
    
    forms.forEach((form, formIndex) => {
      const formState = this.formData.get(formIndex);
      if (!formState) return;
      
      const inputs = form.querySelectorAll('input, textarea, select');
      inputs.forEach((input, inputIndex) => {
        const key = input.name || input.id || `${formIndex}-${inputIndex}`;
        const savedState = formState[key];
        
        if (savedState) {
          input.value = savedState.value;
          if (savedState.checked !== undefined) {
            input.checked = savedState.checked;
          }
        }
      });
    });
  }

  /**
   * Capture scroll positions
   */
  captureScrollPositions() {
    this.scrollPositions.clear();
    
    // Capture window scroll
    this.scrollPositions.set('window', {
      x: window.scrollX,
      y: window.scrollY
    });
    
    // Capture scrollable elements
    const scrollableElements = document.querySelectorAll('[style*="overflow"], .scroll, .overflow-auto, .overflow-scroll');
    scrollableElements.forEach((element, index) => {
      this.scrollPositions.set(`element-${index}`, {
        x: element.scrollLeft,
        y: element.scrollTop
      });
    });
  }

  /**
   * Restore scroll positions
   */
  restoreScrollPositions() {
    // Restore window scroll
    const windowScroll = this.scrollPositions.get('window');
    if (windowScroll) {
      window.scrollTo(windowScroll.x, windowScroll.y);
    }
    
    // Restore scrollable elements
    const scrollableElements = document.querySelectorAll('[style*="overflow"], .scroll, .overflow-auto, .overflow-scroll');
    scrollableElements.forEach((element, index) => {
      const scrollData = this.scrollPositions.get(`element-${index}`);
      if (scrollData) {
        element.scrollLeft = scrollData.x;
        element.scrollTop = scrollData.y;
      }
    });
  }

  /**
   * Capture currently focused element
   */
  captureFocus() {
    this.focusedElement = document.activeElement;
    if (this.focusedElement && this.focusedElement !== document.body) {
      // Store additional focus information
      this.focusedElement._shipwrightFocusData = {
        selectionStart: this.focusedElement.selectionStart,
        selectionEnd: this.focusedElement.selectionEnd
      };
    }
  }

  /**
   * Restore focus
   */
  restoreFocus() {
    if (this.focusedElement && document.body.contains(this.focusedElement)) {
      this.focusedElement.focus();
      
      // Restore selection if it was a text input
      const focusData = this.focusedElement._shipwrightFocusData;
      if (focusData && typeof this.focusedElement.setSelectionRange === 'function') {
        this.focusedElement.setSelectionRange(focusData.selectionStart, focusData.selectionEnd);
      }
      
      // Clean up
      delete this.focusedElement._shipwrightFocusData;
    }
  }
}

class HotReloadClient {
  constructor(url = 'ws://localhost:3001/ws') {
    this.url = url;
    this.ws = null;
    this.reconnectDelay = 1000;
    this.maxReconnectDelay = 30000;
    this.reconnectAttempts = 0;
    this.templates = new Map();
    this.updateHandlers = new Set();
    
    // Enhanced features
    this.domPatcher = new DOMPatcher();
    this.toastNotifier = new ToastNotifier();
    this.statePreserver = new StatePreserver();
    this.connectionState = 'disconnected';
    this.reconnectTimeoutId = null;
    this.updateQueue = [];
    this.isUpdating = false;
  }

  /**
   * Connect to the hot reload server
   */
  connect() {
    console.log('[HotReload] Connecting to', this.url);
    this.connectionState = 'connecting';
    
    try {
      this.ws = new WebSocket(this.url);
      
      this.ws.onopen = () => {
        console.log('[HotReload] Connected');
        this.connectionState = 'connected';
        this.reconnectAttempts = 0;
        this.reconnectDelay = 1000;
        
        // Show connection success toast
        this.toastNotifier.show('Hot reload connected', 'success', 2000);
        
        // Process any queued updates
        this.processUpdateQueue();
      };
      
      this.ws.onmessage = (event) => {
        try {
          this.handleMessage(JSON.parse(event.data));
        } catch (error) {
          console.error('[HotReload] Failed to parse message:', error);
        }
      };
      
      this.ws.onclose = (event) => {
        console.log('[HotReload] Disconnected', event.code, event.reason);
        this.connectionState = 'disconnected';
        
        // Only show disconnect toast if it wasn't a clean close
        if (event.code !== 1000) {
          this.toastNotifier.show('Hot reload disconnected', 'warning', 3000);
        }
        
        this.scheduleReconnect();
      };
      
      this.ws.onerror = (error) => {
        console.error('[HotReload] WebSocket error:', error);
        this.connectionState = 'error';
        this.toastNotifier.show('Hot reload connection error', 'error', 4000);
      };
    } catch (error) {
      console.error('[HotReload] Failed to connect:', error);
      this.connectionState = 'error';
      this.toastNotifier.show('Failed to initialize hot reload', 'error', 4000);
      this.scheduleReconnect();
    }
  }

  /**
   * Schedule a reconnection attempt with exponential backoff
   */
  scheduleReconnect() {
    if (this.connectionState === 'connecting') {
      return; // Already trying to connect
    }
    
    if (this.reconnectTimeoutId) {
      clearTimeout(this.reconnectTimeoutId);
    }
    
    this.reconnectAttempts++;
    
    // Exponential backoff with jitter
    const baseDelay = this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1);
    const jitter = Math.random() * 1000; // Add up to 1 second of jitter
    const delay = Math.min(baseDelay + jitter, this.maxReconnectDelay);
    
    console.log(`[HotReload] Reconnecting in ${Math.round(delay/1000)}s (attempt ${this.reconnectAttempts})`);
    
    this.reconnectTimeoutId = setTimeout(() => {
      if (this.connectionState !== 'connected') {
        this.connect();
      }
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
        this.handleBatchUpdate(message);
        break;
        
      case 'component_update':
        this.handleComponentUpdate(message);
        break;
        
      case 'state_preservation_request':
        this.handleStatePreservationRequest(message);
        break;
        
      case 'asset_updated':
        this.handleAssetUpdate(message);
        break;
        
      case 'full_reload':
        this.handleFullReload(message);
        break;
        
      case 'error':
        console.error('[HotReload] Server error:', message.message);
        this.toastNotifier.show(`Server error: ${message.message}`, 'error', 5000);
        break;
        
      case 'ping':
        this.send({ type: 'pong' });
        break;
        
      default:
        console.warn('[HotReload] Unknown message type:', message.type);
    }
  }

  /**
   * Handle a template update with intelligent DOM patching
   */
  handleTemplateUpdate(update) {
    console.log('[HotReload] Template updated:', update.id);
    
    // Store the update
    this.templates.set(update.hash, update);
    
    // Queue the update for processing
    this.updateQueue.push({ type: 'template', update });
    this.processUpdateQueue();
  }
  
  /**
   * Handle batch updates
   */
  handleBatchUpdate(message) {
    console.log('[HotReload] Batch update received:', message.updates.length, 'updates');
    
    // Queue all updates
    message.updates.forEach(update => {
      this.templates.set(update.hash, update);
      this.updateQueue.push({ type: 'template', update });
    });
    
    this.processUpdateQueue();
  }
  
  /**
   * Handle asset updates (CSS, JS, etc.)
   */
  handleAssetUpdate(message) {
    console.log('[HotReload] Asset updated:', message.asset_type, message.path);
    
    this.updateQueue.push({ type: 'asset', update: message });
    this.processUpdateQueue();
  }
  
  /**
   * Handle full page reload
   */
  handleFullReload(message) {
    console.log('[HotReload] Full reload requested:', message.reason);
    
    this.toastNotifier.show(`Reloading page: ${message.reason}`, 'info', 2000);
    
    // Give the toast time to show before reloading
    setTimeout(() => {
      window.location.reload();
    }, 500);
  }
  
  /**
   * Process the update queue
   */
  async processUpdateQueue() {
    if (this.isUpdating || this.updateQueue.length === 0) {
      return;
    }
    
    this.isUpdating = true;
    
    try {
      // Capture current state before any updates
      this.statePreserver.capture();
      
      // Show updating toast
      const toastId = this.toastNotifier.show('Updating...', 'info', 0);
      
      let updatedCount = 0;
      
      // Process all queued updates
      while (this.updateQueue.length > 0) {
        const queuedUpdate = this.updateQueue.shift();
        
        try {
          if (queuedUpdate.type === 'template') {
            await this.applyTemplateUpdate(queuedUpdate.update);
            updatedCount++;
          } else if (queuedUpdate.type === 'asset') {
            await this.applyAssetUpdate(queuedUpdate.update);
            updatedCount++;
          } else if (queuedUpdate.type === 'component_patch') {
            await this.applyComponentPatch(
              queuedUpdate.instanceId, 
              queuedUpdate.patch, 
              queuedUpdate.preserveState
            );
            updatedCount++;
          }
        } catch (error) {
          console.error('[HotReload] Failed to apply update:', error);
          this.toastNotifier.show(`Update failed: ${error.message}`, 'error', 5000);
        }
      }
      
      // Restore state after all updates
      this.statePreserver.restore();
      
      // Remove updating toast and show success
      this.toastNotifier.remove(toastId);
      
      if (updatedCount > 0) {
        this.toastNotifier.show(`Updated ${updatedCount} component${updatedCount === 1 ? '' : 's'}`, 'success', 2000);
      }
      
      // Notify all handlers
      this.updateHandlers.forEach(handler => {
        try {
          handler({ type: 'batch_complete', count: updatedCount });
        } catch (error) {
          console.error('[HotReload] Error in update handler:', error);
        }
      });
      
    } finally {
      this.isUpdating = false;
    }
  }
  
  /**
   * Handle a component update with structured patches
   */
  handleComponentUpdate(message) {
    console.log('[HotReload] Component update received:', message.instance_id);
    
    // Queue the component update
    this.updateQueue.push({ 
      type: 'component_patch', 
      instanceId: message.instance_id,
      patch: message.patch,
      preserveState: message.preserve_state
    });
    
    this.processUpdateQueue();
  }
  
  /**
   * Handle state preservation request
   */
  handleStatePreservationRequest(message) {
    console.log('[HotReload] State preservation requested for:', message.instance_id);
    
    try {
      const state = this.captureComponentState(message.instance_id);
      
      // Send state back to server
      this.send({
        type: 'state_preservation_response',
        instance_id: message.instance_id,
        state: state,
        success: true
      });
      
    } catch (error) {
      console.error('[HotReload] Failed to capture state:', error);
      this.send({
        type: 'state_preservation_response',
        instance_id: message.instance_id,
        state: null,
        success: false
      });
    }
  }
  
  /**
   * Capture component state for preservation
   */
  captureComponentState(instanceId) {
    const component = document.querySelector(`[data-live-view-id="${instanceId}"]`);
    if (!component) {
      throw new Error(`Component not found: ${instanceId}`);
    }
    
    // Capture comprehensive state
    const state = {
      component_data: {}, // Would be populated by the LiveView system
      form_values: {},
      scroll_positions: {},
      focused_element: null,
      text_selections: {},
      user_state: null
    };
    
    // Capture form values within the component
    const forms = component.querySelectorAll('form');
    forms.forEach((form, formIndex) => {
      const inputs = form.querySelectorAll('input, textarea, select');
      inputs.forEach((input, inputIndex) => {
        const key = input.name || input.id || `${formIndex}-${inputIndex}`;
        
        if (input.type === 'checkbox' || input.type === 'radio') {
          state.form_values[key] = { value: input.value, checked: input.checked };
        } else if (input.type === 'file') {
          // Don't capture file inputs
        } else if (input.tagName === 'SELECT' && input.multiple) {
          state.form_values[key] = Array.from(input.selectedOptions).map(o => o.value);
        } else {
          state.form_values[key] = { value: input.value };
        }
        
        // Capture text selections
        if ((input.type === 'text' || input.type === 'textarea') && 
            input.selectionStart !== input.selectionEnd) {
          state.text_selections[key] = {
            start: input.selectionStart,
            end: input.selectionEnd,
            direction: input.selectionDirection || 'forward'
          };
        }
      });
    });
    
    // Capture scroll positions within the component
    const scrollableElements = component.querySelectorAll('[style*="overflow"], .scroll, .overflow-auto, .overflow-scroll');
    scrollableElements.forEach((element, index) => {
      if (element.scrollTop > 0 || element.scrollLeft > 0) {
        const selector = this.domPatcher.generateSelector(element);
        state.scroll_positions[selector] = {
          x: element.scrollLeft,
          y: element.scrollTop
        };
      }
    });
    
    // Capture focused element if it's within this component
    if (document.activeElement && component.contains(document.activeElement)) {
      state.focused_element = this.domPatcher.generateSelector(document.activeElement);
    }
    
    return state;
  }

  /**
   * Apply a template update using DOM patching
   */
  async applyTemplateUpdate(update) {
    // Try to find the element by data attribute first
    let selector = `[data-template-id="${update.id.file}:${update.id.line}:${update.id.column}"]`;
    let targetElement = document.querySelector(selector);
    
    if (!targetElement) {
      // Fallback to finding by template hash
      selector = `[data-template-hash="${update.hash}"]`;
      targetElement = document.querySelector(selector);
    }
    
    if (!targetElement) {
      console.warn('[HotReload] Could not find target element for template update:', update.id);
      return;
    }
    
    // Apply the patch
    const success = this.domPatcher.patch(selector, update.html, update.id);
    
    if (success) {
      console.log('[HotReload] Successfully patched template:', update.id);
    } else {
      console.warn('[HotReload] Failed to patch template:', update.id);
    }
  }
  
  /**
   * Apply a component patch update
   */
  async applyComponentPatch(instanceId, patch, preserveState) {
    console.log(`[HotReload] Applying component patch for ${instanceId} (preserve state: ${preserveState})`);
    
    // Find the component
    const component = document.querySelector(`[data-live-view-id="${instanceId}"]`);
    if (!component) {
      console.warn(`[HotReload] Component not found: ${instanceId}`);
      return;
    }
    
    // Preserve state if requested
    let preservedState = null;
    if (preserveState) {
      try {
        preservedState = this.captureComponentState(instanceId);
      } catch (error) {
        console.warn('[HotReload] Failed to preserve state:', error);
      }
    }
    
    // Apply the structured patch
    const result = this.domPatcher.applyPatch(patch);
    
    // Restore state if we preserved it
    if (preservedState) {
      try {
        this.restoreComponentState(instanceId, preservedState);
      } catch (error) {
        console.warn('[HotReload] Failed to restore state:', error);
      }
    }
    
    if (result.success) {
      console.log(`[HotReload] Successfully applied patch for ${instanceId}: ${result.appliedCount} operations`);
    } else {
      console.warn(`[HotReload] Patch partially failed for ${instanceId}: ${result.errorCount} errors`);
    }
    
    return result;
  }
  
  /**
   * Restore component state after update
   */
  restoreComponentState(instanceId, state) {
    const component = document.querySelector(`[data-live-view-id="${instanceId}"]`);
    if (!component) {
      throw new Error(`Component not found: ${instanceId}`);
    }
    
    // Use RAF to ensure DOM is settled
    requestAnimationFrame(() => {
      // Restore form values
      Object.entries(state.form_values).forEach(([key, valueData]) => {
        const input = component.querySelector(`[name="${key}"], #${key}`);
        if (input) {
          if (valueData.checked !== undefined) {
            input.checked = valueData.checked;
          }
          if (valueData.value !== undefined) {
            input.value = valueData.value;
          }
          if (Array.isArray(valueData)) {
            // Handle multi-select
            Array.from(input.options).forEach(option => {
              option.selected = valueData.includes(option.value);
            });
          }
        }
      });
      
      // Restore scroll positions
      Object.entries(state.scroll_positions).forEach(([selector, pos]) => {
        const element = component.querySelector(selector);
        if (element) {
          element.scrollLeft = pos.x;
          element.scrollTop = pos.y;
        }
      });
      
      // Restore focus
      if (state.focused_element) {
        const focusElement = component.querySelector(state.focused_element);
        if (focusElement && focusElement.focus) {
          focusElement.focus();
        }
      }
      
      // Restore text selections
      Object.entries(state.text_selections).forEach(([key, selection]) => {
        const input = component.querySelector(`[name="${key}"], #${key}`);
        if (input && input.setSelectionRange) {
          try {
            input.setSelectionRange(selection.start, selection.end, selection.direction);
          } catch (e) {
            // Ignore selection errors
          }
        }
      });
    });
  }
  
  /**
   * Apply an asset update (CSS, JS, etc.)
   */
  async applyAssetUpdate(update) {
    if (update.asset_type === 'css') {
      await this.reloadCSS(update.path);
    } else if (update.asset_type === 'js') {
      // For JS updates, we typically need a full reload
      this.toastNotifier.show('JavaScript updated - reloading page', 'info', 2000);
      setTimeout(() => window.location.reload(), 500);
    } else {
      console.log('[HotReload] Unknown asset type:', update.asset_type);
    }
  }
  
  /**
   * Reload CSS without full page refresh
   */
  async reloadCSS(cssPath) {
    const links = document.querySelectorAll(`link[rel="stylesheet"]`);
    
    for (const link of links) {
      if (link.href.includes(cssPath) || cssPath.includes(link.href.split('/').pop())) {
        const newLink = link.cloneNode();
        newLink.href = link.href.split('?')[0] + '?v=' + Date.now();
        
        // Wait for the new stylesheet to load
        await new Promise((resolve) => {
          newLink.onload = resolve;
          newLink.onerror = resolve;
          link.parentNode.insertBefore(newLink, link.nextSibling);
        });
        
        // Remove the old stylesheet
        link.remove();
        console.log('[HotReload] Reloaded CSS:', cssPath);
        break;
      }
    }
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
    if (this.reconnectTimeoutId) {
      clearTimeout(this.reconnectTimeoutId);
      this.reconnectTimeoutId = null;
    }
    
    if (this.ws) {
      this.ws.close(1000, 'Client disconnect');
      this.ws = null;
    }
    
    this.connectionState = 'disconnected';
    this.toastNotifier.clear();
  }

  /**
   * Get a cached template by hash
   */
  getTemplate(hash) {
    return this.templates.get(hash);
  }

  /**
   * Get current connection state
   */
  getConnectionState() {
    return this.connectionState;
  }
  
  /**
   * Check if currently connected
   */
  isConnected() {
    return this.connectionState === 'connected';
  }
  
  /**
   * Enable or disable toast notifications
   */
  setToastEnabled(enabled) {
    if (!enabled) {
      this.toastNotifier.clear();
    }
    this.toastNotifier.enabled = enabled;
  }
  
  /**
   * Get statistics about the hot reload session
   */
  getStats() {
    return {
      connectionState: this.connectionState,
      reconnectAttempts: this.reconnectAttempts,
      templatesLoaded: this.templates.size,
      updateHandlers: this.updateHandlers.size,
      queuedUpdates: this.updateQueue.length,
      isUpdating: this.isUpdating
    };
  }
  
  /**
   * Check if hot reload is available
   */
  static isAvailable() {
    return typeof WebSocket !== 'undefined' && 
           (window.location.hostname === 'localhost' || 
            window.location.hostname === '127.0.0.1' ||
            window.location.hostname === '0.0.0.0');
  }
}

/**
 * Global hot reload client instance
 */
let globalClient = null;

/**
 * Initialize the global hot reload client
 */
function initHotReload(url, options = {}) {
  if (!HotReloadClient.isAvailable()) {
    console.log('[HotReload] Not available in this environment');
    return null;
  }
  
  if (!globalClient) {
    globalClient = new HotReloadClient(url);
    
    // Apply options
    if (options.toastEnabled !== undefined) {
      globalClient.setToastEnabled(options.toastEnabled);
    }
    
    globalClient.connect();
    
    // Add enhanced visual indicator
    if (options.showIndicator !== false) {
      addHotReloadIndicator();
    }
    
    // Add keyboard shortcuts for debugging
    if (options.enableDebugShortcuts !== false) {
      addDebugShortcuts();
    }
  }
  
  return globalClient;
}

/**
 * Get the global hot reload client
 */
function getHotReloadClient() {
  return globalClient;
}

/**
 * Destroy the global hot reload client
 */
function destroyHotReload() {
  if (globalClient) {
    globalClient.disconnect();
    globalClient = null;
    
    // Remove indicator
    const indicator = document.getElementById('shipwright-hot-reload-indicator');
    if (indicator) {
      indicator.remove();
    }
    
    // Remove toast container
    const toastContainer = document.getElementById('shipwright-toast-container');
    if (toastContainer) {
      toastContainer.remove();
    }
  }
}

/**
 * Add an enhanced visual indicator for hot reload status
 */
function addHotReloadIndicator() {
  const indicator = document.createElement('div');
  indicator.id = 'shipwright-hot-reload-indicator';
  
  const updateIndicator = () => {
    const state = globalClient.getConnectionState();
    const stats = globalClient.getStats();
    
    let color, text;
    switch (state) {
      case 'connected':
        color = '#4CAF50';
        text = `🔥 Hot Reload (${stats.templatesLoaded})`;
        break;
      case 'connecting':
        color = '#FF9800';
        text = '🔄 Connecting...';
        break;
      case 'disconnected':
        color = '#9E9E9E';
        text = '❌ Disconnected';
        break;
      case 'error':
        color = '#F44336';
        text = '⚠️ Error';
        break;
      default:
        color = '#9E9E9E';
        text = '❓ Unknown';
    }
    
    indicator.style.cssText = `
      position: fixed;
      bottom: 10px;
      right: 10px;
      padding: 6px 12px;
      background: ${color};
      color: white;
      font-size: 11px;
      font-family: monospace;
      border-radius: 6px;
      z-index: 9999;
      transition: all 0.3s ease;
      cursor: pointer;
      user-select: none;
      box-shadow: 0 2px 8px rgba(0,0,0,0.15);
    `;
    indicator.textContent = text;
    indicator.title = `Shipwright Hot Reload\nState: ${state}\nReconnect attempts: ${stats.reconnectAttempts}\nTemplates: ${stats.templatesLoaded}\nClick for details`;
  };
  
  // Add click handler for debugging info
  indicator.addEventListener('click', () => {
    const stats = globalClient.getStats();
    console.log('[HotReload] Stats:', stats);
    globalClient.toastNotifier.show(`Stats logged to console`, 'info', 2000);
  });
  
  document.body.appendChild(indicator);
  updateIndicator();
  
  // Update indicator on state changes
  const originalConnect = globalClient.connect;
  const originalScheduleReconnect = globalClient.scheduleReconnect;
  const originalHandleMessage = globalClient.handleMessage;
  
  globalClient.connect = function() {
    const result = originalConnect.call(this);
    updateIndicator();
    return result;
  };
  
  globalClient.scheduleReconnect = function() {
    const result = originalScheduleReconnect.call(this);
    updateIndicator();
    return result;
  };
  
  globalClient.handleMessage = function(message) {
    const result = originalHandleMessage.call(this, message);
    updateIndicator();
    return result;
  };
  
  // Update every few seconds
  setInterval(updateIndicator, 3000);
}

/**
 * Add keyboard shortcuts for debugging
 */
function addDebugShortcuts() {
  document.addEventListener('keydown', (event) => {
    // Ctrl+Shift+R: Force reconnect
    if (event.ctrlKey && event.shiftKey && event.key === 'R') {
      event.preventDefault();
      console.log('[HotReload] Force reconnecting...');
      globalClient.disconnect();
      setTimeout(() => globalClient.connect(), 100);
      return;
    }
    
    // Ctrl+Shift+H: Toggle hot reload stats
    if (event.ctrlKey && event.shiftKey && event.key === 'H') {
      event.preventDefault();
      const stats = globalClient.getStats();
      console.table(stats);
      globalClient.toastNotifier.show('Stats logged to console', 'info', 2000);
      return;
    }
    
    // Ctrl+Shift+T: Toggle toast notifications
    if (event.ctrlKey && event.shiftKey && event.key === 'T') {
      event.preventDefault();
      const enabled = !globalClient.toastNotifier.enabled;
      globalClient.setToastEnabled(enabled);
      console.log(`[HotReload] Toast notifications ${enabled ? 'enabled' : 'disabled'}`);
      if (enabled) {
        globalClient.toastNotifier.show('Toast notifications enabled', 'success', 2000);
      }
      return;
    }
  });
  
  console.log('[HotReload] Debug shortcuts enabled:');
  console.log('  Ctrl+Shift+R: Force reconnect');
  console.log('  Ctrl+Shift+H: Show stats');
  console.log('  Ctrl+Shift+T: Toggle toast notifications');
}