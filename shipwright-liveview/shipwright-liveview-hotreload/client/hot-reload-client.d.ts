/**
 * Hot Reload Client TypeScript Definitions
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

export type UpdateHandler = (update: TemplateUpdate) => void;

export declare class HotReloadClient {
  constructor(url?: string);
  
  connect(): void;
  disconnect(): void;
  onUpdate(handler: UpdateHandler): () => void;
  requestReload(templateId: TemplateId): void;
  getTemplate(hash: string): TemplateUpdate | undefined;
  
  static isAvailable(): boolean;
}

export declare function initHotReload(url?: string): HotReloadClient | null;
export declare function getHotReloadClient(): HotReloadClient | null;