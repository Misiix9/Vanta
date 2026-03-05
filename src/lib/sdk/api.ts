import { invoke } from '@tauri-apps/api/core';
import type { VantaAPI, ToastOptions, ExtensionEnvironment } from './types';

type NavigationCallback = (component: any, props?: Record<string, any>) => void;
type PopCallback = () => void;
type ToastCallback = (options: ToastOptions) => void;

export function createVantaAPI(options: {
  extensionName: string;
  extensionPath: string;
  onPush: NavigationCallback;
  onPop: PopCallback;
  onToast: ToastCallback;
}): VantaAPI {
  const environment: ExtensionEnvironment = {
    isDevelopment: false, // controlled by config
    extensionPath: options.extensionPath,
    extensionName: options.extensionName,
  };

  return {
    navigation: {
      push: options.onPush,
      pop: options.onPop,
    },
    clipboard: {
      copy: async (text: string) => {
        await navigator.clipboard.writeText(text);
      },
    },
    network: {
      fetch: async (url: string, opts?: { method?: string }) => {
        return await invoke<string>('extension_fetch', {
          url,
          method: opts?.method ?? 'GET',
        });
      },
    },
    shell: {
      execute: async (command: string, args?: string[]) => {
        return await invoke<string>('extension_shell_execute', {
          command,
          args: args ?? [],
        });
      },
    },
    storage: {
      get: async (key: string) => {
        return await invoke<string | null>('extension_storage_get', {
          extId: options.extensionName,
          key,
        });
      },
      set: async (key: string, value: string) => {
        await invoke('extension_storage_set', {
          extId: options.extensionName,
          key,
          value,
        });
      },
    },
    window: {
      openMiniPlayer: async () => {
        await invoke('open_spotify_mini_player');
      },
    },
    toast: options.onToast,
    closeMainWindow: async () => {
      await invoke('hide_window');
    },
    getPreference: <T>(_key: string): T | undefined => {
      return undefined;
    },
    environment,
  };
}
