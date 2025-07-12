function encode(value) {
  const buffer = [];
  encodeValue(value, buffer);
  return new Uint8Array(buffer);
}
function decode(data) {
  const view = new DataView(data);
  const result = decodeValue(view, 0);
  return result.value;
}
function encodeValue(value, buffer) {
  if (value === null) {
    buffer.push(192);
  } else if (value === false) {
    buffer.push(194);
  } else if (value === true) {
    buffer.push(195);
  } else if (typeof value === "number") {
    encodeNumber(value, buffer);
  } else if (typeof value === "string") {
    encodeString(value, buffer);
  } else if (value instanceof Uint8Array) {
    encodeBinary(value, buffer);
  } else if (Array.isArray(value)) {
    encodeArray(value, buffer);
  } else if (typeof value === "object") {
    encodeObject(value, buffer);
  }
}
function encodeNumber(value, buffer) {
  if (Number.isInteger(value)) {
    if (value >= 0 && value <= 127) {
      buffer.push(value);
    } else if (value >= -32 && value < 0) {
      buffer.push(224 | value + 32);
    } else if (value >= -128 && value <= 127) {
      buffer.push(208, value & 255);
    } else if (value >= -32768 && value <= 32767) {
      buffer.push(209, value >> 8 & 255, value & 255);
    } else {
      buffer.push(210);
      buffer.push(value >> 24 & 255);
      buffer.push(value >> 16 & 255);
      buffer.push(value >> 8 & 255);
      buffer.push(value & 255);
    }
  } else {
    const view = new DataView(new ArrayBuffer(8));
    view.setFloat64(0, value);
    buffer.push(203);
    for (let i = 0; i < 8; i++) {
      buffer.push(view.getUint8(i));
    }
  }
}
function encodeString(value, buffer) {
  const encoded = new TextEncoder().encode(value);
  const len = encoded.length;
  if (len <= 31) {
    buffer.push(160 | len);
  } else if (len <= 255) {
    buffer.push(217, len);
  } else if (len <= 65535) {
    buffer.push(218, len >> 8 & 255, len & 255);
  }
  for (const byte of encoded) {
    buffer.push(byte);
  }
}
function encodeBinary(value, buffer) {
  const len = value.length;
  if (len <= 255) {
    buffer.push(196, len);
  } else if (len <= 65535) {
    buffer.push(197, len >> 8 & 255, len & 255);
  }
  for (const byte of value) {
    buffer.push(byte);
  }
}
function encodeArray(value, buffer) {
  const len = value.length;
  if (len <= 15) {
    buffer.push(144 | len);
  } else if (len <= 65535) {
    buffer.push(220, len >> 8 & 255, len & 255);
  }
  for (const item of value) {
    encodeValue(item, buffer);
  }
}
function encodeObject(value, buffer) {
  const keys = Object.keys(value);
  const len = keys.length;
  if (len <= 15) {
    buffer.push(128 | len);
  } else if (len <= 65535) {
    buffer.push(222, len >> 8 & 255, len & 255);
  }
  for (const key of keys) {
    encodeString(key, buffer);
    encodeValue(value[key], buffer);
  }
}
function decodeValue(view, offset) {
  const byte = view.getUint8(offset);
  if (byte === 192) return { value: null, offset: offset + 1 };
  if (byte === 194) return { value: false, offset: offset + 1 };
  if (byte === 195) return { value: true, offset: offset + 1 };
  if ((byte & 128) === 0) {
    return { value: byte, offset: offset + 1 };
  }
  if ((byte & 224) === 224) {
    return { value: byte - 256, offset: offset + 1 };
  }
  if ((byte & 224) === 160) {
    const len = byte & 31;
    return decodeString(view, offset + 1, len);
  }
  if ((byte & 240) === 144) {
    const len = byte & 15;
    return decodeArray(view, offset + 1, len);
  }
  if ((byte & 240) === 128) {
    const len = byte & 15;
    return decodeObject(view, offset + 1, len);
  }
  switch (byte) {
    case 208:
      return { value: view.getInt8(offset + 1), offset: offset + 2 };
    case 209:
      return { value: view.getInt16(offset + 1), offset: offset + 3 };
    case 210:
      return { value: view.getInt32(offset + 1), offset: offset + 5 };
    case 203:
      return { value: view.getFloat64(offset + 1), offset: offset + 9 };
    case 217:
      const strLen8 = view.getUint8(offset + 1);
      return decodeString(view, offset + 2, strLen8);
    case 218:
      const strLen16 = view.getUint16(offset + 1);
      return decodeString(view, offset + 3, strLen16);
    case 220:
      const arrLen16 = view.getUint16(offset + 1);
      return decodeArray(view, offset + 3, arrLen16);
    case 222:
      const mapLen16 = view.getUint16(offset + 1);
      return decodeObject(view, offset + 3, mapLen16);
    case 196:
      const binLen8 = view.getUint8(offset + 1);
      return decodeBinary(view, offset + 2, binLen8);
    case 197:
      const binLen16 = view.getUint16(offset + 1);
      return decodeBinary(view, offset + 3, binLen16);
    default:
      throw new Error(`Unsupported MessagePack type: 0x${byte.toString(16)}`);
  }
}
function decodeString(view, offset, length) {
  const bytes = new Uint8Array(length);
  for (let i = 0; i < length; i++) {
    bytes[i] = view.getUint8(offset + i);
  }
  const value = new TextDecoder().decode(bytes);
  return { value, offset: offset + length };
}
function decodeArray(view, offset, length) {
  const arr = [];
  let currentOffset = offset;
  for (let i = 0; i < length; i++) {
    const result = decodeValue(view, currentOffset);
    arr.push(result.value);
    currentOffset = result.offset;
  }
  return { value: arr, offset: currentOffset };
}
function decodeObject(view, offset, length) {
  const obj = {};
  let currentOffset = offset;
  for (let i = 0; i < length; i++) {
    const keyResult = decodeValue(view, currentOffset);
    currentOffset = keyResult.offset;
    const valueResult = decodeValue(view, currentOffset);
    currentOffset = valueResult.offset;
    obj[keyResult.value] = valueResult.value;
  }
  return { value: obj, offset: currentOffset };
}
function decodeBinary(view, offset, length) {
  const bytes = new Uint8Array(length);
  for (let i = 0; i < length; i++) {
    bytes[i] = view.getUint8(offset + i);
  }
  return { value: bytes, offset: offset + length };
}
var MessageType = /* @__PURE__ */ ((MessageType2) => {
  MessageType2[MessageType2["Connect"] = 1] = "Connect";
  MessageType2[MessageType2["Event"] = 2] = "Event";
  MessageType2[MessageType2["Heartbeat"] = 3] = "Heartbeat";
  MessageType2[MessageType2["Render"] = 16] = "Render";
  MessageType2[MessageType2["Diff"] = 17] = "Diff";
  MessageType2[MessageType2["Redirect"] = 18] = "Redirect";
  MessageType2[MessageType2["Command"] = 19] = "Command";
  MessageType2[MessageType2["Error"] = 20] = "Error";
  MessageType2[MessageType2["Ack"] = 21] = "Ack";
  return MessageType2;
})(MessageType || {});
var PatchOp = /* @__PURE__ */ ((PatchOp2) => {
  PatchOp2[PatchOp2["Replace"] = 1] = "Replace";
  PatchOp2[PatchOp2["Remove"] = 2] = "Remove";
  PatchOp2[PatchOp2["Insert"] = 3] = "Insert";
  PatchOp2[PatchOp2["Update"] = 4] = "Update";
  PatchOp2[PatchOp2["SetAttr"] = 5] = "SetAttr";
  PatchOp2[PatchOp2["RemoveAttr"] = 6] = "RemoveAttr";
  PatchOp2[PatchOp2["AddClass"] = 7] = "AddClass";
  PatchOp2[PatchOp2["RemoveClass"] = 8] = "RemoveClass";
  PatchOp2[PatchOp2["SetProp"] = 9] = "SetProp";
  return PatchOp2;
})(PatchOp || {});
var CommandType = /* @__PURE__ */ ((CommandType2) => {
  CommandType2[CommandType2["Show"] = 1] = "Show";
  CommandType2[CommandType2["Hide"] = 2] = "Hide";
  CommandType2[CommandType2["Toggle"] = 3] = "Toggle";
  CommandType2[CommandType2["AddClass"] = 4] = "AddClass";
  CommandType2[CommandType2["RemoveClass"] = 5] = "RemoveClass";
  CommandType2[CommandType2["SetAttribute"] = 6] = "SetAttribute";
  CommandType2[CommandType2["RemoveAttribute"] = 7] = "RemoveAttribute";
  CommandType2[CommandType2["Dispatch"] = 8] = "Dispatch";
  CommandType2[CommandType2["Push"] = 9] = "Push";
  CommandType2[CommandType2["Focus"] = 10] = "Focus";
  CommandType2[CommandType2["Blur"] = 11] = "Blur";
  return CommandType2;
})(CommandType || {});
class Protocol {
  static encode(message) {
    return encode(message);
  }
  static decode(data) {
    return decode(data);
  }
  static createConnect(token, params) {
    return { type: 1, token, params };
  }
  static createEvent(event, element, value, metadata) {
    return { type: 2, event, element, value, metadata };
  }
  static createHeartbeat() {
    return { type: 3, timestamp: Date.now() };
  }
}
class LiveViewConnection {
  constructor(options) {
    this.ws = null;
    this.reconnectAttempts = 0;
    this.reconnectTimer = null;
    this.heartbeatTimer = null;
    this.messageQueue = [];
    this.isConnecting = false;
    this.options = {
      reconnectInterval: 1e3,
      maxReconnectAttempts: 10,
      heartbeatInterval: 3e4,
      onOpen: () => {
      },
      onClose: () => {
      },
      onError: () => {
      },
      onMessage: () => {
      },
      ...options
    };
  }
  connect(token, params) {
    if (this.isConnecting || this.ws && this.ws.readyState === WebSocket.OPEN) {
      return;
    }
    this.isConnecting = true;
    try {
      this.ws = new WebSocket(this.options.url);
      this.ws.binaryType = "arraybuffer";
      this.ws.onopen = () => {
        this.isConnecting = false;
        this.reconnectAttempts = 0;
        this.startHeartbeat();
        const connectMsg = Protocol.createConnect(token, params);
        this.send(Protocol.encode(connectMsg));
        while (this.messageQueue.length > 0) {
          const msg = this.messageQueue.shift();
          this.ws.send(msg);
        }
        this.options.onOpen();
      };
      this.ws.onmessage = (event) => {
        try {
          const message = Protocol.decode(event.data);
          this.handleMessage(message);
          this.options.onMessage(message);
        } catch (error) {
          console.error("Failed to decode message:", error);
        }
      };
      this.ws.onerror = (error) => {
        this.isConnecting = false;
        this.options.onError(error);
      };
      this.ws.onclose = () => {
        this.isConnecting = false;
        this.stopHeartbeat();
        this.options.onClose();
        this.scheduleReconnect();
      };
    } catch (error) {
      this.isConnecting = false;
      console.error("Failed to create WebSocket:", error);
      this.scheduleReconnect();
    }
  }
  disconnect() {
    this.stopReconnect();
    this.stopHeartbeat();
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
    this.messageQueue = [];
  }
  send(data) {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(data);
    } else {
      this.messageQueue.push(data);
    }
  }
  handleMessage(message) {
    switch (message.type) {
      case MessageType.Error:
        console.error("Server error:", message.message);
        break;
      case MessageType.Ack:
        break;
    }
  }
  startHeartbeat() {
    this.stopHeartbeat();
    this.heartbeatTimer = setInterval(() => {
      if (this.ws && this.ws.readyState === WebSocket.OPEN) {
        const heartbeat = Protocol.createHeartbeat();
        this.ws.send(Protocol.encode(heartbeat));
      }
    }, this.options.heartbeatInterval);
  }
  stopHeartbeat() {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer);
      this.heartbeatTimer = null;
    }
  }
  scheduleReconnect() {
    if (this.reconnectAttempts >= this.options.maxReconnectAttempts) {
      console.error("Max reconnection attempts reached");
      return;
    }
    this.stopReconnect();
    const delay = Math.min(
      this.options.reconnectInterval * Math.pow(2, this.reconnectAttempts),
      3e4
      // Max 30 seconds
    );
    this.reconnectTimer = setTimeout(() => {
      this.reconnectAttempts++;
      this.connect();
    }, delay);
  }
  stopReconnect() {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
  }
  get readyState() {
    return this.ws ? this.ws.readyState : WebSocket.CLOSED;
  }
  get isConnected() {
    return this.ws !== null && this.ws.readyState === WebSocket.OPEN;
  }
}
class DomPatcher {
  // Cache for future optimizations
  // private nodeCache: WeakMap<Element, number[]> = new WeakMap()
  constructor(root) {
    this.root = root;
  }
  applyPatches(patches) {
    for (const patch of patches) {
      this.applyPatch(patch);
    }
  }
  applyPatch(patch) {
    const node = this.findNode(patch.path);
    if (!node) return;
    switch (patch.op) {
      case PatchOp.Replace:
        this.replace(node, patch.data);
        break;
      case PatchOp.Remove:
        this.remove(node);
        break;
      case PatchOp.Insert:
        this.insert(node, patch.data.index, patch.data.html);
        break;
      case PatchOp.Update:
        this.update(node, patch.data);
        break;
      case PatchOp.SetAttr:
        this.setAttribute(node, patch.data.name, patch.data.value);
        break;
      case PatchOp.RemoveAttr:
        this.removeAttribute(node, patch.data);
        break;
      case PatchOp.AddClass:
        this.addClass(node, patch.data);
        break;
      case PatchOp.RemoveClass:
        this.removeClass(node, patch.data);
        break;
      case PatchOp.SetProp:
        this.setProperty(node, patch.data.name, patch.data.value);
        break;
    }
  }
  findNode(path) {
    let current = this.root;
    for (const index of path) {
      const children = current.children;
      if (index >= children.length) return null;
      current = children[index];
    }
    return current;
  }
  replace(node, html) {
    const temp = document.createElement("div");
    temp.innerHTML = html;
    const newNode = temp.firstElementChild;
    if (newNode && node.parentNode) {
      node.parentNode.replaceChild(newNode, node);
    }
  }
  remove(node) {
    node.remove();
  }
  insert(parent, index, html) {
    const temp = document.createElement("div");
    temp.innerHTML = html;
    const newNode = temp.firstElementChild;
    if (!newNode) return;
    if (index >= parent.children.length) {
      parent.appendChild(newNode);
    } else {
      parent.insertBefore(newNode, parent.children[index]);
    }
  }
  update(node, content) {
    if (node instanceof HTMLInputElement || node instanceof HTMLTextAreaElement || node instanceof HTMLSelectElement) {
      const activeElement = document.activeElement;
      const selectionStart = node.selectionStart;
      const selectionEnd = node.selectionEnd;
      node.value = content;
      if (activeElement === node && selectionStart !== void 0) {
        node.setSelectionRange(selectionStart, selectionEnd);
      }
    } else {
      node.textContent = content;
    }
  }
  setAttribute(node, name, value) {
    node.setAttribute(name, value);
  }
  removeAttribute(node, name) {
    node.removeAttribute(name);
  }
  addClass(node, className) {
    node.classList.add(className);
  }
  removeClass(node, className) {
    node.classList.remove(className);
  }
  setProperty(node, name, value) {
    node[name] = value;
  }
}
class EventDelegator {
  constructor(element, websocket) {
    this.listeners = /* @__PURE__ */ new Map();
    this.debounceTimers = /* @__PURE__ */ new Map();
    this.throttleTimers = /* @__PURE__ */ new Map();
    this.element = element;
    this.websocket = websocket;
  }
  start() {
    const events = [
      "click",
      "dblclick",
      "mousedown",
      "mouseup",
      "mouseover",
      "mouseout",
      "keydown",
      "keyup",
      "keypress",
      "input",
      "change",
      "submit",
      "focus",
      "blur",
      "focusin",
      "focusout"
    ];
    events.forEach((eventType) => {
      this.element.addEventListener(eventType, this.handleEvent.bind(this), true);
    });
  }
  stop() {
    this.listeners.clear();
    this.debounceTimers.forEach((timer) => clearTimeout(timer));
    this.throttleTimers.forEach((timer) => clearTimeout(timer));
  }
  register(selector, event, config = {}) {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, /* @__PURE__ */ new Map());
    }
    this.listeners.get(event).set(selector, config);
  }
  handleEvent(event) {
    const target = event.target;
    const listeners = this.listeners.get(event.type);
    if (!listeners) return;
    for (const [selector, config] of listeners) {
      if (target.matches(selector)) {
        this.processEvent(event, selector, config);
        break;
      }
    }
  }
  processEvent(event, selector, config) {
    if (config.preventDefault) {
      event.preventDefault();
    }
    if (config.stopPropagation) {
      event.stopPropagation();
    }
    const eventKey = `${event.type}:${selector}`;
    if (config.debounce) {
      this.debounce(eventKey, () => this.sendEvent(event, selector), config.debounce);
    } else if (config.throttle) {
      this.throttle(eventKey, () => this.sendEvent(event, selector), config.throttle);
    } else {
      this.sendEvent(event, selector);
    }
  }
  sendEvent(event, selector) {
    const target = event.target;
    const value = this.extractValue(target);
    const metadata = this.extractMetadata(event);
    const message = Protocol.createEvent(
      event.type,
      selector,
      value,
      metadata
    );
    if (this.websocket.readyState === WebSocket.OPEN) {
      this.websocket.send(Protocol.encode(message));
    }
  }
  extractValue(element) {
    if (element instanceof HTMLInputElement) {
      if (element.type === "checkbox" || element.type === "radio") {
        return element.checked;
      }
      return element.value;
    } else if (element instanceof HTMLTextAreaElement) {
      return element.value;
    } else if (element instanceof HTMLSelectElement) {
      return element.value;
    }
    return null;
  }
  extractMetadata(event) {
    const metadata = {};
    if (event instanceof MouseEvent) {
      metadata.x = event.clientX;
      metadata.y = event.clientY;
      metadata.button = event.button;
      metadata.ctrlKey = event.ctrlKey;
      metadata.shiftKey = event.shiftKey;
      metadata.altKey = event.altKey;
      metadata.metaKey = event.metaKey;
    } else if (event instanceof KeyboardEvent) {
      metadata.key = event.key;
      metadata.code = event.code;
      metadata.ctrlKey = event.ctrlKey;
      metadata.shiftKey = event.shiftKey;
      metadata.altKey = event.altKey;
      metadata.metaKey = event.metaKey;
    }
    return metadata;
  }
  debounce(key, fn, delay) {
    const existing = this.debounceTimers.get(key);
    if (existing) {
      clearTimeout(existing);
    }
    const timer = setTimeout(() => {
      fn();
      this.debounceTimers.delete(key);
    }, delay);
    this.debounceTimers.set(key, timer);
  }
  throttle(key, fn, delay) {
    if (this.throttleTimers.has(key)) {
      return;
    }
    fn();
    const timer = setTimeout(() => {
      this.throttleTimers.delete(key);
    }, delay);
    this.throttleTimers.set(key, timer);
  }
}
class CommandExecutor {
  execute(commands) {
    for (const command of commands) {
      this.executeCommand(command);
    }
  }
  executeCommand(command) {
    const elements = document.querySelectorAll(command.target);
    elements.forEach((element) => {
      switch (command.type) {
        case CommandType.Show:
          this.show(element, command.transition);
          break;
        case CommandType.Hide:
          this.hide(element, command.transition);
          break;
        case CommandType.Toggle:
          this.toggle(element, command.transition);
          break;
        case CommandType.AddClass:
          this.addClass(element, command.args);
          break;
        case CommandType.RemoveClass:
          this.removeClass(element, command.args);
          break;
        case CommandType.SetAttribute:
          this.setAttribute(element, command.args.name, command.args.value);
          break;
        case CommandType.RemoveAttribute:
          this.removeAttribute(element, command.args);
          break;
        case CommandType.Dispatch:
          this.dispatch(element, command.args);
          break;
        case CommandType.Push:
          this.push(command.args);
          break;
        case CommandType.Focus:
          this.focus(element);
          break;
        case CommandType.Blur:
          this.blur(element);
          break;
      }
    });
  }
  show(element, transition) {
    if (transition) {
      this.applyTransition(element, transition, () => {
        element.style.display = "";
      });
    } else {
      element.style.display = "";
    }
  }
  hide(element, transition) {
    if (transition) {
      this.applyTransition(element, transition, () => {
        element.style.display = "none";
      });
    } else {
      element.style.display = "none";
    }
  }
  toggle(element, transition) {
    const isHidden = element.style.display === "none" || window.getComputedStyle(element).display === "none";
    if (isHidden) {
      this.show(element, transition);
    } else {
      this.hide(element, transition);
    }
  }
  addClass(element, className) {
    element.classList.add(...className.split(" "));
  }
  removeClass(element, className) {
    element.classList.remove(...className.split(" "));
  }
  setAttribute(element, name, value) {
    element.setAttribute(name, value);
  }
  removeAttribute(element, name) {
    element.removeAttribute(name);
  }
  dispatch(element, eventName) {
    element.dispatchEvent(new CustomEvent(eventName, { bubbles: true }));
  }
  push(eventData) {
    window.dispatchEvent(new CustomEvent("liveview:push", { detail: eventData }));
  }
  focus(element) {
    element.focus();
  }
  blur(element) {
    element.blur();
  }
  applyTransition(element, transition, callback) {
    const duration = transition.duration || 300;
    const from = transition.from || {};
    const to = transition.to || {};
    Object.assign(element.style, from);
    element.offsetHeight;
    element.style.transition = `all ${duration}ms ease-in-out`;
    requestAnimationFrame(() => {
      Object.assign(element.style, to);
    });
    setTimeout(() => {
      element.style.transition = "";
      callback();
    }, duration);
  }
}
class FormRecovery {
  constructor() {
    this.forms = /* @__PURE__ */ new Map();
    this.observer = null;
  }
  start(element) {
    this.saveAllForms(element);
    this.observer = new MutationObserver((mutations) => {
      mutations.forEach((mutation) => {
        if (mutation.type === "childList") {
          mutation.removedNodes.forEach((node) => {
            if (node instanceof Element) {
              this.saveForms(node);
            }
          });
          mutation.addedNodes.forEach((node) => {
            if (node instanceof Element) {
              this.restoreForms(node);
            }
          });
        }
      });
    });
    this.observer.observe(element, {
      childList: true,
      subtree: true
    });
  }
  stop() {
    if (this.observer) {
      this.observer.disconnect();
      this.observer = null;
    }
    this.forms.clear();
  }
  saveAllForms(element) {
    const forms = element.querySelectorAll("form");
    forms.forEach((form) => this.saveForm(form));
  }
  saveForms(element) {
    if (element instanceof HTMLFormElement) {
      this.saveForm(element);
    } else {
      const forms = element.querySelectorAll("form");
      forms.forEach((form) => this.saveForm(form));
    }
  }
  restoreForms(element) {
    if (element instanceof HTMLFormElement) {
      this.restoreForm(element);
    } else {
      const forms = element.querySelectorAll("form");
      forms.forEach((form) => this.restoreForm(form));
    }
  }
  saveForm(form) {
    const formId = this.getFormId(form);
    if (!formId) return;
    const formData = new FormData();
    const elements = form.elements;
    for (let i = 0; i < elements.length; i++) {
      const element = elements[i];
      if (!element.name) continue;
      if (element instanceof HTMLInputElement) {
        if (element.type === "checkbox" || element.type === "radio") {
          if (element.checked) {
            formData.append(element.name, element.value);
          }
        } else if (element.type !== "file" && element.type !== "submit" && element.type !== "button") {
          formData.append(element.name, element.value);
        }
      } else if (element instanceof HTMLTextAreaElement || element instanceof HTMLSelectElement) {
        formData.append(element.name, element.value);
      }
    }
    this.forms.set(formId, formData);
  }
  restoreForm(form) {
    const formId = this.getFormId(form);
    if (!formId) return;
    const savedData = this.forms.get(formId);
    if (!savedData) return;
    const elements = form.elements;
    for (let i = 0; i < elements.length; i++) {
      const element = elements[i];
      if (!element.name) continue;
      const savedValues = savedData.getAll(element.name);
      if (savedValues.length === 0) continue;
      if (element instanceof HTMLInputElement) {
        if (element.type === "checkbox" || element.type === "radio") {
          element.checked = savedValues.includes(element.value);
        } else if (element.type !== "file") {
          element.value = savedValues[0];
        }
      } else if (element instanceof HTMLTextAreaElement || element instanceof HTMLSelectElement) {
        element.value = savedValues[0];
      }
    }
    const activeElement = document.activeElement;
    if (activeElement && form.contains(activeElement)) {
      if (activeElement instanceof HTMLInputElement || activeElement instanceof HTMLTextAreaElement) {
        const cursorPos = activeElement.selectionStart;
        if (cursorPos !== null) {
          activeElement.setSelectionRange(cursorPos, cursorPos);
        }
      }
    }
  }
  getFormId(form) {
    return form.id || form.name || (form.action ? btoa(form.action) : null);
  }
}
class LiveView {
  constructor(options) {
    this.domPatcher = null;
    this.eventDelegator = null;
    this._fingerprint = "";
    this.options = options;
    if (typeof options.container === "string") {
      const element = document.querySelector(options.container);
      if (!element) {
        throw new Error(`Container element not found: ${options.container}`);
      }
      this.container = element;
    } else {
      this.container = options.container;
    }
    this.connection = new LiveViewConnection({
      url: options.url,
      reconnectInterval: options.reconnectInterval,
      maxReconnectAttempts: options.maxReconnectAttempts,
      heartbeatInterval: options.heartbeatInterval,
      onOpen: () => {
        var _a;
        console.log("LiveView connected");
        (_a = options.onConnect) == null ? void 0 : _a.call(options);
      },
      onClose: () => {
        var _a;
        console.log("LiveView disconnected");
        (_a = options.onDisconnect) == null ? void 0 : _a.call(options);
      },
      onError: (error) => {
        var _a;
        console.error("LiveView error:", error);
        (_a = options.onError) == null ? void 0 : _a.call(options, error);
      },
      onMessage: this.handleMessage.bind(this)
    });
    this.commandExecutor = new CommandExecutor();
    this.formRecovery = new FormRecovery();
  }
  get fingerprint() {
    return this._fingerprint;
  }
  connect() {
    this.connection.connect(this.options.token, this.options.params);
  }
  disconnect() {
    this.cleanup();
    this.connection.disconnect();
  }
  pushEvent(event, payload = {}, callback) {
    const message = Protocol.createEvent(event, void 0, payload);
    this.connection.send(Protocol.encode(message));
    if (callback) {
      console.warn("Event callbacks not yet implemented");
    }
  }
  handleMessage(message) {
    switch (message.type) {
      case MessageType.Render:
        this.handleRender(message);
        break;
      case MessageType.Diff:
        this.handleDiff(message);
        break;
      case MessageType.Command:
        this.handleCommand(message);
        break;
      case MessageType.Redirect:
        this.handleRedirect(message);
        break;
      case MessageType.Error:
        console.error("Server error:", message);
        break;
    }
  }
  handleRender(message) {
    this.cleanup();
    this._fingerprint = message.fingerprint;
    this.container.innerHTML = message.html;
    this.domPatcher = new DomPatcher(this.container);
    this.eventDelegator = new EventDelegator(this.container, this.connection.ws);
    this.eventDelegator.start();
    this.formRecovery.start(this.container);
    if (message.assets) {
      this.loadAssets(message.assets);
    }
    this.bindEvents();
  }
  handleDiff(message) {
    if (!this.domPatcher) {
      console.error("DOM patcher not initialized");
      return;
    }
    this._fingerprint = message.fingerprint;
    this.domPatcher.applyPatches(message.patches);
    this.bindEvents();
  }
  handleCommand(message) {
    this.commandExecutor.execute(message.commands);
  }
  handleRedirect(message) {
    if (message.replace) {
      window.location.replace(message.url);
    } else {
      window.location.href = message.url;
    }
  }
  bindEvents() {
    if (!this.eventDelegator) return;
    const elements = this.container.querySelectorAll("[lv-click], [lv-submit], [lv-change], [lv-keyup], [lv-keydown], [lv-blur], [lv-focus]");
    elements.forEach((element) => {
      const attributes = element.attributes;
      for (let i = 0; i < attributes.length; i++) {
        const attr = attributes[i];
        if (attr.name.startsWith("lv-")) {
          const eventType = attr.name.substring(3);
          const selector = this.getSelector(element);
          const config = this.parseEventConfig(attr.value);
          this.eventDelegator.register(selector, eventType, config);
        }
      }
    });
  }
  getSelector(element) {
    if (element.id) {
      return `#${element.id}`;
    }
    const dataAttrs = Array.from(element.attributes).filter((attr) => attr.name.startsWith("data-")).map((attr) => `[${attr.name}="${attr.value}"]`).join("");
    if (dataAttrs) {
      return `${element.tagName.toLowerCase()}${dataAttrs}`;
    }
    const classes = Array.from(element.classList).map((c) => `.${c}`).join("");
    return `${element.tagName.toLowerCase()}${classes}`;
  }
  parseEventConfig(value) {
    const parts = value.split(":");
    const config = {};
    parts.forEach((part) => {
      if (part === "prevent") {
        config.preventDefault = true;
      } else if (part === "stop") {
        config.stopPropagation = true;
      } else if (part.startsWith("debounce-")) {
        config.debounce = parseInt(part.substring(9));
      } else if (part.startsWith("throttle-")) {
        config.throttle = parseInt(part.substring(9));
      }
    });
    return config;
  }
  loadAssets(assets) {
    if (assets.css) {
      assets.css.forEach((href) => {
        if (!document.querySelector(`link[href="${href}"]`)) {
          const link = document.createElement("link");
          link.rel = "stylesheet";
          link.href = href;
          document.head.appendChild(link);
        }
      });
    }
    if (assets.js) {
      assets.js.forEach((src) => {
        if (!document.querySelector(`script[src="${src}"]`)) {
          const script = document.createElement("script");
          script.src = src;
          script.async = true;
          document.head.appendChild(script);
        }
      });
    }
  }
  cleanup() {
    if (this.eventDelegator) {
      this.eventDelegator.stop();
      this.eventDelegator = null;
    }
    this.formRecovery.stop();
    this.domPatcher = null;
  }
}
if (typeof window !== "undefined") {
  document.addEventListener("DOMContentLoaded", () => {
    const elements = document.querySelectorAll("[data-liveview-url]");
    elements.forEach((element) => {
      const url = element.getAttribute("data-liveview-url");
      const token = element.getAttribute("data-liveview-token");
      if (url) {
        const lv = new LiveView({
          url,
          container: element,
          token: token || void 0
        });
        lv.connect();
        element.__liveview = lv;
      }
    });
  });
}
export {
  LiveView,
  MessageType,
  Protocol
};
