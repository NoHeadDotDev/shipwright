/**
 * Hot Reload Client TypeScript Definitions
 * Enhanced with intelligent DOM patching, toast notifications, and state preservation
 */

export interface TemplateId {
  file: string;
  line: number;
  column: number;
}

export interface DynamicPart {
  index: number;
  kind: 'expression' | 'event_handler' | 'conditional' | 'loop';
}

export interface TemplateUpdate {
  id: TemplateId;
  hash: string;
  html: string;
  dynamic_parts: DynamicPart[];
}

export interface AssetUpdate {
  asset_type: 'css' | 'js' | 'other';
  path: string;
}

export interface FullReloadMessage {
  reason: string;
}

export interface BatchCompleteEvent {
  type: 'batch_complete';
  count: number;
}

export type UpdateEvent = TemplateUpdate | BatchCompleteEvent;
export type UpdateHandler = (update: UpdateEvent) => void;

export type ConnectionState = 'connecting' | 'connected' | 'disconnected' | 'error';

export type ToastType = 'info' | 'success' | 'warning' | 'error';

export interface HotReloadStats {
  connectionState: ConnectionState;
  reconnectAttempts: number;
  templatesLoaded: number;
  updateHandlers: number;
  queuedUpdates: number;
  isUpdating: boolean;
}

export interface HotReloadOptions {
  toastEnabled?: boolean;
  showIndicator?: boolean;
  enableDebugShortcuts?: boolean;
}

/**
 * DOM Diffing and Patching Engine
 */
export declare class DOMPatcher {
  constructor();
  patch(targetSelector: string, newHTML: string, templateId: TemplateId): boolean;
  findSelector(element: Element): string;
}

/**
 * Toast Notification System
 */
export declare class ToastNotifier {
  enabled: boolean;
  
  constructor();
  show(message: string, type?: ToastType, duration?: number): number;
  remove(toastId: number): void;
  clear(): void;
}

/**
 * State Preservation System
 */
export declare class StatePreserver {
  constructor();
  capture(): void;
  restore(): void;
  captureFormData(): void;
  restoreFormData(): void;
  captureScrollPositions(): void;
  restoreScrollPositions(): void;
  captureFocus(): void;
  restoreFocus(): void;
}

/**
 * Enhanced Hot Reload Client
 */
export declare class HotReloadClient {
  readonly domPatcher: DOMPatcher;
  readonly toastNotifier: ToastNotifier;
  readonly statePreserver: StatePreserver;
  
  constructor(url?: string);
  
  connect(): void;
  disconnect(): void;
  onUpdate(handler: UpdateHandler): () => void;
  requestReload(templateId: TemplateId): void;
  getTemplate(hash: string): TemplateUpdate | undefined;
  
  // Enhanced methods
  getConnectionState(): ConnectionState;
  isConnected(): boolean;
  setToastEnabled(enabled: boolean): void;
  getStats(): HotReloadStats;
  
  // Internal methods (exposed for advanced usage)
  handleTemplateUpdate(update: TemplateUpdate): void;
  handleBatchUpdate(message: { updates: TemplateUpdate[] }): void;
  handleAssetUpdate(update: AssetUpdate): void;
  handleFullReload(message: FullReloadMessage): void;
  processUpdateQueue(): Promise<void>;
  applyTemplateUpdate(update: TemplateUpdate): Promise<void>;
  applyAssetUpdate(update: AssetUpdate): Promise<void>;
  reloadCSS(cssPath: string): Promise<void>;
  
  static isAvailable(): boolean;
}

/**
 * Global functions
 */
export declare function initHotReload(url?: string, options?: HotReloadOptions): HotReloadClient | null;
export declare function getHotReloadClient(): HotReloadClient | null;
export declare function destroyHotReload(): void;