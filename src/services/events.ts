import { listen, UnlistenFn } from '@tauri-apps/api/event';

export interface BackendEventPayload<T = unknown> {
  event_id: string;
  timestamp: string;
  entity_id: string;
  event_type: string;
  data: T;
}

export type EventCallback<T = unknown> = (payload: BackendEventPayload<T>) => void;

export async function subscribeToEvent<T = unknown>(
  eventName: string,
  callback: EventCallback<T>
): Promise<UnlistenFn> {
  if (typeof window === 'undefined' || !(window as any).__TAURI_INTERNALS__) {
    return () => {};
  }
  return listen<BackendEventPayload<T>>(eventName, (event) => {
    callback(event.payload);
  });
}
