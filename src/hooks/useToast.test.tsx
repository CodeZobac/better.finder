import { describe, it, expect } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useToast } from './useToast';

describe('useToast Hook', () => {
  it('initializes with empty toasts array', () => {
    const { result } = renderHook(() => useToast());

    expect(result.current.toasts).toEqual([]);
  });

  it('adds a success toast', () => {
    const { result } = renderHook(() => useToast());

    act(() => {
      result.current.success('Success!', 'Operation completed');
    });

    expect(result.current.toasts).toHaveLength(1);
    expect(result.current.toasts[0].type).toBe('success');
    expect(result.current.toasts[0].title).toBe('Success!');
    expect(result.current.toasts[0].message).toBe('Operation completed');
  });

  it('adds an error toast', () => {
    const { result } = renderHook(() => useToast());

    act(() => {
      result.current.error('Error!', 'Something went wrong');
    });

    expect(result.current.toasts).toHaveLength(1);
    expect(result.current.toasts[0].type).toBe('error');
    expect(result.current.toasts[0].title).toBe('Error!');
  });

  it('adds a warning toast', () => {
    const { result } = renderHook(() => useToast());

    act(() => {
      result.current.warning('Warning!');
    });

    expect(result.current.toasts).toHaveLength(1);
    expect(result.current.toasts[0].type).toBe('warning');
    expect(result.current.toasts[0].title).toBe('Warning!');
  });

  it('adds an info toast', () => {
    const { result } = renderHook(() => useToast());

    act(() => {
      result.current.info('Info');
    });

    expect(result.current.toasts).toHaveLength(1);
    expect(result.current.toasts[0].type).toBe('info');
  });

  it('adds multiple toasts', () => {
    const { result } = renderHook(() => useToast());

    act(() => {
      result.current.success('First');
      result.current.error('Second');
      result.current.warning('Third');
    });

    expect(result.current.toasts).toHaveLength(3);
    expect(result.current.toasts[0].title).toBe('First');
    expect(result.current.toasts[1].title).toBe('Second');
    expect(result.current.toasts[2].title).toBe('Third');
  });

  it('closes a toast by id', () => {
    const { result } = renderHook(() => useToast());

    let toastId: string;
    act(() => {
      toastId = result.current.success('Test');
    });

    expect(result.current.toasts).toHaveLength(1);

    act(() => {
      result.current.closeToast(toastId);
    });

    expect(result.current.toasts).toHaveLength(0);
  });

  it('generates unique ids for each toast', () => {
    const { result } = renderHook(() => useToast());

    let id1 = '';
    let id2 = '';
    let id3 = '';
    
    act(() => {
      id1 = result.current.success('First');
      id2 = result.current.success('Second');
      id3 = result.current.success('Third');
    });

    expect(id1).not.toBe(id2);
    expect(id2).not.toBe(id3);
    expect(id1).not.toBe(id3);
  });

  it('supports custom duration', () => {
    const { result } = renderHook(() => useToast());

    act(() => {
      result.current.success('Test', 'Message', 10000);
    });

    expect(result.current.toasts[0].duration).toBe(10000);
  });

  it('closes only the specified toast', () => {
    const { result } = renderHook(() => useToast());

    let id1 = '';
    let id2 = '';
    
    act(() => {
      id1 = result.current.success('First');
      id2 = result.current.error('Second');
    });

    expect(result.current.toasts).toHaveLength(2);

    act(() => {
      result.current.closeToast(id1);
    });

    expect(result.current.toasts).toHaveLength(1);
    expect(result.current.toasts[0].id).toBe(id2);
  });
});
