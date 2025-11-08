import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { AppSettings, Theme } from '../types';
import { X, Settings as SettingsIcon } from 'lucide-react';

interface SettingsProps {
  isOpen: boolean;
  onClose: () => void;
}

const Settings: React.FC<SettingsProps> = ({ isOpen, onClose }) => {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [autoStartStatus, setAutoStartStatus] = useState<boolean | null>(null);

  useEffect(() => {
    if (isOpen) {
      loadSettings();
      checkAutoStartStatus();
    }
  }, [isOpen]);

  const loadSettings = async () => {
    try {
      setLoading(true);
      setError(null);
      const loadedSettings = await invoke<AppSettings>('get_settings');
      setSettings(loadedSettings);
    } catch (err) {
      setError(`Failed to load settings: ${err}`);
      console.error('Failed to load settings:', err);
    } finally {
      setLoading(false);
    }
  };

  const checkAutoStartStatus = async () => {
    try {
      const status = await invoke<boolean>('is_auto_start_enabled');
      setAutoStartStatus(status);
    } catch (err) {
      console.error('Failed to check auto-start status:', err);
      setAutoStartStatus(null);
    }
  };

  const saveSettings = async () => {
    if (!settings) return;

    try {
      setSaving(true);
      setError(null);
      await invoke('update_settings', { settings });
      // Refresh auto-start status after saving
      await checkAutoStartStatus();
      onClose();
    } catch (err) {
      setError(`Failed to save settings: ${err}`);
      console.error('Failed to save settings:', err);
    } finally {
      setSaving(false);
    }
  };

  const updateSetting = <K extends keyof AppSettings>(
    key: K,
    value: AppSettings[K]
  ) => {
    if (settings) {
      setSettings({ ...settings, [key]: value });
    }
  };

  const updateProvider = (provider: keyof AppSettings['enabled_providers'], enabled: boolean) => {
    if (settings) {
      setSettings({
        ...settings,
        enabled_providers: {
          ...settings.enabled_providers,
          [provider]: enabled,
        },
      });
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
      <div className="bg-surface rounded-xl shadow-2xl w-full max-w-2xl max-h-[90vh] overflow-hidden flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-border">
          <div className="flex items-center gap-3">
            <SettingsIcon className="w-6 h-6 text-primary" />
            <h2 className="text-2xl font-semibold text-text-primary">Settings</h2>
          </div>
          <button
            onClick={onClose}
            className="p-2 hover:bg-surface-hover rounded-lg transition-colors"
            aria-label="Close settings"
          >
            <X className="w-5 h-5 text-text-secondary" />
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-6">
          {loading ? (
            <div className="flex items-center justify-center py-12">
              <div className="text-text-secondary">Loading settings...</div>
            </div>
          ) : error ? (
            <div className="bg-red-50 border border-red-200 rounded-lg p-4 text-red-700">
              {error}
            </div>
          ) : settings ? (
            <div className="space-y-6">
              {/* Hotkey */}
              <div>
                <label className="block text-sm font-medium text-text-primary mb-2">
                  Global Hotkey
                </label>
                <input
                  type="text"
                  value={settings.hotkey}
                  onChange={(e) => updateSetting('hotkey', e.target.value)}
                  className="w-full px-4 py-2 border border-border rounded-lg bg-background text-text-primary focus:ring-2 focus:ring-primary focus:border-transparent"
                  placeholder="e.g., Ctrl+K"
                />
                <p className="mt-1 text-sm text-text-secondary">
                  Press this key combination to open the search bar
                </p>
              </div>

              {/* Theme */}
              <div>
                <label className="block text-sm font-medium text-text-primary mb-2">
                  Theme
                </label>
                <div className="grid grid-cols-3 gap-3">
                  {(['light', 'dark', 'system'] as const).map((theme) => (
                    <button
                      key={theme}
                      onClick={() => updateSetting('theme', theme as Theme)}
                      className={`px-4 py-2 rounded-lg border-2 transition-all ${
                        settings.theme === theme
                          ? 'border-primary bg-primary/10 text-primary'
                          : 'border-border hover:border-primary/50 text-text-primary'
                      }`}
                    >
                      {theme.charAt(0).toUpperCase() + theme.slice(1)}
                    </button>
                  ))}
                </div>
              </div>

              {/* Max Results */}
              <div>
                <label className="block text-sm font-medium text-text-primary mb-2">
                  Maximum Results: {settings.max_results}
                </label>
                <input
                  type="range"
                  min="3"
                  max="20"
                  value={settings.max_results}
                  onChange={(e) => updateSetting('max_results', parseInt(e.target.value))}
                  className="w-full h-2 bg-surface-hover rounded-lg appearance-none cursor-pointer accent-primary"
                />
                <div className="flex justify-between text-xs text-text-secondary mt-1">
                  <span>3</span>
                  <span>20</span>
                </div>
              </div>

              {/* Search Providers */}
              <div>
                <label className="block text-sm font-medium text-text-primary mb-3">
                  Search Providers
                </label>
                <div className="space-y-2">
                  {Object.entries(settings.enabled_providers).map(([key, enabled]) => (
                    <label
                      key={key}
                      className="flex items-center justify-between p-3 rounded-lg hover:bg-surface-hover cursor-pointer"
                    >
                      <span className="text-text-primary capitalize">
                        {key.replace(/_/g, ' ')}
                      </span>
                      <input
                        type="checkbox"
                        checked={enabled}
                        onChange={(e) =>
                          updateProvider(
                            key as keyof AppSettings['enabled_providers'],
                            e.target.checked
                          )
                        }
                        className="w-5 h-5 text-primary bg-background border-border rounded focus:ring-primary focus:ring-2"
                      />
                    </label>
                  ))}
                </div>
              </div>

              {/* Start with Windows */}
              <div>
                <label className="flex items-center justify-between p-3 rounded-lg hover:bg-surface-hover cursor-pointer">
                  <div>
                    <div className="text-text-primary font-medium">
                      Start with Windows
                    </div>
                    <div className="text-sm text-text-secondary">
                      Launch Better Finder automatically when Windows starts
                      {autoStartStatus !== null && (
                        <span className="ml-2">
                          (Currently: {autoStartStatus ? 'Enabled' : 'Disabled'})
                        </span>
                      )}
                    </div>
                  </div>
                  <input
                    type="checkbox"
                    checked={settings.start_with_windows}
                    onChange={(e) => updateSetting('start_with_windows', e.target.checked)}
                    className="w-5 h-5 text-primary bg-background border-border rounded focus:ring-primary focus:ring-2"
                  />
                </label>
              </div>
            </div>
          ) : null}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-end gap-3 p-6 border-t border-border">
          <button
            onClick={onClose}
            className="px-4 py-2 text-text-primary hover:bg-surface-hover rounded-lg transition-colors"
            disabled={saving}
          >
            Cancel
          </button>
          <button
            onClick={saveSettings}
            disabled={saving || !settings}
            className="px-6 py-2 bg-primary hover:opacity-90 disabled:opacity-50 text-white rounded-lg transition-all font-medium"
          >
            {saving ? 'Saving...' : 'Save Changes'}
          </button>
        </div>
      </div>
    </div>
  );
};

export default Settings;
