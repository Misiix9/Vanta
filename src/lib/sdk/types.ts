export type CommandMode = 'view' | 'no-view';

export interface ToastOptions {
  title: string;
  message?: string;
  type?: 'success' | 'error' | 'info';
}

export interface ExtensionEnvironment {
  isDevelopment: boolean;
  extensionPath: string;
  extensionName: string;
}

export interface VantaAPI {
  navigation: {
    push: (component: any, props?: Record<string, any>) => void;
    pop: () => void;
  };
  clipboard: {
    copy: (text: string) => Promise<void>;
  };
  network: {
    fetch: (url: string, options?: { method?: string }) => Promise<string>;
  };
  shell: {
    execute: (command: string, args?: string[]) => Promise<string>;
  };
  storage: {
    get: (key: string) => Promise<string | null>;
    set: (key: string, value: string) => Promise<void>;
  };
  window: {
    openMiniPlayer: () => Promise<void>;
  };
  toast: (options: ToastOptions) => void;
  closeMainWindow: () => Promise<void>;
  getPreference: <T>(key: string) => T | undefined;
  environment: ExtensionEnvironment;
}

export interface ListItemProps {
  id: string;
  title: string;
  subtitle?: string;
  icon?: string;
  accessories?: string[];
}

export interface FormFieldProps {
  id: string;
  label: string;
  type: 'text' | 'password' | 'dropdown' | 'checkbox' | 'date';
  placeholder?: string;
  options?: string[];
  required?: boolean;
  defaultValue?: string;
}

export interface GridItemProps {
  id: string;
  title: string;
  subtitle?: string;
  icon?: string;
  image?: string;
}

export interface DetailMetadata {
  label: string;
  value: string;
}

export interface ActionProps {
  id: string;
  title: string;
  icon?: string;
  shortcut?: string;
  onAction: () => void;
}

export interface ExtensionCommandRegistration {
  handler?: (api: VantaAPI) => Promise<void> | void;
  component?: any;
}

export interface ExtensionRegistration {
  commands: Record<string, ExtensionCommandRegistration>;
}
