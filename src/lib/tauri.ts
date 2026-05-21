import { invoke } from "@tauri-apps/api/core";
import { listen, type Event, type UnlistenFn } from "@tauri-apps/api/event";

export async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  return invoke<T>(cmd, args);
}

export async function on<T>(
  eventName: string,
  handler: (payload: T, event: Event<T>) => void,
): Promise<UnlistenFn> {
  return listen<T>(eventName, (event) => handler(event.payload, event));
}

export const events = {
  EmailNew: "email:new",
  EmailProcessed: "email:processed",
  AgentRun: "agent:run",
  SyncProgress: "sync:progress",
  OAuthCallback: "oauth:callback",
} as const;

export type AppEvent = (typeof events)[keyof typeof events];
