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
 * Loads an extension by injecting its bundle as a classic inline script.
 * Classic scripts execute synchronously, so the IIFE registers immediately.
 * Falls back to a polling loop in case of async registration.
 */
export async function loadExtension(
  extId: string,
  bundleCode: string,
  options?: { timeoutMs?: number }
): Promise<ExtensionRegistration | null> {
  let scriptError: string | null = null;

  const errorHandler = (ev: ErrorEvent) => {
    scriptError = ev.message;
  };
  window.addEventListener('error', errorHandler);

  try {
    const script = document.createElement('script');
    script.dataset.vantaExtensionId = extId;
    script.textContent = bundleCode;
    script.onerror = () => { scriptError = 'Script failed to load'; };
    document.head.appendChild(script);

    const immediate = registrations.get(extId);
    if (immediate) return immediate;

    if (scriptError) {
      console.error(`[Vanta] Extension ${extId} threw during load:`, scriptError);
      return null;
    }

    const timeoutMs = options?.timeoutMs ?? 5000;
    const deadline = Date.now() + timeoutMs;
    while (Date.now() < deadline) {
      const reg = registrations.get(extId);
      if (reg) return reg;
      if (scriptError) {
        console.error(`[Vanta] Extension ${extId} error:`, scriptError);
        return null;
      }
      await new Promise((r) => setTimeout(r, 50));
    }
    return registrations.get(extId) ?? null;
  } catch (err) {
    console.error(`[Vanta] Failed to load extension ${extId}:`, err);
    return null;
  } finally {
    window.removeEventListener('error', errorHandler);
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
