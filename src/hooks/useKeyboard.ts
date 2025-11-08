import { useEffect, useCallback, useState } from 'react';

export interface UseKeyboardOptions {
  onArrowUp: () => void;
  onArrowDown: () => void;
  onEnter: () => void;
  onEscape: () => void;
  enabled?: boolean;
}

/**
 * Custom hook for handling keyboard navigation
 * Handles ArrowUp, ArrowDown, Enter, and Escape keys
 */
export const useKeyboard = ({
  onArrowUp,
  onArrowDown,
  onEnter,
  onEscape,
  enabled = true,
}: UseKeyboardOptions) => {
  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      if (!enabled) return;

      switch (event.key) {
        case 'ArrowUp':
          event.preventDefault();
          onArrowUp();
          break;
        case 'ArrowDown':
          event.preventDefault();
          onArrowDown();
          break;
        case 'Enter':
          event.preventDefault();
          onEnter();
          break;
        case 'Escape':
          event.preventDefault();
          onEscape();
          break;
      }
    },
    [enabled, onArrowUp, onArrowDown, onEnter, onEscape]
  );

  useEffect(() => {
    if (!enabled) return;

    window.addEventListener('keydown', handleKeyDown);

    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [enabled, handleKeyDown]);
};

/**
 * Custom hook for managing selection state with keyboard navigation
 * Supports wrapping from last to first result
 */
export const useKeyboardSelection = (
  itemCount: number,
  initialIndex: number = 0
) => {
  const [selectedIndex, setSelectedIndex] = useState(initialIndex);

  // Reset selection when item count changes
  useEffect(() => {
    if (itemCount === 0) {
      setSelectedIndex(0);
    } else if (selectedIndex >= itemCount) {
      setSelectedIndex(Math.max(0, itemCount - 1));
    }
  }, [itemCount, selectedIndex]);

  const moveUp = useCallback(() => {
    setSelectedIndex((prev) => {
      if (itemCount === 0) return 0;
      // Wrap to last item if at first item
      return prev <= 0 ? itemCount - 1 : prev - 1;
    });
  }, [itemCount]);

  const moveDown = useCallback(() => {
    setSelectedIndex((prev) => {
      if (itemCount === 0) return 0;
      // Wrap to first item if at last item
      return prev >= itemCount - 1 ? 0 : prev + 1;
    });
  }, [itemCount]);

  const resetSelection = useCallback(() => {
    setSelectedIndex(0);
  }, []);

  return {
    selectedIndex,
    setSelectedIndex,
    moveUp,
    moveDown,
    resetSelection,
  };
};
