import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/react';
import { highlightText, highlightMatches } from './highlight';
import Fuzzysort from 'fuzzysort';

describe('highlight utility', () => {
  describe('highlightText', () => {
    it('should return plain text when query is empty', () => {
      const result = highlightText('test string', '');
      expect(result).toBe('test string');
    });

    it('should return plain text when text is empty', () => {
      const result = highlightText('', 'query');
      expect(result).toBe('');
    });

    it('should highlight matching characters', () => {
      const result = highlightText('Document', 'doc');
      const { container } = render(<>{result}</>);

      // Should contain highlighted spans
      const highlightedSpans = container.querySelectorAll('.text-primary');
      expect(highlightedSpans.length).toBeGreaterThan(0);
    });

    it('should handle case-insensitive matching', () => {
      const result = highlightText('Document', 'DOC');
      const { container } = render(<>{result}</>);

      // Should still highlight despite case difference
      const highlightedSpans = container.querySelectorAll('.text-primary');
      expect(highlightedSpans.length).toBeGreaterThan(0);
    });

    it('should handle partial matches', () => {
      const result = highlightText('Application', 'app');
      const { container } = render(<>{result}</>);

      const highlightedSpans = container.querySelectorAll('.text-primary');
      expect(highlightedSpans.length).toBeGreaterThan(0);
    });
  });

  describe('highlightMatches', () => {
    it('should return null when result is null', () => {
      const result = highlightMatches(null);
      expect(result).toBeNull();
    });

    it('should return plain text when no indexes', () => {
      const fuzzysortResult = Fuzzysort.single('test', 'test string');
      if (fuzzysortResult) {
        // Clear indexes to simulate no matches
        const modifiedResult = { ...fuzzysortResult, indexes: [] };
        const result = highlightMatches(modifiedResult);
        expect(result).toBe('test string');
      }
    });

    it('should highlight characters at specified indexes', () => {
      const fuzzysortResult = Fuzzysort.single('doc', 'Document');
      if (fuzzysortResult) {
        const result = highlightMatches(fuzzysortResult);
        const { container } = render(<>{result}</>);

        // Should have highlighted spans with primary color
        const highlightedSpans = container.querySelectorAll('.text-primary.font-semibold');
        expect(highlightedSpans.length).toBeGreaterThan(0);
      }
    });

    it('should handle multiple highlight spans', () => {
      const fuzzysortResult = Fuzzysort.single('abc', 'a test b string c');
      if (fuzzysortResult) {
        const result = highlightMatches(fuzzysortResult);
        const { container } = render(<>{result}</>);

        // Should have multiple highlighted characters
        const highlightedSpans = container.querySelectorAll('.text-primary.font-semibold');
        expect(highlightedSpans.length).toBeGreaterThanOrEqual(3);
      }
    });

    it('should preserve non-highlighted text', () => {
      const fuzzysortResult = Fuzzysort.single('d', 'Document');
      if (fuzzysortResult) {
        const result = highlightMatches(fuzzysortResult);
        const { container } = render(<>{result}</>);

        // Should contain both highlighted and non-highlighted text
        expect(container.textContent).toBe('Document');
      }
    });
  });

  describe('highlight styling', () => {
    it('should apply primary color to highlighted characters', () => {
      const result = highlightText('Test', 't');
      const { container } = render(<>{result}</>);

      const highlightedSpan = container.querySelector('.text-primary');
      expect(highlightedSpan).toBeInTheDocument();
    });

    it('should apply font-semibold to highlighted characters', () => {
      const result = highlightText('Test', 't');
      const { container } = render(<>{result}</>);

      const highlightedSpan = container.querySelector('.font-semibold');
      expect(highlightedSpan).toBeInTheDocument();
    });
  });
});
