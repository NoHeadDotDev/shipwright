import { LiveViewConnection } from './connection'
import { DomPatcher } from './dom'
import { EventDelegator } from './events'
import { CommandExecutor } from './commands'
import { FormRecovery } from './form-recovery'
import { Protocol, MessageType, RenderMessage, DiffMessage, CommandMessage, RedirectMessage } from './protocol'

export interface LiveViewOptions {
  url: string
  container: string | Element
  token?: string
  params?: Record<string, any>
  reconnectInterval?: number
  maxReconnectAttempts?: number
  heartbeatInterval?: number
  onConnect?: () => void
  onDisconnect?: () => void
  onError?: (error: any) => void
}

export class LiveView {
  private container: Element
  private connection: LiveViewConnection
  private domPatcher: DomPatcher | null = null
  private eventDelegator: EventDelegator | null = null
  private commandExecutor: CommandExecutor
  private formRecovery: FormRecovery
  private _fingerprint: string = ''
  private options: LiveViewOptions

  get fingerprint(): string {
    return this._fingerprint
  }

  constructor(options: LiveViewOptions) {
    this.options = options
    
    // Get container element
    if (typeof options.container === 'string') {
      const element = document.querySelector(options.container)
      if (!element) {
        throw new Error(`Container element not found: ${options.container}`)
      }
      this.container = element
    } else {
      this.container = options.container
    }

    // Initialize connection
    this.connection = new LiveViewConnection({
      url: options.url,
      reconnectInterval: options.reconnectInterval,
      maxReconnectAttempts: options.maxReconnectAttempts,
      heartbeatInterval: options.heartbeatInterval,
      onOpen: () => {
        console.log('LiveView connected')
        options.onConnect?.()
      },
      onClose: () => {
        console.log('LiveView disconnected')
        options.onDisconnect?.()
      },
      onError: (error) => {
        console.error('LiveView error:', error)
        options.onError?.(error)
      },
      onMessage: this.handleMessage.bind(this)
    })

    // Initialize components
    this.commandExecutor = new CommandExecutor()
    this.formRecovery = new FormRecovery()
  }

  connect(): void {
    this.connection.connect(this.options.token, this.options.params)
  }

  disconnect() {
    this.cleanup()
    this.connection.disconnect()
  }

  pushEvent(event: string, payload: any = {}, callback?: (reply: any) => void) {
    const message = Protocol.createEvent(event, undefined, payload)
    this.connection.send(Protocol.encode(message))
    
    // Store callback for response handling if needed
    if (callback) {
      // This would need to be implemented with message IDs for proper tracking
      console.warn('Event callbacks not yet implemented')
    }
  }

  private handleMessage(message: any) {
    switch (message.type) {
      case MessageType.Render:
        this.handleRender(message as RenderMessage)
        break
      case MessageType.Diff:
        this.handleDiff(message as DiffMessage)
        break
      case MessageType.Command:
        this.handleCommand(message as CommandMessage)
        break
      case MessageType.Redirect:
        this.handleRedirect(message as RedirectMessage)
        break
      case MessageType.Error:
        console.error('Server error:', message)
        break
    }
  }

  private handleRender(message: RenderMessage) {
    // Clean up existing handlers
    this.cleanup()

    // Update fingerprint
    this._fingerprint = message.fingerprint

    // Render HTML
    this.container.innerHTML = message.html

    // Initialize components
    this.domPatcher = new DomPatcher(this.container)
    this.eventDelegator = new EventDelegator(this.container, (this.connection as any).ws)
    this.eventDelegator.start()
    this.formRecovery.start(this.container)

    // Load assets if provided
    if (message.assets) {
      this.loadAssets(message.assets)
    }

    // Set up event bindings
    this.bindEvents()
  }

  private handleDiff(message: DiffMessage) {
    if (!this.domPatcher) {
      console.error('DOM patcher not initialized')
      return
    }

    // Update fingerprint
    this._fingerprint = message.fingerprint

    // Apply patches
    this.domPatcher.applyPatches(message.patches)

    // Re-bind events after DOM changes
    this.bindEvents()
  }

  private handleCommand(message: CommandMessage) {
    this.commandExecutor.execute(message.commands)
  }

  private handleRedirect(message: RedirectMessage) {
    if (message.replace) {
      window.location.replace(message.url)
    } else {
      window.location.href = message.url
    }
  }

  private bindEvents() {
    if (!this.eventDelegator) return

    // Find all elements with lv-* attributes
    const elements = this.container.querySelectorAll('[lv-click], [lv-submit], [lv-change], [lv-keyup], [lv-keydown], [lv-blur], [lv-focus]')
    
    elements.forEach(element => {
      const attributes = element.attributes
      
      for (let i = 0; i < attributes.length; i++) {
        const attr = attributes[i]
        
        if (attr.name.startsWith('lv-')) {
          const eventType = attr.name.substring(3) // Remove 'lv-' prefix
          const selector = this.getSelector(element)
          
          // Parse event config from attribute value
          const config = this.parseEventConfig(attr.value)
          
          this.eventDelegator!.register(selector, eventType, config)
        }
      }
    })
  }

  private getSelector(element: Element): string {
    // Generate a unique selector for the element
    if (element.id) {
      return `#${element.id}`
    }
    
    // Use data attributes if available
    const dataAttrs = Array.from(element.attributes)
      .filter(attr => attr.name.startsWith('data-'))
      .map(attr => `[${attr.name}="${attr.value}"]`)
      .join('')
    
    if (dataAttrs) {
      return `${element.tagName.toLowerCase()}${dataAttrs}`
    }
    
    // Fallback to class-based selector
    const classes = Array.from(element.classList).map(c => `.${c}`).join('')
    return `${element.tagName.toLowerCase()}${classes}`
  }

  private parseEventConfig(value: string): any {
    // Parse event modifiers like "click:prevent:throttle-500"
    const parts = value.split(':')
    const config: any = {}
    
    parts.forEach(part => {
      if (part === 'prevent') {
        config.preventDefault = true
      } else if (part === 'stop') {
        config.stopPropagation = true
      } else if (part.startsWith('debounce-')) {
        config.debounce = parseInt(part.substring(9))
      } else if (part.startsWith('throttle-')) {
        config.throttle = parseInt(part.substring(9))
      }
    })
    
    return config
  }

  private loadAssets(assets: { css?: string[], js?: string[] }) {
    // Load CSS
    if (assets.css) {
      assets.css.forEach(href => {
        if (!document.querySelector(`link[href="${href}"]`)) {
          const link = document.createElement('link')
          link.rel = 'stylesheet'
          link.href = href
          document.head.appendChild(link)
        }
      })
    }

    // Load JS
    if (assets.js) {
      assets.js.forEach(src => {
        if (!document.querySelector(`script[src="${src}"]`)) {
          const script = document.createElement('script')
          script.src = src
          script.async = true
          document.head.appendChild(script)
        }
      })
    }
  }

  private cleanup() {
    if (this.eventDelegator) {
      this.eventDelegator.stop()
      this.eventDelegator = null
    }
    
    this.formRecovery.stop()
    
    this.domPatcher = null
  }
}

// Export main class and types
export { Protocol, MessageType } from './protocol'
export type { LiveViewConnection } from './connection'
export type { ClientCommand, CommandType } from './protocol'

// Auto-initialize if data attributes are present
if (typeof window !== 'undefined') {
  document.addEventListener('DOMContentLoaded', () => {
    const elements = document.querySelectorAll('[data-liveview-url]')
    
    elements.forEach(element => {
      const url = element.getAttribute('data-liveview-url')
      const token = element.getAttribute('data-liveview-token')
      
      if (url) {
        const lv = new LiveView({
          url,
          container: element,
          token: token || undefined
        });
        
        lv.connect();
        
        // Store instance on element for later access
        (element as any).__liveview = lv
      }
    })
  })
}