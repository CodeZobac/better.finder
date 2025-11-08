export interface SearchResult {
  id: string;
  title: string;
  subtitle: string;
  icon: string | null;
  type: ResultType;
  score: number;
  metadata: Record<string, any>;
  action: ResultAction;
}

export enum ResultType {
  File = 'file',
  Application = 'application',
  QuickAction = 'quick_action',
  Calculator = 'calculator',
  Clipboard = 'clipboard',
  Bookmark = 'bookmark',
  RecentFile = 'recent_file',
  WebSearch = 'web_search',
}

export interface ResultAction {
  type: ActionType;
  payload: any;
}

export enum ActionType {
  OpenFile = 'open_file',
  LaunchApp = 'launch_app',
  ExecuteCommand = 'execute_command',
  CopyToClipboard = 'copy_to_clipboard',
  OpenUrl = 'open_url',
  WebSearch = 'web_search',
}

export interface AppSettings {
  hotkey: string;
  theme: Theme;
  max_results: number;
  enabled_providers: EnabledProviders;
  search_delay: number;
  start_with_windows: boolean;
}

export enum Theme {
  Light = 'light',
  Dark = 'dark',
  System = 'system',
}

export interface EnabledProviders {
  files: boolean;
  applications: boolean;
  quick_actions: boolean;
  calculator: boolean;
  clipboard: boolean;
  bookmarks: boolean;
  recent_files: boolean;
}
