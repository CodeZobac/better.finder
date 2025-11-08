import { describe, it, expect, vi } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useKeyboard, useKeyboardSelection } from './useKeyboard';

describe('useKeyboard', () => {
  it('should call onArrowDown when ArrowDown key is pressed', () => {
    const onArrowDown = vi.fn();
    const onArrowUp = vi.fn();
    const onEnter = vi.fn();
    const onEscape = vi.fn();

    renderHook(() =>
      useKeyboard({
        onArrowDown,
        onArrowUp,
        onEnter,
        onEscape,
        enabled: true,
      })
    );

    act(() => {
      const event = new KeyboardEvent('keydown', { key: 'ArrowDown' });
      window.dispatchEvent(event);
    });

    expect(onArrowDown).toHaveBeenCalledTimes(1);
  });

  it('should call onArrowUp when ArrowUp key is pressed', () => {
    const onArrowDown = vi.fn();
    const onArrowUp = vi.fn();
    const onEnter = vi.fn();
    const onEscape = vi.fn();

    renderHook(() =>
      useKeyboard({
        onArrowDown,
        onArrowUp,
        onEnter,
        onEscape,
        enabled: true,
      })
    );

    act(() => {
      const event = new KeyboardEvent('keydown', { key: 'ArrowUp' });
      window.dispatchEvent(event);
    });

    expect(onArrowUp).toHaveBeenCalledTimes(1);
  });

  it('should call onEnter when Enter key is pressed', () => {
    const onArrowDown = vi.fn();
    const onArrowUp = vi.fn();
    const onEnter = vi.fn();
    const onEscape = vi.fn();

    renderHook(() =>
      useKeyboard({
        onArrowDown,
        onArrowUp,
        onEnter,
        onEscape,
        enabled: true,
      })
    );

    act(() => {
      const event = new KeyboardEvent('keydown', { key: 'Enter' });
      window.dispatchEvent(event);
    });

    expect(onEnter).toHaveBeenCalledTimes(1);
  });

  it('should call onEscape when Escape key is pressed', () => {
    const onArrowDown = vi.fn();
    const onArrowUp = vi.fn();
    const onEnter = vi.fn();
    const onEscape = vi.fn();

    renderHook(() =>
      useKeyboard({
        onArrowDown,
        onArrowUp,
        onEnter,
        onEscape,
        enabled: true,
      })
    );

    act(() => {
      const event = new KeyboardEvent('keydown', { key: 'Escape' });
      window.dispatchEvent(event);
    });

    expect(onEscape).toHaveBeenCalledTimes(1);
  });

  it('should not call handlers when disabled', () => {
    const onArrowDown = vi.fn();
    const onArrowUp = vi.fn();
    const onEnter = vi.fn();
    const onEscape = vi.fn();

    renderHook(() =>
      useKeyboard({
        onArrowDown,
        onArrowUp,
        onEnter,
        onEscape,
        enabled: false,
      })
    );

    act(() => {
      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown' }));
      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowUp' }));
      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter' }));
      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));
    });

    expect(onArrowDown).not.toHaveBeenCalled();
    expect(onArrowUp).not.toHaveBeenCalled();
    expect(onEnter).not.toHaveBeenCalled();
    expect(onEscape).not.toHaveBeenCalled();
  });
});

describe('useKeyboardSelection', () => {
  it('should initialize with the provided initial index', () => {
    const { result } = renderHook(() => useKeyboardSelection(5, 2));
    expect(result.current.selectedIndex).toBe(2);
  });

  it('should move down to the next item', () => {
    const { result } = renderHook(() => useKeyboardSelection(5, 0));

    act(() => {
      result.current.moveDown();
    });

    expect(result.current.selectedIndex).toBe(1);
  });

  it('should move up to the previous item', () => {
    const { result } = renderHook(() => useKeyboardSelection(5, 2));

    act(() => {
      result.current.moveUp();
    });

    expect(result.current.selectedIndex).toBe(1);
  });

  it('should wrap to the last item when moving up from the first item', () => {
    const { result } = renderHook(() => useKeyboardSelection(5, 0));

    act(() => {
      result.current.moveUp();
    });

    expect(result.current.selectedIndex).toBe(4);
  });

  it('should wrap to the first item when moving down from the last item', () => {
    const { result } = renderHook(() => useKeyboardSelection(5, 4));

    act(() => {
      result.current.moveDown();
    });

    expect(result.current.selectedIndex).toBe(0);
  });

  it('should reset selection to 0', () => {
    const { result } = renderHook(() => useKeyboardSelection(5, 3));

    act(() => {
      result.current.resetSelection();
    });

    expect(result.current.selectedIndex).toBe(0);
  });

  it('should handle empty list (itemCount = 0)', () => {
    const { result } = renderHook(() => useKeyboardSelection(0, 0));

    act(() => {
      result.current.moveDown();
    });

    expect(result.current.selectedIndex).toBe(0);

    act(() => {
      result.current.moveUp();
    });

    expect(result.current.selectedIndex).toBe(0);
  });

  it('should adjust selectedIndex when itemCount decreases below current index', () => {
    const { result, rerender } = renderHook(
      ({ count }) => useKeyboardSelection(count, 0),
      { initialProps: { count: 5 } }
    );

    act(() => {
      result.current.setSelectedIndex(4);
    });

    expect(result.current.selectedIndex).toBe(4);

    rerender({ count: 3 });

    expect(result.current.selectedIndex).toBe(2);
  });
});
