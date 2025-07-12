import { Protocol } from './protocol'

export interface EventConfig {
  preventDefault?: boolean
  stopPropagation?: boolean
  debounce?: number
  throttle?: number
}

export class EventDelegator {
  private element: Element
  private websocket: WebSocket
  private listeners: Map<string, Map<string, EventConfig>> = new Map()
  private debounceTimers: Map<string, number> = new Map()
  private throttleTimers: Map<string, number> = new Map()

  constructor(element: Element, websocket: WebSocket) {
    this.element = element
    this.websocket = websocket
  }

  start() {
    // Common events to delegate
    const events = [
      'click', 'dblclick', 'mousedown', 'mouseup', 'mouseover', 'mouseout',
      'keydown', 'keyup', 'keypress', 'input', 'change', 'submit',
      'focus', 'blur', 'focusin', 'focusout'
    ]

    events.forEach(eventType => {
      this.element.addEventListener(eventType, this.handleEvent.bind(this), true)
    })
  }

  stop() {
    this.listeners.clear()
    this.debounceTimers.forEach(timer => clearTimeout(timer))
    this.throttleTimers.forEach(timer => clearTimeout(timer))
  }

  register(selector: string, event: string, config: EventConfig = {}) {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, new Map())
    }
    this.listeners.get(event)!.set(selector, config)
  }

  private handleEvent(event: Event) {
    const target = event.target as Element
    const listeners = this.listeners.get(event.type)
    
    if (!listeners) return

    // Find matching selector
    for (const [selector, config] of listeners) {
      if (target.matches(selector)) {
        this.processEvent(event, selector, config)
        break
      }
    }
  }

  private processEvent(event: Event, selector: string, config: EventConfig) {
    if (config.preventDefault) {
      event.preventDefault()
    }
    
    if (config.stopPropagation) {
      event.stopPropagation()
    }

    const eventKey = `${event.type}:${selector}`

    if (config.debounce) {
      this.debounce(eventKey, () => this.sendEvent(event, selector), config.debounce)
    } else if (config.throttle) {
      this.throttle(eventKey, () => this.sendEvent(event, selector), config.throttle)
    } else {
      this.sendEvent(event, selector)
    }
  }

  private sendEvent(event: Event, selector: string) {
    const target = event.target as Element
    const value = this.extractValue(target)
    const metadata = this.extractMetadata(event)

    const message = Protocol.createEvent(
      event.type,
      selector,
      value,
      metadata
    )

    if (this.websocket.readyState === WebSocket.OPEN) {
      this.websocket.send(Protocol.encode(message))
    }
  }

  private extractValue(element: Element): any {
    if (element instanceof HTMLInputElement) {
      if (element.type === 'checkbox' || element.type === 'radio') {
        return element.checked
      }
      return element.value
    } else if (element instanceof HTMLTextAreaElement) {
      return element.value
    } else if (element instanceof HTMLSelectElement) {
      return element.value
    }
    return null
  }

  private extractMetadata(event: Event): Record<string, any> {
    const metadata: Record<string, any> = {}

    if (event instanceof MouseEvent) {
      metadata.x = event.clientX
      metadata.y = event.clientY
      metadata.button = event.button
      metadata.ctrlKey = event.ctrlKey
      metadata.shiftKey = event.shiftKey
      metadata.altKey = event.altKey
      metadata.metaKey = event.metaKey
    } else if (event instanceof KeyboardEvent) {
      metadata.key = event.key
      metadata.code = event.code
      metadata.ctrlKey = event.ctrlKey
      metadata.shiftKey = event.shiftKey
      metadata.altKey = event.altKey
      metadata.metaKey = event.metaKey
    }

    return metadata
  }

  private debounce(key: string, fn: Function, delay: number) {
    const existing = this.debounceTimers.get(key)
    if (existing) {
      clearTimeout(existing)
    }

    const timer = setTimeout(() => {
      fn()
      this.debounceTimers.delete(key)
    }, delay) as unknown as number

    this.debounceTimers.set(key, timer)
  }

  private throttle(key: string, fn: Function, delay: number) {
    if (this.throttleTimers.has(key)) {
      return
    }

    fn()

    const timer = setTimeout(() => {
      this.throttleTimers.delete(key)
    }, delay) as unknown as number

    this.throttleTimers.set(key, timer)
  }
}