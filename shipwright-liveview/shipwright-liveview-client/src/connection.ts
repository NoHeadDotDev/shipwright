import { Protocol, MessageType } from './protocol'

export interface ConnectionOptions {
  url: string
  reconnectInterval?: number
  maxReconnectAttempts?: number
  heartbeatInterval?: number
  onOpen?: () => void
  onClose?: () => void
  onError?: (error: Event) => void
  onMessage?: (message: any) => void
}

export class LiveViewConnection {
  ws: WebSocket | null = null
  private options: Required<ConnectionOptions>
  private reconnectAttempts = 0
  private reconnectTimer: number | null = null
  private heartbeatTimer: number | null = null
  private messageQueue: Uint8Array[] = []
  private isConnecting = false

  constructor(options: ConnectionOptions) {
    this.options = {
      reconnectInterval: 1000,
      maxReconnectAttempts: 10,
      heartbeatInterval: 30000,
      onOpen: () => {},
      onClose: () => {},
      onError: () => {},
      onMessage: () => {},
      ...options
    }
  }

  connect(token?: string, params?: Record<string, any>) {
    if (this.isConnecting || (this.ws && this.ws.readyState === WebSocket.OPEN)) {
      return
    }

    this.isConnecting = true
    
    try {
      this.ws = new WebSocket(this.options.url)
      this.ws.binaryType = 'arraybuffer'

      this.ws.onopen = () => {
        this.isConnecting = false
        this.reconnectAttempts = 0
        this.startHeartbeat()
        
        // Send connect message
        const connectMsg = Protocol.createConnect(token, params)
        this.send(Protocol.encode(connectMsg))
        
        // Flush queued messages
        while (this.messageQueue.length > 0) {
          const msg = this.messageQueue.shift()!
          this.ws!.send(msg)
        }
        
        this.options.onOpen()
      }

      this.ws.onmessage = (event) => {
        try {
          const message = Protocol.decode(event.data)
          this.handleMessage(message)
          this.options.onMessage(message)
        } catch (error) {
          console.error('Failed to decode message:', error)
        }
      }

      this.ws.onerror = (error) => {
        this.isConnecting = false
        this.options.onError(error)
      }

      this.ws.onclose = () => {
        this.isConnecting = false
        this.stopHeartbeat()
        this.options.onClose()
        this.scheduleReconnect()
      }
    } catch (error) {
      this.isConnecting = false
      console.error('Failed to create WebSocket:', error)
      this.scheduleReconnect()
    }
  }

  disconnect() {
    this.stopReconnect()
    this.stopHeartbeat()
    
    if (this.ws) {
      this.ws.close()
      this.ws = null
    }
    
    this.messageQueue = []
  }

  send(data: Uint8Array) {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(data)
    } else {
      // Queue message for sending after reconnect
      this.messageQueue.push(data)
    }
  }

  private handleMessage(message: any) {
    switch (message.type) {
      case MessageType.Error:
        console.error('Server error:', message.message)
        break
      case MessageType.Ack:
        // Handle acknowledgment if needed
        break
    }
  }

  private startHeartbeat() {
    this.stopHeartbeat()
    
    this.heartbeatTimer = setInterval(() => {
      if (this.ws && this.ws.readyState === WebSocket.OPEN) {
        const heartbeat = Protocol.createHeartbeat()
        this.ws.send(Protocol.encode(heartbeat))
      }
    }, this.options.heartbeatInterval) as unknown as number
  }

  private stopHeartbeat() {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer)
      this.heartbeatTimer = null
    }
  }

  private scheduleReconnect() {
    if (this.reconnectAttempts >= this.options.maxReconnectAttempts) {
      console.error('Max reconnection attempts reached')
      return
    }

    this.stopReconnect()
    
    const delay = Math.min(
      this.options.reconnectInterval * Math.pow(2, this.reconnectAttempts),
      30000 // Max 30 seconds
    )
    
    this.reconnectTimer = setTimeout(() => {
      this.reconnectAttempts++
      this.connect()
    }, delay) as unknown as number
  }

  private stopReconnect() {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer)
      this.reconnectTimer = null
    }
  }

  get readyState(): number {
    return this.ws ? this.ws.readyState : WebSocket.CLOSED
  }

  get isConnected(): boolean {
    return this.ws !== null && this.ws.readyState === WebSocket.OPEN
  }
}