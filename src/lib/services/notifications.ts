import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";

let permissionGranted = false;

export async function initNotifications(): Promise<void> {
  try {
    permissionGranted = await isPermissionGranted();
    if (!permissionGranted) {
      const permission = await requestPermission();
      permissionGranted = permission === "granted";
    }
  } catch (e) {
    console.warn("[notifications] Failed to init:", e);
  }
}

export function notify(title: string, body?: string): void {
  if (!permissionGranted) return;
  try {
    sendNotification({ title, body });
  } catch (e) {
    console.warn("[notifications] Failed to send:", e);
  }
}

// Convenience helpers for common events
export function notifyAgentComplete(agentName: string): void {
  notify("Agent Run Complete", `${agentName} has finished processing.`);
}

export function notifyBatchComplete(jobId: string): void {
  notify("Batch Job Complete", `Job ${jobId} has finished.`);
}

export function notifyFineTuningComplete(model: string): void {
  notify("Fine-Tuning Complete", `Model ${model} is ready.`);
}

export function notifyConnectionLost(): void {
  notify("Connection Lost", "Azure connection was interrupted.");
}

export function notifyConnectionRestored(): void {
  notify("Connection Restored", "Azure connection is back online.");
}
