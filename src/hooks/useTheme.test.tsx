import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { useTheme } from './useTheme';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Theme } from '../types';

vi.mock('@tauri-apps/api/core');
vi.mock('@tauri-apps/api/event');

describe('useTheme', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Reset document theme
    document.documentElement.removeAttribute('data-theme');
  });

  it('should load and apply dark theme by default', async () => {
    vi.mocked(invoke).mockResolvedValue('dark');
    vi.mocked(listen).mockResolvedValue(() => {});

    const { result } = renderHook(() => useTheme());

    await waitFor(() => {
      expect(result.current.resolvedTheme).toBe('dark');
    });

    expect(document.documentElement.getAttribute('data-theme')).toBe('dark');
  });

  it('should load and apply light theme', async () => {
    vi.mocked(invoke).mockResolvedValue('light');
    vi.mocked(listen).mockResolvedValue(() => {});

    const { result } = renderHook(() => useTheme());

    await waitFor(() => {
      expect(result.current.resolvedTheme).toBe('light');
    });

    expect(document.documentElement.hasAttribute('data-theme')).toBe(false);
  });

  it('should handle theme changes via event', async () => {
    let eventCallback: ((event: any) => void) | undefined;
    
    vi.mocked(invoke).mockResolvedValue('dark');
    vi.mocked(listen).mockImplementation((event, callback) => {
      if (event === 'theme-changed') {
        eventCallback = callback as any;
      }
      return Promise.resolve(() => {});
    });

    const { result } = renderHook(() => useTheme());

    await waitFor(() => {
      expect(result.current.resolvedTheme).toBe('dark');
    });

    // Simulate theme change event
    vi.mocked(invoke).mockResolvedValue('light');
    if (eventCallback) {
      eventCallback({ payload: Theme.Light });
    }

    await waitFor(() => {
      expect(result.current.resolvedTheme).toBe('light');
    });

    expect(document.documentElement.hasAttribute('data-theme')).toBe(false);
  });

  it('should apply dark theme to document root', async () => {
    vi.mocked(invoke).mockResolvedValue('dark');
    vi.mocked(listen).mockResolvedValue(() => {});

    renderHook(() => useTheme());

    await waitFor(() => {
      expect(document.documentElement.getAttribute('data-theme')).toBe('dark');
    });
  });

  it('should remove data-theme attribute for light theme', async () => {
    // Start with dark theme
    document.documentElement.setAttribute('data-theme', 'dark');
    
    vi.mocked(invoke).mockResolvedValue('light');
    vi.mocked(listen).mockResolvedValue(() => {});

    renderHook(() => useTheme());

    await waitFor(() => {
      expect(document.documentElement.hasAttribute('data-theme')).toBe(false);
    });
  });

  it('should handle errors gracefully and default to dark theme', async () => {
    vi.mocked(invoke).mockRejectedValue(new Error('Failed to load theme'));
    vi.mocked(listen).mockResolvedValue(() => {});

    renderHook(() => useTheme());

    await waitFor(() => {
      expect(document.documentElement.getAttribute('data-theme')).toBe('dark');
    });
  });

  it('should refresh theme when refreshTheme is called', async () => {
    vi.mocked(invoke).mockResolvedValue('dark');
    vi.mocked(listen).mockResolvedValue(() => {});

    const { result } = renderHook(() => useTheme());

    await waitFor(() => {
      expect(result.current.resolvedTheme).toBe('dark');
    });

    // Change the mock to return light theme
    vi.mocked(invoke).mockResolvedValue('light');

    // Call refreshTheme
    result.current.refreshTheme();

    await waitFor(() => {
      expect(result.current.resolvedTheme).toBe('light');
    });

    expect(document.documentElement.hasAttribute('data-theme')).toBe(false);
  });
});
