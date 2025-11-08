import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import ResultGroup from './ResultGroup';
import { SearchResult, ResultType, ActionType } from '../types';

describe('ResultGroup', () => {
  const mockOnSelectResult = vi.fn();
  const mockOnExecuteResult = vi.fn();

  const createMockResults = (): SearchResult[] => [
    {
      id: '1',
      title: 'Document.pdf',
      subtitle: '/home/user/documents',
      icon: null,
      type: ResultType.File,
      score: 100,
      metadata: {},
      action: { type: ActionType.OpenFile, payload: '/home/user/documents/Document.pdf' },
    },
    {
      id: '2',
      title: 'Chrome',
      subtitle: 'Web Browser',
      icon: null,
      type: ResultType.Application,
      score: 90,
      metadata: {},
      action: { type: ActionType.LaunchApp, payload: 'chrome.exe' },
    },
    {
      id: '3',
      title: 'Shutdown',
      subtitle: 'Turn off computer',
      icon: null,
      type: ResultType.QuickAction,
      score: 80,
      metadata: {},
      action: { type: ActionType.ExecuteCommand, payload: 'shutdown' },
    },
  ];

  beforeEach(() => {
    mockOnSelectResult.mockClear();
    mockOnExecuteResult.mockClear();
  });

  describe('Grouping logic', () => {
    it('should render results grouped by type', () => {
      const results = createMockResults();
      render(
        <ResultGroup
          results={results}
          selectedIndex={0}
          onSelectResult={mockOnSelectResult}
          onExecuteResult={mockOnExecuteResult}
        />
      );

      // Check for group headers
      expect(screen.getByText('FILES')).toBeInTheDocument();
      expect(screen.getByText('APPLICATIONS')).toBeInTheDocument();
      expect(screen.getByText('QUICK ACTIONS')).toBeInTheDocument();
    });

    it('should render all results within their groups', () => {
      const results = createMockResults();
      render(
        <ResultGroup
          results={results}
          selectedIndex={0}
          onSelectResult={mockOnSelectResult}
          onExecuteResult={mockOnExecuteResult}
        />
      );

      expect(screen.getByText('Document.pdf')).toBeInTheDocument();
      expect(screen.getByText('Chrome')).toBeInTheDocument();
      expect(screen.getByText('Shutdown')).toBeInTheDocument();
    });

    it('should group multiple results of the same type together', () => {
      const results: SearchResult[] = [
        {
          id: '1',
          title: 'File1.txt',
          subtitle: '/path1',
          icon: null,
          type: ResultType.File,
          score: 100,
          metadata: {},
          action: { type: ActionType.OpenFile, payload: '/path1/File1.txt' },
        },
        {
          id: '2',
          title: 'File2.txt',
          subtitle: '/path2',
          icon: null,
          type: ResultType.File,
          score: 90,
          metadata: {},
          action: { type: ActionType.OpenFile, payload: '/path2/File2.txt' },
        },
      ];

      render(
        <ResultGroup
          results={results}
          selectedIndex={0}
          onSelectResult={mockOnSelectResult}
          onExecuteResult={mockOnExecuteResult}
        />
      );

      // Should only have one FILES header
      const filesHeaders = screen.getAllByText('FILES');
      expect(filesHeaders).toHaveLength(1);

      // Both files should be present
      expect(screen.getByText('File1.txt')).toBeInTheDocument();
      expect(screen.getByText('File2.txt')).toBeInTheDocument();
    });
  });

  describe('Group headers', () => {
    it('should render group headers in uppercase', () => {
      const results = createMockResults();
      const { container } = render(
        <ResultGroup
          results={results}
          selectedIndex={0}
          onSelectResult={mockOnSelectResult}
          onExecuteResult={mockOnExecuteResult}
        />
      );

      const headers = container.querySelectorAll('.uppercase');
      expect(headers.length).toBeGreaterThan(0);
    });

    it('should style group headers correctly', () => {
      const results = createMockResults();
      render(
        <ResultGroup
          results={results}
          selectedIndex={0}
          onSelectResult={mockOnSelectResult}
          onExecuteResult={mockOnExecuteResult}
        />
      );

      const header = screen.getByText('FILES');
      expect(header).toBeInTheDocument();
      // Check that the header has the correct styling classes
      expect(header.className).toContain('text-xs');
      expect(header.className).toContain('font-semibold');
      expect(header.className).toContain('uppercase');
    });
  });

  describe('Visual separators', () => {
    it('should render separators between groups', () => {
      const results = createMockResults();
      const { container } = render(
        <ResultGroup
          results={results}
          selectedIndex={0}
          onSelectResult={mockOnSelectResult}
          onExecuteResult={mockOnExecuteResult}
        />
      );

      const separators = container.querySelectorAll('.border-t');
      // Should have separators between groups (n-1 separators for n groups)
      expect(separators.length).toBeGreaterThan(0);
    });
  });

  describe('Empty results', () => {
    it('should render nothing when results array is empty', () => {
      const { container } = render(
        <ResultGroup
          results={[]}
          selectedIndex={0}
          onSelectResult={mockOnSelectResult}
          onExecuteResult={mockOnExecuteResult}
        />
      );

      expect(container.firstChild).toBeNull();
    });
  });

  describe('Highlighting', () => {
    it('should pass query to ResultItem for highlighting', () => {
      const results = createMockResults();
      const { container } = render(
        <ResultGroup
          results={results}
          selectedIndex={0}
          onSelectResult={mockOnSelectResult}
          onExecuteResult={mockOnExecuteResult}
          query="doc"
        />
      );

      // Results should still be rendered (text may be split by highlighting)
      expect(container.textContent).toContain('Document.pdf');
    });
  });

  describe('Virtual scrolling', () => {
    it('should use regular rendering for small result sets', () => {
      const results = createMockResults(); // 3 results
      render(
        <ResultGroup
          results={results}
          selectedIndex={0}
          onSelectResult={mockOnSelectResult}
          onExecuteResult={mockOnExecuteResult}
        />
      );

      // All results should be rendered for small lists
      expect(screen.getByText('Document.pdf')).toBeInTheDocument();
      expect(screen.getByText('Chrome')).toBeInTheDocument();
      expect(screen.getByText('Shutdown')).toBeInTheDocument();
    });

    it('should render large result sets', () => {
      // Create 10 results to trigger virtual scrolling
      const results: SearchResult[] = Array.from({ length: 10 }, (_, i) => ({
        id: `${i}`,
        title: `Result ${i}`,
        subtitle: `Subtitle ${i}`,
        icon: null,
        type: ResultType.File,
        score: 100 - i,
        metadata: {},
        action: { type: ActionType.OpenFile, payload: `/path${i}` },
      }));

      const { container } = render(
        <ResultGroup
          results={results}
          selectedIndex={0}
          onSelectResult={mockOnSelectResult}
          onExecuteResult={mockOnExecuteResult}
        />
      );

      // Check that results are rendered (virtual scrolling implementation detail)
      expect(container.textContent).toContain('Result');
      expect(container.textContent).toContain('FILES');
    });
  });
});
