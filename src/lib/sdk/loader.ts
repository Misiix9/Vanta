import type { ExtensionRegistration } from './types';

declare global {
  interface Window {
    __vanta_sdk?: Record<string, unknown>;
    __vanta_host?: {
      registerExtension: (name: string, registration: ExtensionRegistration) => void;
    };
  }
}

const registrations = new Map<string, ExtensionRegistration>();

/**
 * Initializes the global SDK and host objects on window.
 * Extensions use these to access SDK components and register themselves.
 */
export function setupHost(sdkComponents?: Record<string, unknown>): void {
  window.__vanta_sdk = sdkComponents ?? {};
  window.__vanta_host = {
    registerExtension(name: string, registration: ExtensionRegistration) {
      registrations.set(name, registration);
    },
  };
}

/**
 * Loads an extension by injecting its bundle as an inline script.
 * Returns the extension's registration if successful, null otherwise.
 * All loading is wrapped in try/catch to prevent extension crashes from affecting the host.
 * Module scripts run asynchronously, so we wait for the extension to call registerExtension.
 */
export async function loadExtension(
  extId: string,
  bundleCode: string,
  options?: { timeoutMs?: number }
): Promise<ExtensionRegistration | null> {
  try {
    const script = document.createElement('script');
    script.type = 'module';
    script.textContent = bundleCode;
    script.dataset.vantaExtensionId = extId;
    document.head.appendChild(script);

    const timeoutMs = options?.timeoutMs ?? 5000;
    const deadline = Date.now() + timeoutMs;
    while (Date.now() < deadline) {
      const reg = registrations.get(extId);
      if (reg) return reg;
      await new Promise((r) => setTimeout(r, 50));
    }
    return registrations.get(extId) ?? null;
  } catch (err) {
    console.error(`[Vanta] Failed to load extension ${extId}:`, err);
    return null;
  }
}

/**
 * Unloads an extension by removing its registration and cleaning up injected scripts.
 */
export function unloadExtension(extId: string): void {
  try {
    registrations.delete(extId);
    const script = document.querySelector(`script[data-vanta-extension-id="${extId}"]`);
    if (script?.parentNode) {
      script.parentNode.removeChild(script);
    }
  } catch (err) {
    console.error(`[Vanta] Failed to unload extension ${extId}:`, err);
  }
}

/**
 * Returns a loaded extension's registration, or undefined if not loaded.
 */
export function getRegistration(extId: string): ExtensionRegistration | undefined {
  return registrations.get(extId);
}
