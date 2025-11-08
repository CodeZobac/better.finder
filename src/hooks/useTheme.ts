import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Theme } from '../types';

export function useTheme() {
  const [theme, setTheme] = useState<Theme>(Theme.Dark);
  const [resolvedTheme, setResolvedTheme] = useState<'light' | 'dark'>('dark');

  // Load initial theme
  useEffect(() => {
    loadTheme();
  }, []);

  // Listen for theme changes
  useEffect(() => {
    const unlisten = listen<Theme>('theme-changed', (event) => {
      setTheme(event.payload);
      resolveAndApplyTheme(event.payload);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const loadTheme = async () => {
    try {
      const resolvedThemeValue = await invoke<'light' | 'dark'>('get_resolved_theme');
      setResolvedTheme(resolvedThemeValue);
      applyTheme(resolvedThemeValue);
    } catch (error) {
      console.error('Failed to load theme:', error);
      // Default to dark theme on error
      applyTheme('dark');
    }
  };

  const resolveAndApplyTheme = async (themeValue: Theme) => {
    try {
      const resolvedThemeValue = await invoke<'light' | 'dark'>('get_resolved_theme');
      setResolvedTheme(resolvedThemeValue);
      applyTheme(resolvedThemeValue);
    } catch (error) {
      console.error('Failed to resolve theme:', error);
      // Fallback: if theme is explicitly set, use it; otherwise default to dark
      const fallbackTheme = themeValue === Theme.Light ? 'light' : 'dark';
      applyTheme(fallbackTheme);
    }
  };

  const applyTheme = (themeValue: 'light' | 'dark') => {
    const root = document.documentElement;
    if (themeValue === 'dark') {
      root.setAttribute('data-theme', 'dark');
    } else {
      root.removeAttribute('data-theme');
    }
  };

  return {
    theme,
    resolvedTheme,
    refreshTheme: loadTheme,
  };
}
