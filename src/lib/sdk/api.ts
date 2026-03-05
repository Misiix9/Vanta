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
    toast: options.onToast,
    closeMainWindow: async () => {
      await invoke('hide_window');
    },
    getPreference: <T>(_key: string): T | undefined => {
      return undefined; // Extension preferences TBD
    },
    environment,
  };
}
