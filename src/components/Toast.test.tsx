import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import Toast, { ToastMessage } from './Toast';

describe('Toast Component', () => {
  let mockOnClose: (id: string) => void;

  beforeEach(() => {
    mockOnClose = vi.fn() as (id: string) => void;
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.restoreAllMocks();
    vi.useRealTimers();
  });

  const createToast = (type: ToastMessage['type'], title: string, message?: string): ToastMessage => ({
    id: 'test-toast-1',
    type,
    title,
    message,
    duration: 5000,
  });

  it('renders success toast with title', () => {
    const toast = createToast('success', 'Success!');
    render(<Toast toast={toast} onClose={mockOnClose} />);

    expect(screen.getByText('Success!')).toBeInTheDocument();
    expect(screen.getByRole('alert')).toBeInTheDocument();
  });

  it('renders error toast with title and message', () => {
    const toast = createToast('error', 'Error occurred', 'Something went wrong');
    render(<Toast toast={toast} onClose={mockOnClose} />);

    expect(screen.getByText('Error occurred')).toBeInTheDocument();
    expect(screen.getByText('Something went wrong')).toBeInTheDocument();
  });

  it('renders warning toast', () => {
    const toast = createToast('warning', 'Warning!');
    render(<Toast toast={toast} onClose={mockOnClose} />);

    expect(screen.getByText('Warning!')).toBeInTheDocument();
  });

  it('renders info toast', () => {
    const toast = createToast('info', 'Information');
    render(<Toast toast={toast} onClose={mockOnClose} />);

    expect(screen.getByText('Information')).toBeInTheDocument();
  });

  it('calls onClose when close button is clicked', async () => {
    const user = userEvent.setup({ delay: null });
    const toast = createToast('success', 'Test');
    render(<Toast toast={toast} onClose={mockOnClose} />);

    const closeButton = screen.getByLabelText('Close notification');
    await user.click(closeButton);

    // Wait for animation
    vi.advanceTimersByTime(300);

    await waitFor(() => {
      expect(mockOnClose).toHaveBeenCalledWith('test-toast-1');
    });
  });

  it('auto-closes after duration', async () => {
    const toast = createToast('success', 'Test');
    render(<Toast toast={toast} onClose={mockOnClose} />);

    // Fast-forward time by duration
    vi.advanceTimersByTime(5000);

    // Wait for animation
    vi.advanceTimersByTime(300);

    await waitFor(() => {
      expect(mockOnClose).toHaveBeenCalledWith('test-toast-1');
    });
  });

  it('respects custom duration', async () => {
    const toast: ToastMessage = {
      id: 'test-toast-2',
      type: 'info',
      title: 'Custom duration',
      duration: 2000,
    };
    render(<Toast toast={toast} onClose={mockOnClose} />);

    // Fast-forward time by custom duration
    vi.advanceTimersByTime(2000);

    // Wait for animation
    vi.advanceTimersByTime(300);

    await waitFor(() => {
      expect(mockOnClose).toHaveBeenCalledWith('test-toast-2');
    });
  });

  it('displays correct icon for each toast type', () => {
    const types: Array<ToastMessage['type']> = ['success', 'error', 'warning', 'info'];

    types.forEach((type) => {
      const toast = createToast(type, `${type} toast`);
      const { container, unmount } = render(<Toast toast={toast} onClose={mockOnClose} />);

      // Check that an icon is rendered (lucide-react icons have specific classes)
      const icon = container.querySelector('svg');
      expect(icon).toBeInTheDocument();

      unmount();
    });
  });
});
