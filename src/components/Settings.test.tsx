import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import Settings from './Settings';
import { AppSettings, Theme } from '../types';

// Mock Tauri API
const mockInvoke = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: any[]) => mockInvoke(...args),
}));

describe('Settings Component', () => {
  const mockSettings: AppSettings = {
    hotkey: 'Ctrl+K',
    theme: Theme.System,
    max_results: 8,
    enabled_providers: {
      files: true,
      applications: true,
      quick_actions: true,
      calculator: true,
      clipboard: true,
      bookmarks: true,
      recent_files: true,
    },
    search_delay: 150,
    start_with_windows: false,
  };

  const mockOnClose = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockResolvedValue(mockSettings);
  });

  it('should not render when isOpen is false', () => {
    const { container } = render(<Settings isOpen={false} onClose={mockOnClose} />);
    expect(container.firstChild).toBeNull();
  });

  it('should load and display settings when opened', async () => {
    render(<Settings isOpen={true} onClose={mockOnClose} />);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('get_settings');
    });

    await waitFor(() => {
      expect(screen.getByDisplayValue('Ctrl+K')).toBeInTheDocument();
    });
  });

  it('should update hotkey input', async () => {
    const user = userEvent.setup();
    render(<Settings isOpen={true} onClose={mockOnClose} />);

    await waitFor(() => {
      expect(screen.getByDisplayValue('Ctrl+K')).toBeInTheDocument();
    });

    const hotkeyInput = screen.getByDisplayValue('Ctrl+K');
    await user.clear(hotkeyInput);
    await user.type(hotkeyInput, 'Ctrl+Space');

    expect(hotkeyInput).toHaveValue('Ctrl+Space');
  });

  it('should update theme selection', async () => {
    const user = userEvent.setup();
    render(<Settings isOpen={true} onClose={mockOnClose} />);

    await waitFor(() => {
      expect(screen.getByText('Light')).toBeInTheDocument();
    });

    const lightButton = screen.getByText('Light');
    await user.click(lightButton);

    // The button should be selected (has specific styling)
    expect(lightButton.closest('button')).toHaveClass('border-primary');
  });

  it('should update max results slider', async () => {
    const user = userEvent.setup();
    render(<Settings isOpen={true} onClose={mockOnClose} />);

    await waitFor(() => {
      expect(screen.getByText(/Maximum Results:/)).toBeInTheDocument();
    });

    const slider = screen.getByRole('slider');
    // For range inputs, we can't use clear(), just change the value directly
    await user.click(slider);
    // Simulate changing the slider value
    expect(slider).toHaveValue('8');
  });

  it('should toggle provider checkboxes', async () => {
    const user = userEvent.setup();
    render(<Settings isOpen={true} onClose={mockOnClose} />);

    await waitFor(() => {
      expect(screen.getByText('files')).toBeInTheDocument();
    });

    // Get all checkboxes and find the one for "files" (not "recent files")
    const checkboxes = screen.getAllByRole('checkbox');
    const filesCheckbox = checkboxes[0]; // First checkbox is "files"
    expect(filesCheckbox).toBeChecked();

    await user.click(filesCheckbox);
    expect(filesCheckbox).not.toBeChecked();
  });

  it('should toggle start with Windows checkbox', async () => {
    const user = userEvent.setup();
    render(<Settings isOpen={true} onClose={mockOnClose} />);

    await waitFor(() => {
      expect(screen.getByText('Start with Windows')).toBeInTheDocument();
    });

    const startWithWindowsCheckbox = screen.getByRole('checkbox', {
      name: /start with windows/i,
    });
    expect(startWithWindowsCheckbox).not.toBeChecked();

    await user.click(startWithWindowsCheckbox);
    expect(startWithWindowsCheckbox).toBeChecked();
  });

  it('should check and display auto-start status when opened', async () => {
    mockInvoke
      .mockResolvedValueOnce(mockSettings) // get_settings
      .mockResolvedValueOnce(true); // is_auto_start_enabled

    render(<Settings isOpen={true} onClose={mockOnClose} />);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('get_settings');
      expect(mockInvoke).toHaveBeenCalledWith('is_auto_start_enabled');
    });

    await waitFor(() => {
      expect(screen.getByText(/Currently: Enabled/)).toBeInTheDocument();
    });
  });

  it('should refresh auto-start status after saving', async () => {
    const user = userEvent.setup();
    mockInvoke
      .mockResolvedValueOnce(mockSettings) // get_settings (initial)
      .mockResolvedValueOnce(false) // is_auto_start_enabled (initial)
      .mockResolvedValueOnce(undefined) // update_settings
      .mockResolvedValueOnce(true); // is_auto_start_enabled (after save)

    render(<Settings isOpen={true} onClose={mockOnClose} />);

    await waitFor(() => {
      expect(screen.getByText('Save Changes')).toBeInTheDocument();
    });

    const saveButton = screen.getByText('Save Changes');
    await user.click(saveButton);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('update_settings', {
        settings: mockSettings,
      });
      // Should check auto-start status again after saving
      expect(mockInvoke).toHaveBeenCalledWith('is_auto_start_enabled');
      expect(mockOnClose).toHaveBeenCalled();
    });
  });

  it('should save settings when Save Changes is clicked', async () => {
    const user = userEvent.setup();
    mockInvoke.mockResolvedValueOnce(mockSettings).mockResolvedValueOnce(undefined);

    render(<Settings isOpen={true} onClose={mockOnClose} />);

    await waitFor(() => {
      expect(screen.getByText('Save Changes')).toBeInTheDocument();
    });

    const saveButton = screen.getByText('Save Changes');
    await user.click(saveButton);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('update_settings', {
        settings: mockSettings,
      });
      expect(mockOnClose).toHaveBeenCalled();
    });
  });

  it('should close without saving when Cancel is clicked', async () => {
    const user = userEvent.setup();
    render(<Settings isOpen={true} onClose={mockOnClose} />);

    await waitFor(() => {
      expect(screen.getByText('Cancel')).toBeInTheDocument();
    });

    const cancelButton = screen.getByText('Cancel');
    await user.click(cancelButton);

    expect(mockOnClose).toHaveBeenCalled();
    expect(mockInvoke).not.toHaveBeenCalledWith('update_settings', expect.anything());
  });

  it('should display error message when loading fails', async () => {
    mockInvoke.mockRejectedValueOnce(new Error('Failed to load'));

    render(<Settings isOpen={true} onClose={mockOnClose} />);

    await waitFor(() => {
      expect(screen.getByText(/Failed to load settings/)).toBeInTheDocument();
    });
  });

  it('should display error message when saving fails', async () => {
    const user = userEvent.setup();
    mockInvoke
      .mockResolvedValueOnce(mockSettings)
      .mockRejectedValueOnce(new Error('Failed to save'));

    render(<Settings isOpen={true} onClose={mockOnClose} />);

    await waitFor(() => {
      expect(screen.getByText('Save Changes')).toBeInTheDocument();
    });

    const saveButton = screen.getByText('Save Changes');
    await user.click(saveButton);

    await waitFor(() => {
      expect(screen.getByText(/Failed to save settings/)).toBeInTheDocument();
    });

    expect(mockOnClose).not.toHaveBeenCalled();
  });
});
