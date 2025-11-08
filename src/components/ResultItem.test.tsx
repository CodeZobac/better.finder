import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import ResultItem from './ResultItem';
import { SearchResult, ResultType, ActionType } from '../types';

describe('ResultItem', () => {
  const mockOnSelect = vi.fn();
  const mockOnExecute = vi.fn();

  const createMockResult = (type: ResultType): SearchResult => ({
    id: '1',
    title: 'Test Result',
    subtitle: 'Test subtitle',
    icon: null,
    type,
    score: 100,
    metadata: {},
    action: {
      type: ActionType.OpenFile,
      payload: '/test/path',
    },
  });

  beforeEach(() => {
    mockOnSelect.mockClear();
    mockOnExecute.mockClear();
  });

  describe('Rendering with different result types', () => {
    it('should render file result with correct badge', () => {
      const result = createMockResult(ResultType.File);
      render(
        <ResultItem
          result={result}
          isSelected={false}
          onSelect={mockOnSelect}
          onExecute={mockOnExecute}
        />
      );

      expect(screen.getByText('Test Result')).toBeInTheDocument();
      expect(screen.getByText('Test subtitle')).toBeInTheDocument();
      expect(screen.getByText('File')).toBeInTheDocument();
    });

    it('should render application result with correct badge', () => {
      const result = createMockResult(ResultType.Application);
      render(
        <ResultItem
          result={result}
          isSelected={false}
          onSelect={mockOnSelect}
          onExecute={mockOnExecute}
        />
      );

      expect(screen.getByText('App')).toBeInTheDocument();
    });

    it('should render calculator result with correct badge', () => {
      const result = createMockResult(ResultType.Calculator);
      render(
        <ResultItem
          result={result}
          isSelected={false}
          onSelect={mockOnSelect}
          onExecute={mockOnExecute}
        />
      );

      expect(screen.getByText('Calc')).toBeInTheDocument();
    });

    it('should render quick action result with correct badge', () => {
      const result = createMockResult(ResultType.QuickAction);
      render(
        <ResultItem
          result={result}
          isSelected={false}
          onSelect={mockOnSelect}
          onExecute={mockOnExecute}
        />
      );

      expect(screen.getByText('Action')).toBeInTheDocument();
    });
  });

  describe('Selection states', () => {
    it('should apply selected styles when isSelected is true', () => {
      const result = createMockResult(ResultType.File);
      const { container } = render(
        <ResultItem
          result={result}
          isSelected={true}
          onSelect={mockOnSelect}
          onExecute={mockOnExecute}
        />
      );

      const itemDiv = container.firstChild as HTMLElement;
      expect(itemDiv.className).toContain('bg-surface-hover');
      expect(itemDiv.className).toContain('translate-x-1');
    });

    it('should not apply selected styles when isSelected is false', () => {
      const result = createMockResult(ResultType.File);
      const { container } = render(
        <ResultItem
          result={result}
          isSelected={false}
          onSelect={mockOnSelect}
          onExecute={mockOnExecute}
        />
      );

      const itemDiv = container.firstChild as HTMLElement;
      // Should have hover class but not the active bg-surface-hover without hover prefix
      expect(itemDiv.className).toContain('hover:bg-surface-hover');
      expect(itemDiv.className).not.toMatch(/^.*\sbg-surface-hover\s.*$/);
    });
  });

  describe('User interactions', () => {
    it('should call onSelect when mouse enters', async () => {
      const result = createMockResult(ResultType.File);
      const { container } = render(
        <ResultItem
          result={result}
          isSelected={false}
          onSelect={mockOnSelect}
          onExecute={mockOnExecute}
        />
      );

      const itemDiv = container.firstChild as HTMLElement;
      await userEvent.hover(itemDiv);

      expect(mockOnSelect).toHaveBeenCalledTimes(1);
    });

    it('should call onSelect and onExecute when clicked', async () => {
      const user = userEvent.setup();
      const result = createMockResult(ResultType.File);
      const { container } = render(
        <ResultItem
          result={result}
          isSelected={false}
          onSelect={mockOnSelect}
          onExecute={mockOnExecute}
        />
      );

      const itemDiv = container.firstChild as HTMLElement;
      await user.click(itemDiv);

      // Click triggers both hover (onSelect) and click (onSelect + onExecute)
      expect(mockOnSelect).toHaveBeenCalled();
      expect(mockOnExecute).toHaveBeenCalledTimes(1);
    });
  });

  describe('Highlight rendering', () => {
    it('should render highlighted title when provided', () => {
      const result = createMockResult(ResultType.File);
      const highlightedTitle = (
        <>
          <span className="text-primary font-semibold">T</span>
          <span>est Result</span>
        </>
      );

      render(
        <ResultItem
          result={result}
          isSelected={false}
          onSelect={mockOnSelect}
          onExecute={mockOnExecute}
          highlightedTitle={highlightedTitle}
        />
      );

      // The highlighted title should be rendered
      const titleElement = screen.getByText('est Result');
      expect(titleElement).toBeInTheDocument();
    });

    it('should render plain title when no highlight provided', () => {
      const result = createMockResult(ResultType.File);
      render(
        <ResultItem
          result={result}
          isSelected={false}
          onSelect={mockOnSelect}
          onExecute={mockOnExecute}
        />
      );

      expect(screen.getByText('Test Result')).toBeInTheDocument();
    });
  });

  describe('Icon rendering', () => {
    it('should render base64 image icon when provided', () => {
      const result = {
        ...createMockResult(ResultType.File),
        icon: 'data:image/png;base64,test',
      };

      const { container } = render(
        <ResultItem
          result={result}
          isSelected={false}
          onSelect={mockOnSelect}
          onExecute={mockOnExecute}
        />
      );

      const img = container.querySelector('img');
      expect(img).toBeInTheDocument();
      expect(img?.src).toContain('data:image/png;base64,test');
    });

    it('should render default icon when no icon provided', () => {
      const result = createMockResult(ResultType.File);
      const { container } = render(
        <ResultItem
          result={result}
          isSelected={false}
          onSelect={mockOnSelect}
          onExecute={mockOnExecute}
        />
      );

      // Should render an SVG icon (lucide-react)
      const svg = container.querySelector('svg');
      expect(svg).toBeInTheDocument();
    });
  });
});
