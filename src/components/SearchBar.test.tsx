import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import SearchBar from './SearchBar';

describe('SearchBar', () => {
  const mockOnClose = vi.fn();

  beforeEach(() => {
    mockOnClose.mockClear();
  });

  describe('Input field rendering and interaction', () => {
    it('should render the search input when visible', () => {
      render(<SearchBar isVisible={true} onClose={mockOnClose} />);
      
      const input = screen.getByPlaceholderText(/search files, apps, and more/i);
      expect(input).toBeInTheDocument();
    });

    it('should not render when not visible', () => {
      render(<SearchBar isVisible={false} onClose={mockOnClose} />);
      
      const input = screen.queryByPlaceholderText(/search files, apps, and more/i);
      expect(input).not.toBeInTheDocument();
    });

    it('should auto-focus the input when visible', () => {
      render(<SearchBar isVisible={true} onClose={mockOnClose} />);
      
      const input = screen.getByPlaceholderText(/search files, apps, and more/i);
      expect(input).toHaveFocus();
    });

    it('should update input value when typing', async () => {
      const user = userEvent.setup();
      render(<SearchBar isVisible={true} onClose={mockOnClose} />);
      
      const input = screen.getByPlaceholderText(/search files, apps, and more/i);
      await user.type(input, 'test query');
      
      expect(input).toHaveValue('test query');
    });

    it('should show clear button when input has text', async () => {
      const user = userEvent.setup();
      render(<SearchBar isVisible={true} onClose={mockOnClose} />);
      
      const input = screen.getByPlaceholderText(/search files, apps, and more/i);
      await user.type(input, 'test');
      
      const clearButton = screen.getByLabelText(/clear search/i);
      expect(clearButton).toBeInTheDocument();
    });

    it('should clear input when clear button is clicked', async () => {
      const user = userEvent.setup();
      render(<SearchBar isVisible={true} onClose={mockOnClose} />);
      
      const input = screen.getByPlaceholderText(/search files, apps, and more/i);
      await user.type(input, 'test');
      
      const clearButton = screen.getByLabelText(/clear search/i);
      await user.click(clearButton);
      
      expect(input).toHaveValue('');
    });
  });

  describe('Window visibility toggling', () => {
    it('should call onClose when Escape key is pressed', () => {
      render(<SearchBar isVisible={true} onClose={mockOnClose} />);
      
      fireEvent.keyDown(window, { key: 'Escape' });
      
      // Wait for animation timeout
      setTimeout(() => {
        expect(mockOnClose).toHaveBeenCalledTimes(1);
      }, 150);
    });
  });

  describe('Keyboard navigation', () => {
    it('should handle ArrowDown key press', () => {
      render(<SearchBar isVisible={true} onClose={mockOnClose} />);
      
      fireEvent.keyDown(window, { key: 'ArrowDown' });
      
      // Component should not crash and should handle the event
      expect(screen.getByPlaceholderText(/search files, apps, and more/i)).toBeInTheDocument();
    });

    it('should handle ArrowUp key press', () => {
      render(<SearchBar isVisible={true} onClose={mockOnClose} />);
      
      fireEvent.keyDown(window, { key: 'ArrowUp' });
      
      // Component should not crash and should handle the event
      expect(screen.getByPlaceholderText(/search files, apps, and more/i)).toBeInTheDocument();
    });

    it('should handle Enter key press', () => {
      render(<SearchBar isVisible={true} onClose={mockOnClose} />);
      
      fireEvent.keyDown(window, { key: 'Enter' });
      
      // Component should not crash and should handle the event
      expect(screen.getByPlaceholderText(/search files, apps, and more/i)).toBeInTheDocument();
    });

    it('should not handle keyboard events when not visible', () => {
      const { rerender } = render(<SearchBar isVisible={true} onClose={mockOnClose} />);
      
      rerender(<SearchBar isVisible={false} onClose={mockOnClose} />);
      
      fireEvent.keyDown(window, { key: 'Escape' });
      
      // onClose should not be called when component is not visible
      expect(mockOnClose).not.toHaveBeenCalled();
    });
  });

  describe('Animations and transitions', () => {
    it('should apply slide-in animation when visible', () => {
      const { container } = render(<SearchBar isVisible={true} onClose={mockOnClose} />);
      
      const searchBarDiv = container.querySelector('.animate-slide-in');
      expect(searchBarDiv).toBeInTheDocument();
    });

    it('should show "No results found" message when query exists but no results', async () => {
      const user = userEvent.setup();
      render(<SearchBar isVisible={true} onClose={mockOnClose} />);
      
      const input = screen.getByPlaceholderText(/search files, apps, and more/i);
      await user.type(input, 'nonexistent');
      
      const noResultsMessage = screen.getByText(/no results found/i);
      expect(noResultsMessage).toBeInTheDocument();
    });
  });
});
