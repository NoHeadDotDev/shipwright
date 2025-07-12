// Minimal MessagePack implementation for LiveView
// Supports only the types we need to keep bundle size small

export function encode(value: any): Uint8Array {
  const buffer: number[] = []
  encodeValue(value, buffer)
  return new Uint8Array(buffer)
}

export function decode(data: ArrayBuffer): any {
  const view = new DataView(data)
  const result = decodeValue(view, 0)
  return result.value
}

function encodeValue(value: any, buffer: number[]): void {
  if (value === null) {
    buffer.push(0xc0)
  } else if (value === false) {
    buffer.push(0xc2)
  } else if (value === true) {
    buffer.push(0xc3)
  } else if (typeof value === 'number') {
    encodeNumber(value, buffer)
  } else if (typeof value === 'string') {
    encodeString(value, buffer)
  } else if (value instanceof Uint8Array) {
    encodeBinary(value, buffer)
  } else if (Array.isArray(value)) {
    encodeArray(value, buffer)
  } else if (typeof value === 'object') {
    encodeObject(value, buffer)
  }
}

function encodeNumber(value: number, buffer: number[]): void {
  if (Number.isInteger(value)) {
    if (value >= 0 && value <= 127) {
      buffer.push(value)
    } else if (value >= -32 && value < 0) {
      buffer.push(0xe0 | (value + 32))
    } else if (value >= -128 && value <= 127) {
      buffer.push(0xd0, value & 0xff)
    } else if (value >= -32768 && value <= 32767) {
      buffer.push(0xd1, (value >> 8) & 0xff, value & 0xff)
    } else {
      buffer.push(0xd2)
      buffer.push((value >> 24) & 0xff)
      buffer.push((value >> 16) & 0xff)
      buffer.push((value >> 8) & 0xff)
      buffer.push(value & 0xff)
    }
  } else {
    // Float
    const view = new DataView(new ArrayBuffer(8))
    view.setFloat64(0, value)
    buffer.push(0xcb)
    for (let i = 0; i < 8; i++) {
      buffer.push(view.getUint8(i))
    }
  }
}

function encodeString(value: string, buffer: number[]): void {
  const encoded = new TextEncoder().encode(value)
  const len = encoded.length
  
  if (len <= 31) {
    buffer.push(0xa0 | len)
  } else if (len <= 255) {
    buffer.push(0xd9, len)
  } else if (len <= 65535) {
    buffer.push(0xda, (len >> 8) & 0xff, len & 0xff)
  }
  
  for (const byte of encoded) {
    buffer.push(byte)
  }
}

function encodeBinary(value: Uint8Array, buffer: number[]): void {
  const len = value.length
  
  if (len <= 255) {
    buffer.push(0xc4, len)
  } else if (len <= 65535) {
    buffer.push(0xc5, (len >> 8) & 0xff, len & 0xff)
  }
  
  for (const byte of value) {
    buffer.push(byte)
  }
}

function encodeArray(value: any[], buffer: number[]): void {
  const len = value.length
  
  if (len <= 15) {
    buffer.push(0x90 | len)
  } else if (len <= 65535) {
    buffer.push(0xdc, (len >> 8) & 0xff, len & 0xff)
  }
  
  for (const item of value) {
    encodeValue(item, buffer)
  }
}

function encodeObject(value: Record<string, any>, buffer: number[]): void {
  const keys = Object.keys(value)
  const len = keys.length
  
  if (len <= 15) {
    buffer.push(0x80 | len)
  } else if (len <= 65535) {
    buffer.push(0xde, (len >> 8) & 0xff, len & 0xff)
  }
  
  for (const key of keys) {
    encodeString(key, buffer)
    encodeValue(value[key], buffer)
  }
}

function decodeValue(view: DataView, offset: number): { value: any, offset: number } {
  const byte = view.getUint8(offset)
  
  if (byte === 0xc0) return { value: null, offset: offset + 1 }
  if (byte === 0xc2) return { value: false, offset: offset + 1 }
  if (byte === 0xc3) return { value: true, offset: offset + 1 }
  
  // Positive fixint
  if ((byte & 0x80) === 0) {
    return { value: byte, offset: offset + 1 }
  }
  
  // Negative fixint
  if ((byte & 0xe0) === 0xe0) {
    return { value: byte - 256, offset: offset + 1 }
  }
  
  // Fixstr
  if ((byte & 0xe0) === 0xa0) {
    const len = byte & 0x1f
    return decodeString(view, offset + 1, len)
  }
  
  // Fixarray
  if ((byte & 0xf0) === 0x90) {
    const len = byte & 0x0f
    return decodeArray(view, offset + 1, len)
  }
  
  // Fixmap
  if ((byte & 0xf0) === 0x80) {
    const len = byte & 0x0f
    return decodeObject(view, offset + 1, len)
  }
  
  // Other types
  switch (byte) {
    case 0xd0: // int8
      return { value: view.getInt8(offset + 1), offset: offset + 2 }
    case 0xd1: // int16
      return { value: view.getInt16(offset + 1), offset: offset + 3 }
    case 0xd2: // int32
      return { value: view.getInt32(offset + 1), offset: offset + 5 }
    case 0xcb: // float64
      return { value: view.getFloat64(offset + 1), offset: offset + 9 }
    case 0xd9: // str8
      const strLen8 = view.getUint8(offset + 1)
      return decodeString(view, offset + 2, strLen8)
    case 0xda: // str16
      const strLen16 = view.getUint16(offset + 1)
      return decodeString(view, offset + 3, strLen16)
    case 0xdc: // array16
      const arrLen16 = view.getUint16(offset + 1)
      return decodeArray(view, offset + 3, arrLen16)
    case 0xde: // map16
      const mapLen16 = view.getUint16(offset + 1)
      return decodeObject(view, offset + 3, mapLen16)
    case 0xc4: // bin8
      const binLen8 = view.getUint8(offset + 1)
      return decodeBinary(view, offset + 2, binLen8)
    case 0xc5: // bin16
      const binLen16 = view.getUint16(offset + 1)
      return decodeBinary(view, offset + 3, binLen16)
    default:
      throw new Error(`Unsupported MessagePack type: 0x${byte.toString(16)}`)
  }
}

function decodeString(view: DataView, offset: number, length: number): { value: string, offset: number } {
  const bytes = new Uint8Array(length)
  for (let i = 0; i < length; i++) {
    bytes[i] = view.getUint8(offset + i)
  }
  const value = new TextDecoder().decode(bytes)
  return { value, offset: offset + length }
}

function decodeArray(view: DataView, offset: number, length: number): { value: any[], offset: number } {
  const arr: any[] = []
  let currentOffset = offset
  
  for (let i = 0; i < length; i++) {
    const result = decodeValue(view, currentOffset)
    arr.push(result.value)
    currentOffset = result.offset
  }
  
  return { value: arr, offset: currentOffset }
}

function decodeObject(view: DataView, offset: number, length: number): { value: Record<string, any>, offset: number } {
  const obj: Record<string, any> = {}
  let currentOffset = offset
  
  for (let i = 0; i < length; i++) {
    const keyResult = decodeValue(view, currentOffset)
    currentOffset = keyResult.offset
    
    const valueResult = decodeValue(view, currentOffset)
    currentOffset = valueResult.offset
    
    obj[keyResult.value] = valueResult.value
  }
  
  return { value: obj, offset: currentOffset }
}

function decodeBinary(view: DataView, offset: number, length: number): { value: Uint8Array, offset: number } {
  const bytes = new Uint8Array(length)
  for (let i = 0; i < length; i++) {
    bytes[i] = view.getUint8(offset + i)
  }
  return { value: bytes, offset: offset + length }
}