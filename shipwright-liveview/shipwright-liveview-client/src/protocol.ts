import { encode, decode } from './msgpack'

// Message types
export enum MessageType {
  // Client to Server
  Connect = 0x01,
  Event = 0x02,
  Heartbeat = 0x03,
  
  // Server to Client
  Render = 0x10,
  Diff = 0x11,
  Redirect = 0x12,
  Command = 0x13,
  Error = 0x14,
  Ack = 0x15
}

// Client Messages
export interface ConnectMessage {
  type: MessageType.Connect
  token?: string
  params?: Record<string, any>
}

export interface EventMessage {
  type: MessageType.Event
  event: string
  element?: string
  value?: any
  metadata?: Record<string, any>
}

export interface HeartbeatMessage {
  type: MessageType.Heartbeat
  timestamp: number
}

// Server Messages
export interface RenderMessage {
  type: MessageType.Render
  html: string
  fingerprint: string
  assets?: {
    css?: string[]
    js?: string[]
  }
}

export interface DiffMessage {
  type: MessageType.Diff
  patches: DomPatch[]
  fingerprint: string
}

export interface CommandMessage {
  type: MessageType.Command
  commands: ClientCommand[]
}

export interface RedirectMessage {
  type: MessageType.Redirect
  url: string
  replace?: boolean
}

export interface ErrorMessage {
  type: MessageType.Error
  code: string
  message: string
}

export interface AckMessage {
  type: MessageType.Ack
  id: number
}

// DOM Patch Operations
export enum PatchOp {
  Replace = 1,
  Remove = 2,
  Insert = 3,
  Update = 4,
  SetAttr = 5,
  RemoveAttr = 6,
  AddClass = 7,
  RemoveClass = 8,
  SetProp = 9
}

export interface DomPatch {
  op: PatchOp
  path: number[] // Path to node as array of child indices
  data?: any
}

// Client Commands (like Phoenix.LiveView.JS)
export enum CommandType {
  Show = 1,
  Hide = 2,
  Toggle = 3,
  AddClass = 4,
  RemoveClass = 5,
  SetAttribute = 6,
  RemoveAttribute = 7,
  Dispatch = 8,
  Push = 9,
  Focus = 10,
  Blur = 11
}

export interface ClientCommand {
  type: CommandType
  target: string // CSS selector
  args?: any
  transition?: {
    duration?: number
    from?: Record<string, string>
    to?: Record<string, string>
  }
}

// Protocol helpers
export class Protocol {
  static encode(message: any): Uint8Array {
    return encode(message)
  }

  static decode(data: ArrayBuffer): any {
    return decode(data)
  }

  static createConnect(token?: string, params?: Record<string, any>): ConnectMessage {
    return { type: MessageType.Connect, token, params }
  }

  static createEvent(event: string, element?: string, value?: any, metadata?: Record<string, any>): EventMessage {
    return { type: MessageType.Event, event, element, value, metadata }
  }

  static createHeartbeat(): HeartbeatMessage {
    return { type: MessageType.Heartbeat, timestamp: Date.now() }
  }
}