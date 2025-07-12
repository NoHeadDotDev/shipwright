export declare interface ClientCommand {
    type: CommandType;
    target: string;
    args?: any;
    transition?: {
        duration?: number;
        from?: Record<string, string>;
        to?: Record<string, string>;
    };
}

export declare enum CommandType {
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

declare interface ConnectionOptions {
    url: string;
    reconnectInterval?: number;
    maxReconnectAttempts?: number;
    heartbeatInterval?: number;
    onOpen?: () => void;
    onClose?: () => void;
    onError?: (error: Event) => void;
    onMessage?: (message: any) => void;
}

declare interface ConnectMessage {
    type: MessageType.Connect;
    token?: string;
    params?: Record<string, any>;
}

declare interface EventMessage {
    type: MessageType.Event;
    event: string;
    element?: string;
    value?: any;
    metadata?: Record<string, any>;
}

declare interface HeartbeatMessage {
    type: MessageType.Heartbeat;
    timestamp: number;
}

export declare class LiveView {
    private container;
    private connection;
    private domPatcher;
    private eventDelegator;
    private commandExecutor;
    private formRecovery;
    private _fingerprint;
    private options;
    get fingerprint(): string;
    constructor(options: LiveViewOptions);
    connect(): void;
    disconnect(): void;
    pushEvent(event: string, payload?: any, callback?: (reply: any) => void): void;
    private handleMessage;
    private handleRender;
    private handleDiff;
    private handleCommand;
    private handleRedirect;
    private bindEvents;
    private getSelector;
    private parseEventConfig;
    private loadAssets;
    private cleanup;
}

export declare class LiveViewConnection {
    ws: WebSocket | null;
    private options;
    private reconnectAttempts;
    private reconnectTimer;
    private heartbeatTimer;
    private messageQueue;
    private isConnecting;
    constructor(options: ConnectionOptions);
    connect(token?: string, params?: Record<string, any>): void;
    disconnect(): void;
    send(data: Uint8Array): void;
    private handleMessage;
    private startHeartbeat;
    private stopHeartbeat;
    private scheduleReconnect;
    private stopReconnect;
    get readyState(): number;
    get isConnected(): boolean;
}

export declare interface LiveViewOptions {
    url: string;
    container: string | Element;
    token?: string;
    params?: Record<string, any>;
    reconnectInterval?: number;
    maxReconnectAttempts?: number;
    heartbeatInterval?: number;
    onConnect?: () => void;
    onDisconnect?: () => void;
    onError?: (error: any) => void;
}

export declare enum MessageType {
    Connect = 1,
    Event = 2,
    Heartbeat = 3,
    Render = 16,
    Diff = 17,
    Redirect = 18,
    Command = 19,
    Error = 20,
    Ack = 21
}

export declare class Protocol {
    static encode(message: any): Uint8Array;
    static decode(data: ArrayBuffer): any;
    static createConnect(token?: string, params?: Record<string, any>): ConnectMessage;
    static createEvent(event: string, element?: string, value?: any, metadata?: Record<string, any>): EventMessage;
    static createHeartbeat(): HeartbeatMessage;
}

export { }
